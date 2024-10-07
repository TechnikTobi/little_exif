// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub mod file;
pub mod vec;
mod png_chunk;

use std::collections::VecDeque;

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

fn
decode_metadata_png
(
	encoded_data: &Vec<u8>
)
-> Result<Vec<u8>, std::io::Error>
{

	let mut exif_all: VecDeque<u8> = VecDeque::new();
	let mut other_byte: Option<u8> = None;

	// This performs the reverse operation to encode_byte:
	// Two succeeding bytes represent the ASCII values of the digits of 
	// a hex value, e.g. 0x31, 0x32 represent '1' and '2', so the resulting
	// hex value is 0x12, which gets pushed onto exif_all
	for byte in encoded_data
	{
		// Ignore newline characters
		if *byte == '\n' as u8
		{
			continue;
		}

		if other_byte.is_none()
		{
			other_byte = Some(*byte);
			continue;
		}

		let value_string = "".to_owned()
			+ &(other_byte.unwrap() as char).to_string()
			+ &(*byte as char).to_string();
			
		if let Ok(value) = u8::from_str_radix(value_string.trim(), 16)
		{
			exif_all.push_back(value);
		}
		
		other_byte = None;
	}

	// Now remove the first element until the exif header is found
	// Store the popped elements to get the size information
	let mut exif_header_found = false;
	let mut pop_storage: Vec<u8> = Vec::new();

	while !exif_header_found
	{
		let mut counter = 0;
		for header_value in &EXIF_HEADER
		{
			if *header_value != exif_all[counter]
			{
				break;
			}
			counter += 1;
		}

		exif_header_found = counter == EXIF_HEADER.len();

		if exif_header_found
		{
			break;
		}
		pop_storage.push(exif_all.pop_front().unwrap());
	}

	// The exif header has been found
	// -> exif_all now starts with the exif header information
	// -> pop_storage has in its last 4 elements the size information
	//    that will now get extracted
	// Consider this part optional as it might be removed in the future and
	// isn't strictly necessary and just for validating the data we get
	assert!(pop_storage.len() > 0);

	// Using the encode_byte function re-encode the bytes regarding the size
	// information and construct its value using decimal based shifting
	// Example: 153 = 0
	// + 5*10*10^(2*0) + 3*1*10^(2*0) 
	// + 0*10*10^(2*1) + 1*1*10^(2*1)
	let mut given_exif_len = 0u64;
	for i in 0..std::cmp::min(4, pop_storage.len())
	{
		let re_encoded_byte = encode_byte(&pop_storage[pop_storage.len() -1 -i]);
		let tens_place = u64::from_str_radix(&(re_encoded_byte[0] as char).to_string(), 10).unwrap();
		let ones_place = u64::from_str_radix(&(re_encoded_byte[1] as char).to_string(), 10).unwrap();
		given_exif_len = given_exif_len + tens_place * 10 * 10_u64.pow((2 * i).try_into().unwrap());
		given_exif_len = given_exif_len + ones_place *  1 * 10_u64.pow((2 * i).try_into().unwrap());
	}

	assert!(given_exif_len == exif_all.len().try_into().unwrap());
	// End optional part

	return Ok(Vec::from(exif_all));
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