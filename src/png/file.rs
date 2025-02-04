// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::path::Path;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::fs::File;

use crc::Crc;
use crc::CRC_32_ISO_HDLC;
use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec_zlib;

use crate::general_file_io::*;
use crate::metadata::Metadata;

use super::PNG_SIGNATURE;
use super::RAW_PROFILE_TYPE_EXIF;

use super::png_chunk::PngChunk;
use super::decode_metadata_png;
use super::encode_metadata_png;

fn
check_signature
(
	path: &Path
)
-> Result<File, std::io::Error>
{
	let mut file = open_read_file(path)?;
	
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
	let png_chunk_result = PngChunk::from_string(
		&chunk_name.clone().unwrap(),
		chunk_length
	);
	if let Ok(png_chunk) = png_chunk_result
	{
		return Ok(png_chunk);
	}
	else
	{
		eprintln!("Warning: Unknown PNG chunk name: {}", chunk_name.unwrap());
		return Ok(png_chunk_result.err().unwrap());
	}
}

/// "Parses" the PNG by checking various properties:
/// - Can the file be opened and is the signature valid?
/// - Are the various chunks OK or not? For this, the local subroutine `get_next_chunk_descriptor` is used
pub(crate) fn
parse_png
(
	path: &Path
)
-> Result<Vec<PngChunk>, std::io::Error>
{
	let mut file = check_signature(path)?;
	let mut chunks = Vec::new();

	loop
	{
		let chunk_descriptor = get_next_chunk_descriptor(&mut file)?;
		chunks.push(chunk_descriptor);

		if chunks.last().unwrap().as_string() == "IEND".to_string()
		{
			break;
		}
	}

	return Ok(chunks);
}

// Clears existing metadata chunk from a png file
// Gets called before writing any new metadata
#[allow(non_snake_case)]
pub(crate) fn
clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{

	// Parse the PNG - if this fails, the clear operation fails as well
	let parse_png_result = parse_png(path)?;

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut file = open_write_file(path)?;
	let mut seek_counter = 8u64;

	for chunk in &parse_png_result
	{
		match chunk.as_string().as_str()
		{
			"eXIf" => {
				todo!();
			},
			
			"zTXt" => {
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
						break;
					}
				}

				// Skip the CRC as it is not important at this point
				perform_file_action!(file.seek(SeekFrom::Current(4)));

				// If this is not the correct zTXt chunk, ignore current
				// (wrong) zTXt chunk and continue with next chunk
				if !correct_zTXt_chunk
				{	
					continue;
				}
				
				// We have now established that this is the correct chunk to 
				// delete. Therefore: Copy data from here (after CRC) onwards 
				// into a buffer...
				let mut buffer = Vec::new();
				perform_file_action!(file.read_to_end(&mut buffer));

				// ...compute the new file length while we are at it...
				let new_file_length = seek_counter + buffer.len() as u64;

				// ...go back to the chunk to be removed...
				perform_file_action!(file.seek(SeekFrom::Start(seek_counter)));

				// ...and overwrite it using the data from the buffer
				perform_file_action!(file.write_all(&buffer));
				perform_file_action!(file.seek(SeekFrom::Start(seek_counter)));		

				// Update the size of the file - otherwise there will be
				// duplicate bytes at the end!
				perform_file_action!(file.set_len(new_file_length));

			}

			_ => {
				// Jump to the next chunk
				seek_counter += chunk.length() as u64 + 12;
				perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));
				continue;
			}
		};

	}

	return Ok(());
}

#[allow(non_snake_case)]
pub(crate) fn
read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	// Parse the PNG - if this fails, the read fails as well
	let parse_png_result = parse_png(path)?;

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut file = check_signature(path).unwrap();
	for chunk in &parse_png_result
	{

		match chunk.as_string().as_str()
		{
			"eXIf" => {
				// Can be directly decoded

				// Skip chunk length and type (4+4 Bytes)
				perform_file_action!(file.seek(SeekFrom::Current(8)));

				// Read chunk data into buffer
				// No need to verify this using CRC as already done by parse_png(path)
				let mut eXIf_chunk_data = vec![0u8; chunk.length() as usize];
				if file.read(&mut eXIf_chunk_data).unwrap() != chunk.length() as usize
				{
					return io_error!(Other, "Could not read chunk data");
				}
				
				return Ok(eXIf_chunk_data);
			},
			
			"zTXt" => {
				// More common & expected case

				// Skip chunk length and type (4+4 Bytes)
				perform_file_action!(file.seek(SeekFrom::Current(8)));

				// Read chunk data into buffer
				// No need to verify this using CRC as already done by 
				// previously calling parse_png(path)
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
						break;
					}
				}

				if !correct_zTXt_chunk
				{
					// Skip CRC from current (wrong) zTXt chunk and continue
					perform_file_action!(file.seek(SeekFrom::Current(4)));
					continue;
				}

				// Decode zlib data...
				if let Ok(decompressed_data) = decompress_to_vec_zlib(
					&zTXt_chunk_data[RAW_PROFILE_TYPE_EXIF.len()..]
				)
				{
					// ...and perform PNG-specific decoding & return the result
					return Ok(decode_metadata_png(&decompressed_data).unwrap());
				}
				else
				{
					return io_error!(Other, "Could not inflate compressed chunk data!");
				}
			}

			_ => {
				perform_file_action!(file.seek(SeekFrom::Current(chunk.length() as i64 + 12)));
				continue;
			}
		};
	}

	return io_error!(Other, "No metadata found!");

}



#[allow(non_snake_case)]
pub(crate) fn
write_metadata
(
	path:     &Path,
	metadata: &Metadata
)
-> Result<(), std::io::Error>
{

	// First clear the existing metadata
	// This also parses the PNG and checks its validity, so it is safe to
	// assume that is, in fact, a usable PNG file
	let _ = clear_metadata(path)?;

	let mut IHDR_length = 0u32;
	if let Ok(chunks) = parse_png(path)
	{
		IHDR_length = chunks[0].length();
	}

	// Encode the data specifically for PNG and open the image file
	let encoded_metadata = encode_metadata_png(&metadata.encode()?);
	let seek_start = 0u64         // Skip ...
	+ PNG_SIGNATURE.len() as u64  // PNG Signature
	+ IHDR_length         as u64  // IHDR data section
	+ 12                  as u64; // rest of IHDR chunk (length, type, CRC)

	// Get to first chunk after IHDR, copy all the data starting from there
	let mut file   = open_write_file(path)?;
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

#[cfg(test)]
mod tests 
{

	#[test]
	fn
	parsing_test() 
	{
		let chunks = crate::png::file::parse_png(
			std::path::Path::new("tests/png_parse_test_image.png")
		).unwrap();
		assert_eq!(chunks.len(), 3);
	}
	
}
