// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub mod chunk;
mod read;
mod text;

use std::collections::VecDeque;
use std::fs::File;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Write;
use std::ops::Neg;
use std::path::Path;

use crc::Crc;
use crc::CRC_32_ISO_HDLC;
use log::warn;
use miniz_oxide::deflate::compress_to_vec_zlib;
use text::construct_similar_with_new_data;
use text::get_data_from_text_chunk;

use crate::general_file_io::io_error;
use crate::general_file_io::open_read_file;
use crate::general_file_io::EXIF_HEADER;
use crate::general_file_io::LITTLE_ENDIAN_INFO;
use crate::general_file_io::BIG_ENDIAN_INFO;
use crate::general_file_io::NEWLINE;
use crate::general_file_io::SPACE;

use crate::metadata::Metadata;

use crate::png::chunk::PngChunk;
use crate::png::read::read_chunk_length;
use crate::png::read::read_chunk_name;
use crate::png::read::read_chunk_data;
use crate::png::read::read_chunk_crc;
use crate::png::text::get_keyword_from_text_chunk;

use crate::xmp::remove_exif_from_xmp;
use crate::util::range_remove;

pub(crate) const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
pub(crate) const RAW_PROFILE_TYPE_EXIF: [u8; 21] = [
	0x52, 0x61, 0x77, 0x20,                             // Raw
	0x70, 0x72, 0x6F, 0x66, 0x69, 0x6C, 0x65, 0x20,     // profile
	0x74, 0x79, 0x70, 0x65, 0x20,                       // type
	0x65, 0x78, 0x69, 0x66,                             // exif
];

pub(crate) const XML_COM_ADOBE_XMP: [u8; 17] = [
	0x58, 0x4d, 0x4c, 0x3a,                 // XML:
	0x63, 0x6f, 0x6d, 0x2e,                 // com.
	0x61, 0x64, 0x6f, 0x62, 0x65, 0x2e,     // adobe.
	0x78, 0x6d, 0x70,                       // xmp
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
check_signature
(
	file_buffer: &Vec<u8>
)
-> Result<Cursor<&Vec<u8>>, std::io::Error>
{	
	// Check the signature
	let signature_is_valid = file_buffer[0..8].iter()
		.zip(PNG_SIGNATURE.iter())
		.filter(|&(read, constant)| read == constant)
		.count() == PNG_SIGNATURE.len();

	if !signature_is_valid
	{
		return io_error!(InvalidData, "Can't open PNG file - Wrong signature!");
	}

	// Signature is valid - can proceed using the data as PNG file
	let mut cursor = Cursor::new(file_buffer);
	cursor.set_position(8);
	return Ok(cursor);
}

fn
file_check_signature
(
	path: &Path
)
-> Result<File, std::io::Error>
{
	let mut file = open_read_file(path)?;
	
	// Check the signature
	let mut signature_buffer = [0u8; 8];
	file.read(&mut signature_buffer)?;
	check_signature(&signature_buffer.to_vec())?;

	// Signature is valid - can proceed using the file as PNG file
	return Ok(file);
}




/// "Parses" the PNG by checking various properties:
/// - Can the file be opened and is the signature valid?
/// - Are the various chunks OK or not? For this, the local subroutine `get_next_chunk_descriptor` is used
pub(crate) fn
vec_parse_png
(
	file_buffer: &Vec<u8>
)
-> Result<Vec<PngChunk>, std::io::Error>
{
	let mut cursor = check_signature(file_buffer)?;
	return generic_parse_png(&mut cursor);
}

/// "Parses" the PNG by checking various properties:
/// - Can the file be opened and is the signature valid?
/// - Are the various chunks OK or not? For this, the local subroutine `get_next_chunk_descriptor` is used
pub(crate) fn
file_parse_png
(
	path: &Path
)
-> Result<Vec<PngChunk>, std::io::Error>
{
	let mut file = file_check_signature(path)?;
	return generic_parse_png(&mut file);
}

fn
generic_parse_png
<T: Seek + Read>
(
	cursor: &mut T
)
-> Result<Vec<PngChunk>, std::io::Error>
{
	let mut chunks = Vec::new();

	loop
	{
		let chunk_descriptor = get_next_chunk_descriptor(cursor)?;
		chunks.push(chunk_descriptor);

		if chunks.last().unwrap().as_string() == "IEND".to_string()
		{
			break;
		}
	}

	return Ok(chunks);
}




// TODO: Check if this is also affected by endianness
// Edit: Should... not? I guess?
fn
get_next_chunk_descriptor
<T: Seek + Read>
(
	cursor: &mut T
)
-> Result<PngChunk, std::io::Error>
{
	// Read the start of the chunk, its data and CRC
	let chunk_length = read_chunk_length(cursor)?;
	let chunk_name   = read_chunk_name(cursor)?;
	let chunk_data   = read_chunk_data(cursor, chunk_length as usize)?;
	let chunk_crc    = read_chunk_crc(cursor)?;

	// Compute CRC on chunk
	let mut crc_input = Vec::new();
	crc_input.extend(chunk_name.bytes().into_iter());
	crc_input.extend(chunk_data.iter());

	let crc_struct = Crc::<u32>::new(&CRC_32_ISO_HDLC);
	let checksum = crc_struct.checksum(&crc_input) as u32;

	for i in 0..4
	{
		if ((checksum >> (8 * (3-i))) as u8) != chunk_crc[i]
		{
			return io_error!(InvalidData, "Checksum check failed while reading PNG!");
		}
	}

	// If validating the chunk using the CRC was successful, return its descriptor
	// Note: chunk_length does NOT include the +4 for the CRC area!
	let png_chunk_result = PngChunk::from_string(
		&chunk_name.clone(),
		chunk_length
	);
	if let Ok(png_chunk) = png_chunk_result
	{
		return Ok(png_chunk);
	}
	else
	{
		warn!("Unknown PNG chunk name: {}", chunk_name);
		return Ok(png_chunk_result.err().unwrap());
	}
}




pub(crate) fn
read_metadata
(
	file_buffer: &Vec<u8>
)
-> Result<Vec<u8>, std::io::Error>
{
	// Parse the PNG - if this fails, the read fails as well
	let parse_png_result = vec_parse_png(file_buffer)?;

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut cursor = check_signature(file_buffer).unwrap();

	return generic_read_metadata(&mut cursor, &parse_png_result);
}

pub(crate) fn
file_read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	// Parse the PNG - if this fails, the read fails as well
	let parse_png_result = file_parse_png(path)?;

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut file = file_check_signature(path).unwrap();

	return generic_read_metadata(&mut file, &parse_png_result);
}

#[allow(non_snake_case)]
fn
generic_read_metadata
<T: Seek + Read>
(
	cursor:     &mut T,
	parsed_png: &Vec<PngChunk>
)
-> Result<Vec<u8>, std::io::Error>
{
	for chunk in parsed_png
	{

		match chunk.as_string().as_str()
		{
			"eXIf" => {
				// Can be directly decoded

				// Skip chunk length and type (4+4 Bytes)
				cursor.seek(std::io::SeekFrom::Current(4+4))?;

				// Read chunk data into buffer
				// No need to verify this using CRC as already done by parse_png(path)
				let eXIf_chunk_data = read_chunk_data(
					cursor, 
					chunk.length() as usize
				)?;
				
				return Ok(eXIf_chunk_data);
			},
			
			"tEXt" | "zTXt" | "iTXt" => {
				// More common & expected case

				// Skip chunk length and type (4+4 Bytes)
				cursor.seek(std::io::SeekFrom::Current(4))?;

				let chunk_name = read_chunk_name(cursor)?;

				// Read chunk data into buffer
				// No need to verify this using CRC as already done by 
				// previously calling parse_png(path)
				let chunk_data = read_chunk_data(
					cursor, 
					chunk.length() as usize
				)?;

				// Check that this chunk contains raw profile EXIF data
				let keyword = get_keyword_from_text_chunk(&chunk_data);
				let mut has_raw_profile_type_exif = false;
				if keyword.len() == RAW_PROFILE_TYPE_EXIF.len()
				{
					has_raw_profile_type_exif = keyword
						.bytes()
						.zip(RAW_PROFILE_TYPE_EXIF.iter())
						.all(|(a,b)| a == *b);
				}

				if !has_raw_profile_type_exif
				{
					// Skip CRC from current (wrong) chunk and continue
					cursor.seek(std::io::SeekFrom::Current(4))?;
					continue;
				}

				let decompressed_data = get_data_from_text_chunk(
					chunk_name.as_str(), 
					&chunk_data
				)?;
				
				return Ok(decode_metadata_png(&decompressed_data).unwrap());
			}

			_ => {
				cursor.seek(std::io::SeekFrom::Current(chunk.length() as i64 + 12))?;
				continue;
			}
		};
	}

	return io_error!(Other, "No metadata found!");

}




// Clears existing metadata chunk from a png file
// Gets called before writing any new metadata
#[allow(non_snake_case)]
pub(crate) fn
file_clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{
	// Load the entire file into memory instead of reading one byte at a time
	// to improve the overall speed
	let mut file_buffer: Vec<u8> = std::fs::read(path)?;

	// Clear the metadata via the buffer based function
	clear_metadata(&mut file_buffer)?;

	// Write the file
	// Possible to optimize further by returning the purged bytestream itself?
	let mut file = std::fs::OpenOptions::new()
		.write(true)
		.truncate(true)
		.open(path)?;
	file.write_all(&file_buffer)?;

	return Ok(());
}

// Clears existing metadata chunk from a png file
// Gets called before writing any new metadata
#[allow(non_snake_case)]
pub(crate) fn
clear_metadata
(
	file_buffer: &mut Vec<u8>
)
-> Result<(), std::io::Error>
{
	// Parse the PNG - if this fails, the clear operation fails as well
	let parse_png_result = vec_parse_png(&file_buffer)?;

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut cursor = Cursor::new(file_buffer);

	// Skip the PNG file header (8 bytes)
	let mut remove_start;
	cursor.seek(std::io::SeekFrom::Current(8))?;

	for chunk in &parse_png_result
	{
		// Where the chunk that we might want to remove starts
		remove_start = cursor.stream_position()? as usize;

		match chunk.as_string().as_str()
		{
			"eXIf" => {
				// Remove the entire chunk (done after the match)
			},

			"iTXt" | "zTXt" | "tEXt" => {

				// Skip chunk length and type (4+4 Bytes)
				cursor.seek(std::io::SeekFrom::Current(4+4))?;

				// Read chunk data into buffer for checking that this is the
				// correct chunk to delete
				let chunk_data = read_chunk_data(
					&mut cursor, 
					chunk.length() as usize
				)?;

				let keyword = get_keyword_from_text_chunk(&chunk_data);

				// Compare to the "Raw profile type exif" string constant
				let mut has_raw_profile_type_exif = false;
				if keyword.len() == RAW_PROFILE_TYPE_EXIF.len()
				{
					has_raw_profile_type_exif = keyword
						.bytes()
						.zip(RAW_PROFILE_TYPE_EXIF.iter())
						.all(|(a,b)| a == *b);
				}

				// Compare to the "XML:com.adobe.xmp" string constant
				let mut has_xml_com_adobe_xmp = false;
				if keyword.len() == XML_COM_ADOBE_XMP.len()
				{
					has_xml_com_adobe_xmp = keyword
						.bytes()
						.zip(XML_COM_ADOBE_XMP.iter())
						.all(|(a,b)| a == *b);
				}

				if has_xml_com_adobe_xmp
				{
					// Don't fully remove the chunk, only remove EXIF from XMP
					// To do that, reposition the cursor to the start of the 
					// entire
					cursor.seek_relative((chunk.length() as i64).neg())?;
					cursor.seek_relative(-8)?;
					clear_exif_from_xmp_metadata(&mut cursor, &chunk_data)?;
					continue;
				}

				// If this is not the correct zTXt/iTXt chunk, 
				// ignore it, skip its CRC and continue with next chunk
				if !has_raw_profile_type_exif
				{
					cursor.seek_relative(4)?;
					continue;
				}
			},

			_ => {
				// In any other case, skip this chunk and continue with the 
				// next one after adjusting the cursor
				cursor.seek(std::io::SeekFrom::Current(12 + chunk.length() as i64))?;
				continue;
			}
		}

		// As we haven't continued to the next chunk in a previous match arm, 
		// we have now established that we want to remove this chunk.
		cursor.set_position(remove_start as u64);
		remove_chunk_at(&mut cursor)?;

	}

	return Ok(());
}



/// Removes the chunk that starts at the given position.
/// After that, cursor is positioned at the start of the next chunk.
fn
remove_chunk_at
(
	cursor: &mut Cursor<&mut Vec<u8>>,
)
-> Result<(), std::io::Error>
{
	let chunk_start_position = cursor.position() as usize;
	let chunk_length         = read_chunk_length(cursor)?;

	// Seek to the end of the chunk, with the 8 additional bytes due to the 
	// name and CRC fields
	cursor.seek_relative(chunk_length as i64 + 8)?;
	let chunk_end_position = cursor.position() as usize;

	range_remove(
		cursor.get_mut(), 
		chunk_start_position, 
		chunk_end_position
	);

	// Set the position of the cursor to the original start position
	cursor.set_position(chunk_start_position as u64);

	return Ok(());
}



fn
clear_exif_from_xmp_metadata
(
	cursor:     &mut Cursor<&mut Vec<u8>>,
	chunk_data: &[u8],
)
-> Result<(), std::io::Error>
{
	// Read the chunk name and seek back
	let _          = read_chunk_length(cursor)?;
	let chunk_name = read_chunk_name(cursor)?;
	cursor.seek_relative(-8)?;

	// Clear the EXIF from the XMP data
	let clean_xmp_data = remove_exif_from_xmp(
		// &chunk_data[XML_COM_ADOBE_XMP.len()..]
		&get_data_from_text_chunk(chunk_name.as_str(), &chunk_data)?
	).unwrap();

	// Construct new chunk data field
	let new_chunk_data = construct_similar_with_new_data(
		chunk_name.as_str(), 
		chunk_data, 
		&clean_xmp_data
	)?;

	// Replace chunk
	remove_chunk_at(cursor)?;
	return write_chunk(cursor, chunk_name.as_str(), &new_chunk_data);
}



pub(crate) fn
write_metadata
(
	file_buffer: &mut Vec<u8>,
	metadata:    &Metadata
)
-> Result<(), std::io::Error>
{
	// First clear the existing metadata
	// This also parses the PNG and checks its validity, so it is safe to
	// assume that is, in fact, a usable PNG file
	clear_metadata(file_buffer)?;

	// Parsed PNG is Ok to use - Create a cursor for writing
	let mut cursor = Cursor::new(file_buffer);

	// Call the generic write function
	return generic_write_metadata(&mut cursor, metadata);
}

pub(crate) fn
file_write_metadata
(
	path:     &Path,
	metadata: &Metadata
)
-> Result<(), std::io::Error>
{
	// First clear the existing metadata
	// This also parses the PNG and checks its validity, so it is safe to
	// assume that is, in fact, a usable PNG file
	// For that, load the entire file into memory
	let mut file_buffer: Vec<u8> = std::fs::read(path)?;

	// Clear old metadata and write new to buffer
	write_metadata(&mut file_buffer, metadata)?;

	// Write the file
	// Possible to optimize further by returning the purged bytestream itself?
	let mut file = std::fs::OpenOptions::new()
		.write(true)
		.truncate(true)
		.open(path)?;
	file.write_all(&file_buffer)?;

	return Ok(());
}

/// Assumes the cursor to be positioned at the insert position
#[allow(non_snake_case)]
fn
write_chunk
<T: Seek + Read + Write>
(
	cursor:     &mut T,
	chunk_name: &str,
	chunk_data: &[u8],
)
-> Result<(), std::io::Error>
{
	// Create a new vec for computing the CRC
	let mut data = chunk_name.as_bytes().to_vec();
	data.extend(chunk_data);

	// Compute CRC and append it to the data vector
	let crc_struct = Crc::<u32>::new(&CRC_32_ISO_HDLC);
	let checksum = crc_struct.checksum(&data) as u32;
	for i in 0..4
	{
		data.push( (checksum >> (8 * (3-i))) as u8);		
	}

	// Prepare writing: 
	// - Backup cursor position 
	// - Read everything from there onwards into a buffer
	// - Go back to insert position
	let     backup_cursor_position = cursor.stream_position()?;
	let mut buffer                 = Vec::new();
	cursor.read_to_end(&mut buffer)?;
	cursor.seek(SeekFrom::Start(backup_cursor_position))?;

	// Write length of the new chunk (which is 8 bytes shorter than `data`)
	let chunk_data_len = chunk_data.len() as u32;
	for i in 0..4
	{
		cursor.write(&[(chunk_data_len >> (8 * (3-i))) as u8])?;
	}

	// Write data of new chunk, remember that position, write remaining PNG
	// data and revert position so that cursor now points to the chunk right
	// after the one that has been written
	cursor.write_all(&data)?;
	let end_of_written_chunk_cursor_position = cursor.stream_position()?;
	cursor.write_all(&buffer)?;
	cursor.seek(SeekFrom::Start(end_of_written_chunk_cursor_position))?;

	return Ok(());
}

#[allow(non_snake_case)]
fn
generic_write_metadata
<T: Seek + Read + Write>
(
	cursor:     &mut T,
	metadata:   &Metadata
)
-> Result<(), std::io::Error>
{
	cursor.seek(SeekFrom::Start(8))?;

	let mut IHDR_length = 0u32;

	if let Ok(chunks) = generic_parse_png(cursor)
	{
		IHDR_length = chunks[0].length();
	}

	// Encode the data specifically for PNG and open the image file
	let encoded_metadata = encode_metadata_png(&metadata.encode()?);
	let seek_start = 0u64         // Skip ...
	+ PNG_SIGNATURE.len() as u64  // PNG Signature
	+ IHDR_length         as u64  // IHDR data section
	+ 12                  as u64; // rest of IHDR chunk (length, type, CRC)

	// Build data of new chunk using zlib compression (level=8 -> default)
	let zTXt_chunk_data: Vec<u8> = construct_zTXt_chunk_data(
		Vec::new(),
		&encoded_metadata
	);

	// Seek to insert position and write the chunk
	cursor.seek(SeekFrom::Start(seek_start))?;
	return write_chunk(cursor, "zTXt", &zTXt_chunk_data);
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

	// Now remove the first element until the exif header or endian information 
	// is found.
	// Store the popped elements to get the size information
	let mut exif_header_found = false;
	let mut endian_info_found = false;
	let mut pop_storage: Vec<u8> = Vec::new();

	while !exif_header_found && !endian_info_found
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

		counter = 0;

		// But what if the EXIF_HEADER is missing and we are directly starting
		// with the endian information? See issue #54
		for endian_info in &LITTLE_ENDIAN_INFO
		{
			if *endian_info != exif_all[counter]
			{
				break;
			}
			counter += 1;
		}

		endian_info_found = counter == LITTLE_ENDIAN_INFO.len();

		if endian_info_found
		{
			break;
		}

		// And the same check for big endian
		for endian_info in &BIG_ENDIAN_INFO
		{
			if *endian_info != exif_all[counter]
			{
				break;
			}
			counter += 1;
		}

		endian_info_found = counter == BIG_ENDIAN_INFO.len();

		if endian_info_found
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

/// Provides the PNG specific encoding result as vector of bytes to be used
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

	return construct_zTXt_chunk_data(
		vec![0x7a, 0x54, 0x58, 0x74], 
		&basic_png_encode_result
	);
}



#[allow(non_snake_case)]
fn
construct_zTXt_chunk_data
(
	prefix:                   Vec<u8>,
	basic_png_encode_result: &Vec<u8>
)
-> Vec<u8>
{
	// For further information on this see paragraph 11.3.3.3 of the this
	// document: https://www.w3.org/TR/png/#11zTXt

	// Build data of new chunk using zlib compression (level=8 -> default)
	let mut zTXt_chunk_data: Vec<u8> = Vec::new();

	// Optional prefix, needed by the `as_u8_vec` function
	zTXt_chunk_data.extend(prefix.iter());

	// Exif Keyword
	zTXt_chunk_data.extend(RAW_PROFILE_TYPE_EXIF.iter());

	// Null separator that signals the end of the keyword
	zTXt_chunk_data.push(0x00);

	// The compression method for the zTXt chunk, with 0 telling a reader to
	// use the standard deflate compression
	zTXt_chunk_data.push(0x00);

	// The actual data bytes, compressed using the deflate method
	zTXt_chunk_data.extend(compress_to_vec_zlib(basic_png_encode_result, 8).iter());

	return zTXt_chunk_data;
}



#[cfg(test)]
mod tests 
{

	#[test]
	fn
	parsing_test() 
	{
		let chunks = crate::png::file_parse_png(
			std::path::Path::new("tests/png_parse_test_image.png")
		).unwrap();
		assert_eq!(chunks.len(), 3);
	}
	
}
