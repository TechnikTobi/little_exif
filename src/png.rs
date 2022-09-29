use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;

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

fn
check_signature
(
	path: &Path
)
-> Result<File, std::io::Error>
{
	if !path.exists()
	{
		return io_error!(NotFound, "Can't parse PNG file - File does not exist!");
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
		return io_error!(InvalidData, "Can't parse PNG file - Wrong signature!");
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

// Clears existing metadata from a png file
// Gets called before writing any new metadata
pub fn
clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{
	let parse_png_result = parse_png(path);
	if let Ok(chunks) = parse_png_result
	{
		let mut file = check_signature(path).unwrap();
		let mut seek_counter = 0u64;

		for chunk in &chunks
		{
			if chunk.as_string() == String::from("zTXt")
			{
				// Get to the next chunk...
				perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));

				// Copy data from there onwards into a buffer
				let mut buffer = Vec::new();
				perform_file_action!(file.read_to_end(&mut buffer));

				// Go back to the chunk to be removed
				// And overwrite it using the data from the buffer
				perform_file_action!(file.seek(SeekFrom::Start(seek_counter)));
				perform_file_action!(file.write_all(&buffer));
				perform_file_action!(file.seek(SeekFrom::Start(seek_counter)));
			}
			else
			{
				seek_counter += chunk.length() as u64 + 12;
				perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));
			}
		}

		return Ok(());
	}
	else
	{
		return Err(parse_png_result.err().unwrap());
	}
}

#[allow(non_snake_case)]
pub fn
write_metadata
(
	path: &Path,
	encoded_metadata: &Vec<u8>
)
-> Result<(), std::io::Error>
{

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

pub fn
read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	let parse_png_result = parse_png(path);
	if let Ok(chunks) = parse_png_result
	{
		let mut file = check_signature(path).unwrap();

		for chunk in &chunks
		{
			if chunk.as_string() == String::from("zTXt")
			{

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

				// Decode zlib data and return
				return Ok(decompress_to_vec_zlib(&zTXt_chunk_data[RAW_PROFILE_TYPE_EXIF.len()..]).unwrap());
			}

			// Seek to next chunk
			perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));
			
		}

		return io_error!(Other, "No metadata found!");
	}
	else
	{
		return Err(parse_png_result.err().unwrap());
	}
}