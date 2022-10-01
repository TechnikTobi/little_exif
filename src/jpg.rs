use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;

use crc::{Crc, CRC_32_ISO_HDLC};

use crate::endian::*;
use crate::general_file_io::*;

pub const JPG_SIGNATURE: [u8; 2] = [0xff, 0xd8];
// const JPG_APP1_MARKER: [u8; 2] = [0xff, 0xe1];
const JPG_APP1_MARKER: u16 = 0xffe1;

struct
JPGsegment
{
	marker: u16,
	length: u16,
	data: Vec<u8>
}

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
	jpg_exif.extend(to_u8_vec_macro!(u16, &JPG_APP1_MARKER, &Endian::Big));
	jpg_exif.extend(to_u8_vec_macro!(u16, &length, &Endian::Big));
	jpg_exif.extend(exif_vec.iter());

	return jpg_exif;
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
		return io_error!(NotFound, "Can't parse JPG file - File does not exist!");
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
		return io_error!(InvalidData, "Can't open JPG file - Wrong signature!");
	}

	// Signature is valid - can proceed using the file as JPG file
	return Ok(file);
}

fn
parse_jpg
(
	path: &Path
)
-> Result<Vec<JPGsegment>, std::io::Error>
{
	let mut file_result = check_signature(path);
	let mut segments = Vec::new();

	if file_result.is_err()
	{
		return Err(file_result.err().unwrap());
	}

	let mut file = file_result.unwrap();

	loop
	{
		// First read the next marker
		let mut marker_buffer = [0u8; 2];
		perform_file_action!(file.read(&mut marker_buffer));

		// Then read the length of the segment
		let mut length_buffer = [0u8; 2];
		perform_file_action!(file.read(&mut length_buffer));

		// Decode the length to determine how much more data needs to be read
		// For now I don't know how to determine endianness of a JPG
		// But it seems like that (most) JPGs use big endian notation
		let length = from_u8_vec_macro!(u16, &length_buffer.to_vec(), &Endian::Big);
		let marker = from_u8_vec_macro!(u16, &marker_buffer.to_vec(), &Endian::Big);

		// Read remaining data
		// Note: The length includes the 2 bytes for the length field as well
		println!("{}", length);
		let mut data = vec![0u8; (length - 2) as usize];
		perform_file_action!(file.read(&mut data));

		segments.push(JPGsegment { marker, length, data } );
	}

	return Ok(segments);
}

pub fn
clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{

	let parse_jpg_result = parse_jpg(path);
	if let Err(error) = parse_jpg_result
	{
		return Err(error);
	}

	let mut file = check_signature(path).unwrap();
	let mut seek_counter = 2u64;

	for segment in &parse_jpg_result.unwrap()
	{

		let total_seg_length = segment.length + 2;

		if segment.marker != JPG_APP1_MARKER
		{
			seek_counter += total_seg_length as u64;
			perform_file_action!(file.seek(SeekFrom::Current(total_seg_length as i64)));
			continue;
		}

		// Get to the next section
		perform_file_action!(file.seek(SeekFrom::Current(total_seg_length as i64)));

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

pub fn
write_metadata
(
	path: &Path,
	encoded_metadata: &Vec<u8>
)
-> Result<(), std::io::Error>
{
	if let Err(error) = clear_metadata(path)
	{
		return Err(error);
	}

	return Ok(());
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