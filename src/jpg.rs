use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;

use crc::{Crc, CRC_32_ISO_HDLC};
use deflate::deflate_bytes_zlib;

pub const JPG_SIGNATURE: [u8; 2] = [0xff, 0xd8];

fn
check_signature
(
	path: &Path
)
-> Result<File, String>
{
	if !path.exists()
	{
		return Err("Can't open JPG file - File does not exist!".to_string());
	}

	let mut file = OpenOptions::new()
		.read(true)
		.open(path)
		.expect("Could not open file");
	
	// Check the signature
	let mut signature_buffer = [0u8; 2];
	file.read(&mut signature_buffer).unwrap();
	let signature_is_valid = signature_buffer.iter()
		.zip(JPG_SIGNATURE.iter())
		.filter(|&(read, constant)| read == constant)
		.count() == JPG_SIGNATURE.len();

	if !signature_is_valid
	{
		return Err("Can't open JPG file - Wrong signature!".to_string());
	}

	// Signature is valid - can proceed using the file as JPG file
	return Ok(file);
}

pub fn
clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{
	Ok(())
}

pub fn
write_metadata
(
	path: &Path,
	encoded_metadata: &Vec<u8>
)
-> Result<(), std::io::Error>
{
	Ok(())
}