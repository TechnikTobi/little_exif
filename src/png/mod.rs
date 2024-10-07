// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub mod file;
mod png_chunk;

use miniz_oxide::deflate::compress_to_vec_zlib;

use crate::general_file_io::EXIF_HEADER;
use crate::general_file_io::NEWLINE;
use crate::general_file_io::SPACE;

pub(crate) const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
pub(crate) const RAW_PROFILE_TYPE_EXIF: [u8; 23] = [
	0x52, 0x61, 0x77, 0x20,                             // Raw
	0x70, 0x72, 0x6F, 0x66, 0x69, 0x6C, 0x65, 0x20,     // profile
	0x74, 0x79, 0x70, 0x65, 0x20,                       // type
	0x65, 0x78, 0x69, 0x66, 0x00, 0x00                  // exif NUL NUL
];

// The bytes during encoding need to be encoded themselves:
// A given byte (e.g. 0x30 for the char '0') has two values in the string of its hex representation ('3' and '0')
// These two characters need to be encoded themselves (51 for '3', 48 for '0'), resulting in the final encoded
// version of the EXIF data
// Independent of endian as this does not affect the ordering of values WITHIN a byte 
fn encode_byte(byte: &u8) -> [u8; 2] 
{
	[
		byte / 16 + (if byte / 16 < 10 {'0' as u8} else {'a' as u8 - 10}),
		byte % 16 + (if byte % 16 < 10 {'0' as u8} else {'a' as u8 - 10}) 
	]
}

fn
encode_metadata_png
(
	exif_vec: &Vec<u8>
)
-> Vec<u8>
{
	// The size of the EXIF data area, consists of
	// - length of EXIF header (follows after ssss)
	// - length of exif_vec
	// - 1 for ssss itself (why not 4? idk)
	let ssss = (
		EXIF_HEADER.len() as u32 
		+ exif_vec.len()  as u32 
		+ 1               as u32
	).to_string();

	// Construct final vector with the bytes as they will be sent to the encoder
	//                               \n       e     x     i     f     \n
	let mut png_exif: Vec<u8> = vec![NEWLINE, 0x65, 0x78, 0x69, 0x66, NEWLINE];

	// Write ssss
	for _ in 0..(8-ssss.len())
	{
		png_exif.push(SPACE);
	}
	png_exif.extend(ssss.as_bytes().to_vec().iter());
	png_exif.push(NEWLINE);

	// Write EXIF header and previously constructed EXIF data as encoded bytes
	for byte in &EXIF_HEADER
	{
		png_exif.extend(encode_byte(byte).iter());
	}

	for byte in exif_vec
	{
		png_exif.extend(encode_byte(byte).iter());
	}
	
	// Write end of EXIF data - 2* 0x30 results in the String "00" for 0x00
	png_exif.push(0x30);
	png_exif.push(0x30);
	png_exif.push(NEWLINE);

	return png_exif;
}

/// Provides the WebP specific encoding result as vector of bytes to be used
/// by the user (e.g. in combination with another library)
#[allow(non_snake_case)]
pub(crate) fn
as_u8_vec
(
	general_encoded_metadata: &Vec<u8>,
	as_zTXt_chunk:            bool
)
-> Vec<u8>
{
	let basic_png_encode_result = encode_metadata_png(general_encoded_metadata);

	if !as_zTXt_chunk
	{
		return basic_png_encode_result;
	}

	// Build data of new chunk using zlib compression (level=8 -> default)
	let mut zTXt_chunk_data: Vec<u8> = vec![0x7a, 0x54, 0x58, 0x74];
	zTXt_chunk_data.extend(RAW_PROFILE_TYPE_EXIF.iter());
	zTXt_chunk_data.extend(compress_to_vec_zlib(&basic_png_encode_result, 8).iter());

	return zTXt_chunk_data;
}