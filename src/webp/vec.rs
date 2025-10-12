// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

use crate::general_file_io::EXIF_HEADER;
use crate::metadata::Metadata;
use crate::util::insert_multiple_at;
use crate::util::range_remove;

use super::riff_chunk::RiffChunk;
use super::riff_chunk::RiffChunkDescriptor;
use super::*;

/// A WebP file starts as follows
/// - The RIFF signature: ASCII characters "R", "I", "F", "F"  -> 4 bytes
/// - The file size starting at offset 8                       -> 4 bytes
/// - The WEBP signature: ASCII characters "W", "E", "B", "P"  -> 4 bytes
/// This individually checks these bytes using the dedicated functions
fn
check_signature
(
	file_buffer: &Vec<u8>
)
-> Result<Cursor<&Vec<u8>>, std::io::Error>
{
	check_riff_signature(file_buffer      )?;
	check_byte_count(    file_buffer, None)?;
	check_webp_signature(file_buffer      )?;

	let mut cursor = Cursor::new(file_buffer);
	cursor.set_position(12);

	return Ok(cursor);
}



/// Gets the next RIFF chunk, starting at the current file buffer cursor
/// Advances the cursor to the start of the next chunk
fn
get_next_chunk
(
	cursor: &mut Cursor<&Vec<u8>>
)
-> Result<RiffChunk, std::io::Error>
{
	// Read the start of the chunk
	let mut chunk_start = [0u8; 8];

	
	let mut bytes_read = cursor.read(&mut chunk_start).unwrap();

	// Check that indeed 8 bytes were read
	if bytes_read != 8
	{
		return io_error!(UnexpectedEof, "Could not read start of chunk");
	}

	// Construct name of chunk and its length
	let chunk_name = String::from_utf8(chunk_start[0..4].to_vec());
	let mut chunk_length = from_u8_vec_macro!(u32, &chunk_start[4..8].to_vec(), &Endian::Little);

	// Account for the possible padding byte
	chunk_length += chunk_length % 2;

	// Read RIFF chunk data
	let mut chunk_data_buffer = vec![0u8; chunk_length as usize];
	bytes_read = cursor.read(&mut chunk_data_buffer).unwrap();
	if bytes_read != chunk_length as usize
	{
		return io_error!(
			Other, 
			format!("Could not read RIFF chunk data! Expected {chunk_length} bytes but read {bytes_read}")
		);
	}

	if let Ok(parsed_chunk_name) = chunk_name
	{
		return Ok(RiffChunk::new(
			parsed_chunk_name as String, 
			chunk_length      as usize,
			chunk_data_buffer as Vec<u8>
		));
	}
	else
	{
		return io_error!(Other, "Could not parse RIFF fourCC chunk name!");
	}
}



/// Gets a descriptor of the next RIFF chunk, starting at the current buffer
/// cursor position. Advances the cursor to the start of the next chunk
/// Relies on `get_next_chunk` by basically calling that function and throwing
/// away the actual payload
fn
get_next_chunk_descriptor
(
	cursor: &mut Cursor<&Vec<u8>>
)
-> Result<RiffChunkDescriptor, std::io::Error>
{
	let next_chunk_result = get_next_chunk(cursor)?;
	return Ok(next_chunk_result.descriptor());
}



/// "Parses" the WebP file by checking various properties:
/// - Can the file be opened and is the signature valid, including the file size?
/// - Are the chunks and their size descriptions OK? Relies on the local subroutine `get_next_chunk_descriptor`
pub(crate) fn
parse_webp
(
	file_buffer: &Vec<u8>
)
-> Result<Vec<RiffChunkDescriptor>, std::io::Error>
{
	let mut cursor = check_signature(file_buffer)?;

	let mut chunks = Vec::new();

	// The amount of data we expect to read while parsing the chunks
	let expected_length = file_buffer.len();

	// How much data we have parsed so far.
	// Starts with 12 bytes: 
	// - 4 bytes for RIFF signature
	// - 4 bytes for file size
	// - 4 bytes for WEBP signature
	// These bytes are already read in by the `check_signature` subroutine
	let mut parsed_length = 12;

	loop
	{
		let next_chunk_descriptor_result = get_next_chunk_descriptor(&mut cursor);
		if let Ok(chunk_descriptor) = next_chunk_descriptor_result
		{
			// The parsed length increases by the length of the chunk's 
			// header (4 byte) + it's size section (4 byte) and the payload
			// size, which is noted by the aforementioned size section
			parsed_length += 4 + 4 + chunk_descriptor.len();

			// Add the chunk descriptor
			chunks.push(chunk_descriptor);
			
			if parsed_length == expected_length
			{
				// In this case we don't expect any more data to be in the file
				break;
			}			
		}
		else
		{
			// This is the case when the read of the next chunk descriptor 
			// fails due to not being able to fetch 8 bytes for the header and
			// chunk size information, indicating that there is no further data
			// in the file and we are done with parsing.
			// If the subroutine fails due to other reasons, the error gets
			// propagated further.
			if next_chunk_descriptor_result.as_ref().err().unwrap().kind() == std::io::ErrorKind::UnexpectedEof
			{
				break;
			}
			else
			{
				return Err(next_chunk_descriptor_result.err().unwrap());
			}
		}
	}

	return Ok(chunks);
}



fn
check_exif_in_file
(
	file_buffer: &Vec<u8>
)
-> Result<(Cursor<&Vec<u8>>, Vec<RiffChunkDescriptor>), std::io::Error>
{
	// Parse the WebP file - if this fails, we surely can't read any metadata
	let parsed_webp_result = parse_webp(file_buffer);
	if let Err(error) = parsed_webp_result
	{
		return Err(error);
	}

	// Next, check if this is an Extended File Format WebP file
	// In this case, the first Chunk SHOULD have the type "VP8X"
	// Otherwise, the file is either invalid ("VP8X" at wrong location) or a 
	// Simple File Format WebP file which don't contain any EXIF metadata.
	if let Some(first_chunk) = parsed_webp_result.as_ref().unwrap().first()
	{
		// Compare the chunk descriptor header.
		if first_chunk.header().to_lowercase() != VP8X_HEADER.to_lowercase()
		{
			return io_error!(
				Other, 
				format!("Expected first chunk of WebP file to be of type 'VP8X' but instead got {}!", first_chunk.header())
			);
		}
	}
	else
	{
		return io_error!(Other, "Could not read first chunk descriptor of WebP file!");
	}

	// Finally, check the flag by opening up the file and reading the data of
	// the VP8X chunk
	// Regarding the seek:
	// - RIFF + file size + WEBP -> 12 byte
	// - VP8X header             ->  4 byte
	// - VP8X chunk size         ->  4 byte
	let mut cursor = check_signature(file_buffer).unwrap();
	let mut flag_buffer = vec![0u8; 4usize];
	cursor.set_position(12u64 + 4u64 + 4u64);
	if cursor.read(&mut flag_buffer).unwrap() != 4
	{
		return io_error!(Other, "Could not read flags of VP8X chunk!");
	}

	// Check the 5th bit of the 32 bit flag_buffer. 
	// For further details see the Extended File Format section at
	// https://developers.google.com/speed/webp/docs/riff_container#extended_file_format
	if flag_buffer[0] & 0x08 != 0x08
	{
		return io_error!(Other, "No EXIF chunk according to VP8X flags!");
	}

	return Ok((cursor, parsed_webp_result.unwrap()));
}



/// Reads the raw EXIF data from the WebP file. Note that if the file contains
/// multiple such chunks, the first one is returned and the others get ignored.
pub(crate) fn
read_metadata
(
	file_buffer: &Vec<u8>
)
-> Result<Vec<u8>, std::io::Error>
{
	// Check the signature, parse it, check that it has a VP8X chunk and the
	// EXIF flag is set there
	let (mut cursor, parse_webp_result) = check_exif_in_file(file_buffer).unwrap();

	// At this point we have established that the file has to contain an EXIF
	// chunk at some point. So, now we need to find & return it
	// Start by seeking to the start of the first chunk and visiting chunk after
	// chunk via checking the type and seeking again to the next chunk via the
	// size information
	cursor.set_position(12u64);
	let mut header_buffer = vec![0u8; 4usize];
	let mut chunk_index = 0usize;
	loop
	{
		// Read the chunk type into the buffer
		if cursor.read(&mut header_buffer).unwrap() != 4
		{
			return io_error!(Other, "Could not read chunk type while traversing WebP file!");
		}
		let chunk_type = String::from_u8_vec(&header_buffer.to_vec(), &Endian::Little);

		// Check that this is still the type that we expect from the previous
		// parsing over the file
		// TODO: Maybe remove this part?
		let expected_chunk_type = parse_webp_result.iter().nth(chunk_index).unwrap().header();
		if chunk_type != expected_chunk_type
		{
			return io_error!(
				Other, 
				format!("Got unexpected chunk type! Expected {} but got {}", expected_chunk_type, chunk_type)
			);
		}

		// Get the size of this chunk from the previous parsing process and skip
		// the 4 bytes regarding the size
		let chunk_size = parse_webp_result.iter().nth(chunk_index).unwrap().len();
		cursor.seek(std::io::SeekFrom::Current(4))?;

		if chunk_type.to_lowercase() == EXIF_CHUNK_HEADER.to_lowercase()
		{
			// Read the EXIF chunk's data into a buffer
			let mut payload_buffer = vec![0u8; chunk_size];
			cursor.read(&mut payload_buffer)?;

			// Add the 6 bytes of the EXIF_HEADER as Prefix for the generic EXIF
			// data parser that is called on the result of this read function
			// Otherwise the result would directly start with the Endianness
			// information, leading to a failed EXIF header signature check in 
			// the function `decode_metadata_general`
			let mut raw_exif_data = EXIF_HEADER.to_vec();
			raw_exif_data.append(&mut payload_buffer);

			return Ok(raw_exif_data);
		}
		else
		{
			// Skip the entire chunk
			cursor.seek(std::io::SeekFrom::Current(chunk_size as i64))?;

			// Note that we have to seek another byte in case the chunk is of 
			// uneven size to account for the padding byte that must be included
			cursor.seek(std::io::SeekFrom::Current(chunk_size as i64 % 2))?;
		}

		// Update for next loop iteration
		chunk_index += 1;
	}
}



fn
update_file_size_information
(
	cursor: &mut Cursor<&mut Vec<u8>>,
	delta:  i32
)
-> Result<(), std::io::Error>
{
	// Note from the documentation:
	// As the size of any chunk is even, the size given by the RIFF header is also even.

	// Update the file size information, first by reading in the current value...
	let file_size_buffer = cursor.get_ref()[4..8].to_vec();

	// ...converting it to u32 representation...
	let old_file_size = from_u8_vec_macro!(u32, &file_size_buffer, &Endian::Little);

	// ...adding the delta byte count (and performing some checks)...
	if delta < 0
	{
		assert!(old_file_size as i32 > delta);
	}
	let new_file_size = (old_file_size as i32 + delta) as u32;

	assert!(old_file_size % 2 == 0);
	assert!(new_file_size % 2 == 0);

	// ...and writing back to file...
	cursor.set_position(4);
	cursor.write_all(&to_u8_vec_macro!(u32, &new_file_size, &Endian::Little))?;

	Ok(())
}



fn
convert_to_extended_format
(
	cursor: &mut Cursor<&mut Vec<u8>>
)
-> Result<(), std::io::Error>
{
	// Start by getting the first chunk of the WebP file
	let mut read_cursor = Cursor::new(cursor.get_ref().as_ref());
	read_cursor.set_position(12);
	let first_chunk_result = get_next_chunk(&mut read_cursor);

	// Check that this get operation was successful
	if first_chunk_result.is_err()
	{
		return Err(first_chunk_result.err().unwrap());
	}

	let first_chunk = first_chunk_result.unwrap();

	// Find out what simple type of WebP file we are dealing with
	let (width, height) = match first_chunk.descriptor().header().as_str()
	{
		"VP8 " 
			=> get_dimension_info_from_vp8_chunk(first_chunk.payload()),
		"VP8L"
			=> get_dimension_info_from_vp8l_chunk(first_chunk.payload()),
		_ 
			=> io_error!(Other, format!("Expected either 'VP8 ' or 'VP8L' chunk for conversion but got {:?}!", first_chunk.descriptor().header().as_str()))
	}?;

	let width_vec  = to_u8_vec_macro!(u32, &width,  &Endian::Little);
	let height_vec = to_u8_vec_macro!(u32, &height, &Endian::Little);

	let mut vp8x_chunk = vec![
		0x56, 0x50, 0x38, 0x58, // ASCII chars "V", "P", "8", "X"                  -> 4 byte
		0x0A, 0x00, 0x00, 0x00, // size of this chunk (32 + 24 + 24 bit = 10 byte) -> 4 byte
		0x00, 0x00, 0x00, 0x00, // Flags and reserved area                         -> 4 byte
	];

	// Add the two 24 bits for width and height information
	for i in 0..3 { vp8x_chunk.push(width_vec[i]); }
	for i in 0..3 { vp8x_chunk.push(height_vec[i]); }

	// Write the VP8X chunk
	insert_multiple_at(cursor.get_mut(), 12, &mut vp8x_chunk);

	// Finally, update the file size information
	update_file_size_information(cursor, 18)?;

	Ok(())
}



fn
get_dimension_info_from_vp8l_chunk
(
	payload: &Vec<u8>
)
-> Result<(u32, u32), std::io::Error>
{
	// Get the 4 bytes containing the dimension information
	// (although we only need 28 bits)
	// Starting at byte 1 instead of 0 due to the 0x2F byte
	// See: https://developers.google.com/speed/webp/docs/webp_lossless_bitstream_specification#3_riff_header
	let width_height_info_buffer = payload[1..5].to_vec();
	
	// Convert to a single u32 number for bit-mask operations
	let width_height_info = from_u8_vec_macro!(u32, &width_height_info_buffer, &Endian::Little);
	
	let mut width  = 0;
	let mut height = 0;

	// Get the first 14 bit to construct the width
	for bit_index in 0..14
	{
		width  |= ((width_height_info >> (27 - bit_index)) & 0x01) << (13 - (bit_index % 14));
	}

	// Get the next 14 bit to construct the height
	for bit_index in 14..28
	{
		height |= ((width_height_info >> (27 - bit_index)) & 0x01) << (13 - (bit_index % 14));
	}

	return Ok((width, height));
}



fn
set_exif_flag
(
	cursor: &mut Cursor<&mut Vec<u8>>,
	exif_flag_value: bool
)
-> Result<(), std::io::Error>
{
	// Parse the WebP file - if this fails, we surely can't read any metadata
	let parsed_webp_result = parse_webp(cursor.get_ref())?;

	// Next, check if this is an Extended File Format WebP file
	// In this case, the first Chunk SHOULD have the type "VP8X"
	// Otherwise we have to create the VP8X chunk!
	if let Some(first_chunk) = parsed_webp_result.first()
	{
		// Compare the chunk descriptor header and call chunk creator if required
		if first_chunk.header().to_lowercase() != VP8X_HEADER.to_lowercase()
		{
			convert_to_extended_format(cursor)?;
		}
	}
	else
	{
		return io_error!(Other, "Could not read first chunk descriptor of WebP file!");
	}	

	// At this point we know that we have a VP8X chunk at the expected location
	// Mask the old flag by either or-ing with 1 at the EXIF flag position for
	// setting it to true, or and-ing with 1 everywhere but the EXIF flag pos
	// to set it to false
	cursor.get_mut()[20] = if exif_flag_value
	{
		cursor.get_ref()[20] | 0x08
	}
	else
	{
		cursor.get_ref()[20] & 0b11110111
	};

	Ok(())
}



pub(crate) fn
clear_metadata
(
	file_buffer: &mut Vec<u8>
)
-> Result<(), std::io::Error>
{
	// Check the file signature, parse it, check that it has a VP8X chunk and
	// the EXIF flag is set there
	let exif_check_result = check_exif_in_file(file_buffer);
	if exif_check_result.is_err()
	{
		match exif_check_result.as_ref().err().unwrap().to_string().as_str()
		{
			"No EXIF chunk according to VP8X flags!"
				=> return Ok(()),
			"Expected first chunk of WebP file to be of type 'VP8X' but instead got VP8L!"
				=> return Ok(()),
			"Expected first chunk of WebP file to be of type 'VP8X' but instead got VP8 !"
				=> return Ok(()),
			_
				=> return Err(exif_check_result.err().unwrap())
		}
	}

	let (_, parse_webp_result) = exif_check_result.unwrap();
	let mut cursor = Cursor::new(file_buffer);

	// Compute a delta of how much the file size information has to change
	let mut delta = 0i32;

	// Skip the WEBP signature
	cursor.set_position(4);

	for parsed_chunk in parse_webp_result
	{
		// At the start of each iteration, the file cursor is at the start of
		// the fourCC section of a chunk

		// Compute how many bytes this chunk has
		let parsed_chunk_byte_count = 
			4u64                            // fourCC section of EXIF chunk
			+ 4u64                          // size information of EXIF chunk
			+ parsed_chunk.len() as u64     // actual size of EXIF chunk data
			+ parsed_chunk.len() as u64 % 2 // accounting for possible padding byte
		;

		// Not an EXIF chunk, seek to next one and continue
		if parsed_chunk.header().to_lowercase() != EXIF_CHUNK_HEADER.to_lowercase()
		{
			cursor.seek(std::io::SeekFrom::Current(parsed_chunk_byte_count as i64))?;
			continue;
		}

		// Remove the range containing the EXIF chunk
		let remove_start = cursor.position() as usize;
		let remove_end   = remove_start + parsed_chunk_byte_count as usize;
		range_remove(cursor.get_mut(), remove_start, remove_end);

		// Additionally, update the size information that gets written to the 
		// file header after this loop
		delta -= parsed_chunk_byte_count as i32;
	}

	// Update file size information
	update_file_size_information(&mut cursor, delta)?;
	
	// Set the flags in the VP8X chunk. First, read in the current flags
	set_exif_flag(&mut cursor, false)?;

	return Ok(());
}



/// Writes the given generally encoded metadata to the WebP image file at 
/// the specified path. 
/// Note that *all* previously stored EXIF metadata gets removed first before
/// writing the "new" metadata. 
pub(crate) fn
write_metadata
(
	file_buffer: &mut Vec<u8>,
	metadata:    &Metadata
)
-> Result<(), std::io::Error>
{
	// Clear the metadata from the file and return if this results in an error
	clear_metadata(file_buffer)?;

	// Encode the general metadata format to WebP specifications
	let mut encoded_metadata = encode_metadata_webp(&metadata.encode()?);
	let encoded_metadata_len = encoded_metadata.len() as i32;

	// Find a location where to put the EXIF chunk
	// This is done by requesting a chunk descriptor as long as we find a chunk
	// that is both known and should be located *before* the EXIF chunk
	let pre_exif_chunks = [
		"VP8X",
		"VP8",
		"VP8L",
		"ICCP",
		"ANIM"
	];

	let mut read_cursor = Cursor::new(file_buffer.as_ref());

	loop
	{
		// Request a chunk descriptor. If this fails, check the error 
		// Depending on its type, either continue normally or return it
		let chunk_descriptor_result = get_next_chunk_descriptor(&mut read_cursor);

		if let Ok(chunk_descriptor) = chunk_descriptor_result
		{
			let mut chunk_type_found_in_pre_exif_chunks = false;

			// Check header of chunk descriptor against any of the known chunks
			// that should come before the EXIF chunk
			for pre_exif_chunk in &pre_exif_chunks
			{
				chunk_type_found_in_pre_exif_chunks |= pre_exif_chunk.to_lowercase() == chunk_descriptor.header().to_lowercase();
			}

			if !chunk_type_found_in_pre_exif_chunks
			{
				break;
			}
		}
		else
		{
			match chunk_descriptor_result.as_ref().err().unwrap().kind()
			{
				std::io::ErrorKind::UnexpectedEof
					=> break, // No further chunks, place EXIF chunk here
				_
					=> return Err(chunk_descriptor_result.err().unwrap())
			}
		}
	}

	// Write the EXIF chunk at the found location
	insert_multiple_at(file_buffer, read_cursor.position() as usize, &mut encoded_metadata);

	// Update the file size information by adding the byte count of the EXIF chunk
	// (Note: Due to  the WebP specific encoding function, this vector already
	// contains the EXIF header characters and size information, as well as the
	// possible padding byte. Therefore, simply taking the length of this
	// vector takes their byte count also into account and no further values
	// need to be added)
	let mut write_cursor = Cursor::new(file_buffer);
	update_file_size_information(&mut write_cursor, encoded_metadata_len)?;

	// Finally, set the EXIF flag
	set_exif_flag(&mut write_cursor, true)?;

	return Ok(());
}
