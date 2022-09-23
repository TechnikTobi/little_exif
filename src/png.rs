use std::path::Path;
use std::io::Read;
use std::io::Seek;
use std::fs::File;
use std::fs::OpenOptions;

use crc::{Crc, CRC_32_ISO_HDLC};

use crate::png_chunk::{PngChunkOrdering, PngChunk};

pub const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

fn
check_signature
(
	path: &Path
)
-> Result<File, String>
{
	if !path.exists()
	{
		return Err("Can't parse PNG file - File does not exist!".to_string());
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
		.count() == 8;

	if !signature_is_valid
	{
		return Err("Can't parse PNG file - Wrong signature!".to_string());
	}

	// Signature is valid - can proceed using the file as PNG file
	return Ok(file);
}



// TODO: Check if this is also affected by endianness
fn
get_next_chunk_descriptor
(
	file: &mut File
)
-> Result<PngChunk, String>
{
	// Read the start of the chunk
	let mut chunk_start = [0u8; 8];
	let mut bytes_read = file.read(&mut chunk_start).unwrap();

	// Check that indeed 8 bytes were read
	if bytes_read != 8
	{
		return Err("Could not read start of chunk".to_string());
	}

	// Construct name of chunk and its length (+4 for the CRC at the end)
	let chunk_name = String::from_utf8((&chunk_start[4..8]).to_vec());
	let mut chunk_length = 0u32;
	for byte in &chunk_start[0..4]
	{
		chunk_length = chunk_length * 256 + *byte as u32;
	}
	chunk_length += 4;

	// Read chunk data ...
	let mut chunk_data_buffer = vec![0u8; (chunk_length-4) as usize];
	bytes_read = file.read(&mut chunk_data_buffer).unwrap();
	if bytes_read != (chunk_length-4) as usize
	{
		return Err("Could not read chunk data".to_string());
	}

	// ... and CRC values
	let mut chunk_crc_buffer = [0u8; 4];
	bytes_read = file.read(&mut chunk_crc_buffer).unwrap();
	if bytes_read != 4
	{
		return Err("Could not read chunk CRC".to_string());
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
			return Err("Checksum check failed while reading PNG!".to_string());
		}
	}

	// If validating the chunk using the CRC was successful, return its descriptor
	PngChunk::from_string(
		&chunk_name.unwrap(),
		chunk_length
	)
}



pub fn
parse_png
(
	path: &Path
)
-> Result<Vec<PngChunk>, String>
{
	let mut file = check_signature(path);
	let mut chunks = Vec::new();

	if file.is_err()
	{
		return Err(file.err().unwrap());
	}

	loop
	{
		if let Ok(chunk_descriptor) = get_next_chunk_descriptor(file.as_mut().unwrap())
		{
			chunks.push(chunk_descriptor);

			if chunks.last().unwrap().as_string() == "IEND".to_string()
			{
				break;
			}
		}
		else
		{
			return Err("Could not read next chunk".to_string());
		}
	}

	file.unwrap().rewind();

	return Ok(chunks);
}

pub fn
write_metadata
(
	path: &Path,
	encoded_metadata: &Vec<u8>
)
-> Result<(), String>
{
	Ok(())
}