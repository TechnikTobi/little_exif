use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;
use std::collections::VecDeque;

use crc::{Crc, CRC_32_ISO_HDLC};
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec_zlib;

use crate::png_chunk::PngChunk;
use crate::general_file_io::*;

pub const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
pub const RAW_PROFILE_TYPE_EXIF: [u8; 23] = [
	0x52, 0x61, 0x77, 0x20,								// Raw
	0x70, 0x72, 0x6F, 0x66, 0x69, 0x6C, 0x65, 0x20,		// profile
	0x74, 0x79, 0x70, 0x65, 0x20,						// type
	0x65, 0x78, 0x69, 0x66, 0x00, 0x00					// exif NUL NUL
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
		EXIF_HEADER.len()	as u32 
		+ exif_vec.len()	as u32 
		+ 1					as u32
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
	// Two successing bytes represent the ASCII values of the digits of 
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

fn
check_signature
(
	path: &Path
)
-> Result<File, std::io::Error>
{
	if !path.exists()
	{
		return io_error!(NotFound, "Can't open PNG file - File does not exist!");
	}

	let mut file = OpenOptions::new()
		.read(true)
		.open(path)
		.expect("Could not open file");
	
	// Check the signature
	let mut signature_buffer = [0u8; 8];
	file.read(&mut signature_buffer).unwrap();
	let signature_is_valid = signature_buffer.iter()
		.zip(PNG_SIGNATURE.iter())
		.filter(|&(read, constant)| read == constant)
		.count() == PNG_SIGNATURE.len();

	if !signature_is_valid
	{
		return io_error!(InvalidData, "Can't open PNG file - Wrong signature!");
	}

	// Signature is valid - can proceed using the file as PNG file
	return Ok(file);
}

// TODO: Check if this is also affected by endianness
// Edit: Should... not? I guess?
fn
get_next_chunk_descriptor
(
	file: &mut File
)
-> Result<PngChunk, std::io::Error>
{
	// Read the start of the chunk
	let mut chunk_start = [0u8; 8];
	let mut bytes_read = file.read(&mut chunk_start).unwrap();

	// Check that indeed 8 bytes were read
	if bytes_read != 8
	{
		return io_error!(Other, "Could not read start of chunk");
	}

	// Construct name of chunk and its length
	let chunk_name = String::from_utf8((&chunk_start[4..8]).to_vec());
	let mut chunk_length = 0u32;
	for byte in &chunk_start[0..4]
	{
		chunk_length = chunk_length * 256 + *byte as u32;
	}

	// Read chunk data ...
	let mut chunk_data_buffer = vec![0u8; chunk_length as usize];
	bytes_read = file.read(&mut chunk_data_buffer).unwrap();
	if bytes_read != chunk_length as usize
	{
		return io_error!(Other, "Could not read chunk data");
	}

	// ... and CRC values
	let mut chunk_crc_buffer = [0u8; 4];
	bytes_read = file.read(&mut chunk_crc_buffer).unwrap();
	if bytes_read != 4
	{
		return io_error!(Other, "Could not read chunk CRC");
	}

	// Compute CRC on chunk
	let mut crc_input = Vec::new();
	crc_input.extend(chunk_start[4..8].iter());
	crc_input.extend(chunk_data_buffer.iter());

	let crc_struct = Crc::<u32>::new(&CRC_32_ISO_HDLC);
	let checksum = crc_struct.checksum(&crc_input) as u32;

	for i in 0..4
	{
		if ((checksum >> (8 * (3-i))) as u8) != chunk_crc_buffer[i]
		{
			return io_error!(InvalidData, "Checksum check failed while reading PNG!");
		}
	}

	// If validating the chunk using the CRC was successful, return its descriptor
	// Note: chunk_length does NOT include the +4 for the CRC area!
	if let Ok(png_chunk) = PngChunk::from_string(
		&chunk_name.unwrap(),
		chunk_length
	)
	{
		return Ok(png_chunk);
	}
	else
	{
		return io_error!(Other, "Invalid PNG chunk name");
	}
}

pub fn
parse_png
(
	path: &Path
)
-> Result<Vec<PngChunk>, std::io::Error>
{
	let mut file = check_signature(path);
	let mut chunks = Vec::new();

	if file.is_err()
	{
		return Err(file.err().unwrap());
	}

	loop
	{
		let next_chunk_descriptor_result = get_next_chunk_descriptor(file.as_mut().unwrap());
		if let Ok(chunk_descriptor) = next_chunk_descriptor_result
		{
			chunks.push(chunk_descriptor);

			if chunks.last().unwrap().as_string() == "IEND".to_string()
			{
				break;
			}
		}
		else
		{
			return Err(next_chunk_descriptor_result.err().unwrap());
		}
	}

	return Ok(chunks);
}

// Clears existing metadata chunk from a png file
// Gets called before writing any new metadata
#[allow(non_snake_case)]
pub fn
clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{

	// Parse the PNG - if this fails, the clear operation fails as well
	let parse_png_result = parse_png(path);
	if let Err(error) = parse_png_result
	{
		return Err(error);
	}

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut file = check_signature(path).unwrap();
	let mut seek_counter = 0u64;

	for chunk in &parse_png_result.unwrap()
	{
		// If this is not a zTXt chunk, jump to the next chunk
		if chunk.as_string() != String::from("zTXt")
		{
			seek_counter += chunk.length() as u64 + 12;
			perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));
			continue;
		}

		// Skip chunk length and type (4+4 Bytes)
		perform_file_action!(file.seek(SeekFrom::Current(8)));

		// Read chunk data into buffer for checking that this is the 
		// correct chunk to delete
		let mut zTXt_chunk_data = vec![0u8; chunk.length() as usize];
		if file.read(&mut zTXt_chunk_data).unwrap() != chunk.length() as usize
		{
			return io_error!(Other, "Could not read chunk data");
		}

		// Compare to the "Raw profile type exif" string constant
		let mut correct_zTXt_chunk = true;
		for i in 0..RAW_PROFILE_TYPE_EXIF.len()
		{
			if zTXt_chunk_data[i] != RAW_PROFILE_TYPE_EXIF[i]
			{
				correct_zTXt_chunk = false;
			}
		}

		// If this is not the correct zTXt chunk, skip CRC from current
		// (wrong) zTXt chunk and continue with next chunk
		if !correct_zTXt_chunk
		{
			perform_file_action!(file.seek(SeekFrom::Current(4)));
			continue;
		}
		
		// We have now established that this is the correct chunk to delete
		// Therefore: Get to the next chunk...
		perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));

		// ...copy data from there onwards into a buffer...
		let mut buffer = Vec::new();
		perform_file_action!(file.read_to_end(&mut buffer));

		// ...go back to the chunk to be removed...
		perform_file_action!(file.seek(SeekFrom::Start(seek_counter)));

		// ...and overwrite it using the data from the buffer
		perform_file_action!(file.write_all(&buffer));
		perform_file_action!(file.seek(SeekFrom::Start(seek_counter)));		
	}

	return Ok(());
}

#[allow(non_snake_case)]
pub fn
read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	// Parse the PNG - if this fails, the read fails as well
	let parse_png_result = parse_png(path);
	if let Err(error) = parse_png_result
	{
		return Err(error);
	}

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut file = check_signature(path).unwrap();
	for chunk in &parse_png_result.unwrap()
	{
		// Wrong chunk? Seek to the next one
		if chunk.as_string() != String::from("zTXt")
		{
			perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));
		}

		// We now have a zTXt chunk:
		// Skip chunk length and type (4+4 Bytes)
		perform_file_action!(file.seek(SeekFrom::Current(8)));

		// Read chunk data into buffer
		// No need to verify this using CRC as already done by parse_png(path)
		let mut zTXt_chunk_data = vec![0u8; chunk.length() as usize];
		if file.read(&mut zTXt_chunk_data).unwrap() != chunk.length() as usize
		{
			return io_error!(Other, "Could not read chunk data");
		}

		// Check that this is the correct zTXt chunk...
		let mut correct_zTXt_chunk = true;
		for i in 0..RAW_PROFILE_TYPE_EXIF.len()
		{
			if zTXt_chunk_data[i] != RAW_PROFILE_TYPE_EXIF[i]
			{
				correct_zTXt_chunk = false;
			}
		}

		if !correct_zTXt_chunk
		{
			// Skip CRC from current (wrong) zTXt chunk and continue
			perform_file_action!(file.seek(SeekFrom::Current(4)));
			continue;
		}

		// Decode zlib data...
		if let Ok(decompressed_data) = decompress_to_vec_zlib(&zTXt_chunk_data[RAW_PROFILE_TYPE_EXIF.len()..])
		{
			// ...and perform PNG-specific decoding & return the result
			return Ok(decode_metadata_png(&decompressed_data).unwrap());
		}
		else
		{
			return io_error!(Other, "Could not inflate compressed chunk data!");
		}
	}

	return io_error!(Other, "No metadata found!");

}

#[allow(non_snake_case)]
pub fn
write_metadata
(
	path: &Path,
	general_encoded_metadata: &Vec<u8>
)
-> Result<(), std::io::Error>
{

	let encoded_metadata = encode_metadata_png(general_encoded_metadata);

	// First clear the existing metadata
	// This also parses the PNG and checks its validity, so it is safe to
	// assume that is, in fact, a usable PNG file
	if let Err(error) = clear_metadata(path)
	{
		return Err(error);
	}

	let mut IHDR_length = 0u32;
	if let Ok(chunks) = parse_png(path)
	{
		IHDR_length = chunks[0].length();
	}

	let mut file = OpenOptions::new()
		.write(true)
		.read(true)
		.open(path)
		.expect("Could not open file");

	let seek_start = 0u64			// Skip ...
	+ PNG_SIGNATURE.len()	as u64	//	PNG Signature
	+ IHDR_length			as u64	//	IHDR data section
	+ 12					as u64;	//	rest of IHDR chunk (length, type, CRC)

	// Get to first chunk after IHDR, copy all the data starting from there
	let mut buffer = Vec::new();
	perform_file_action!(file.seek(SeekFrom::Start(seek_start)));
	perform_file_action!(file.read_to_end(&mut buffer));
	perform_file_action!(file.seek(SeekFrom::Start(seek_start)));

	// Build data of new chunk using zlib compression (level=8 -> default)
	let mut zTXt_chunk_data: Vec<u8> = vec![0x7a, 0x54, 0x58, 0x74];
	zTXt_chunk_data.extend(RAW_PROFILE_TYPE_EXIF.iter());
	zTXt_chunk_data.extend(compress_to_vec_zlib(&encoded_metadata, 8).iter());

	// Compute CRC and append it to the chunk data
	let crc_struct = Crc::<u32>::new(&CRC_32_ISO_HDLC);
	let checksum = crc_struct.checksum(&zTXt_chunk_data) as u32;
	for i in 0..4
	{
		zTXt_chunk_data.push( (checksum >> (8 * (3-i))) as u8);		
	}

	// Write new data to PNG file
	// Start with length of the new chunk (subtracting 8 for type and CRC)
	let chunk_data_len = zTXt_chunk_data.len() as u32 - 8;
	for i in 0..4
	{
		perform_file_action!(file.write( &[(chunk_data_len >> (8 * (3-i))) as u8] ));
	}

	// Write data of new chunk and rest of PNG file
	perform_file_action!(file.write_all(&zTXt_chunk_data));
	perform_file_action!(file.write_all(&buffer));

	return Ok(());
}