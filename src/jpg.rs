use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;

use crc::{Crc, CRC_32_ISO_HDLC};

use crate::endian::*;

pub const JPG_SIGNATURE: [u8; 2] = [0xff, 0xd8];
const JPG_APP1_MARKER: [u8; 2] = [0xff, 0xe1];

fn
encode_metadata_jpg
(
	exif_vec: &Vec<u8>
)
-> Vec<u8>
{
	// vector storing the data that will be returned
	let mut jpg_exif: Vec<u8> = Vec::new();

	// Compute the length of the exif data (includes the two bytes of the
	// actual length field)
	let length = 2u16 + (exif_vec.len() as u16);

	// Start with the APP1 marker and the length of the data
	// Then copy the previously encoded EXIF data 
	jpg_exif.extend(JPG_APP1_MARKER.iter());
	jpg_exif.extend(to_u8_vec_macro!(u16, &length, &Endian::Big));
	jpg_exif.extend(exif_vec.iter());

	return jpg_exif;
}

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

pub fn
read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	Ok(Vec::new())
}