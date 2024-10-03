// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;

use crate::endian::Endian;
use crate::u8conversion::*;
use crate::general_file_io::*;
use crate::util::range_remove;

pub(crate) const JXL_SIGNATURE:      [u8; 2]  = [0xff, 0x0a];
pub(crate) const ISO_BMFF_JXL_SIGNATURE: [u8; 12] = [
	0x00, 0x00, 0x00, 0x0c,
	0x4a, 0x58, 0x4c, 0x20,
	0x0d, 0x0a, 0x87, 0x0a
];

/// Checks if the given file buffer vector starts with the necessary bytes that
/// indicate a JXL file in an ISO BMFF container
/// These containers are divided into boxes, each consisting of
/// - 4 bytes that give the box size n
/// - 4 bytes that give the box type (e.g. "jxlc" for a JXL codestream)
/// - n-8 bytes of data
/// These 12 bytes are for checking the first box that is the same for all such
/// stored JXL images
fn
starts_with_iso_bmff_signature
(
	file_buffer: &Vec<u8>
)
-> bool
{
	return file_buffer[0..12].iter()
		.zip(ISO_BMFF_JXL_SIGNATURE.iter())
		.filter(|&(read, constant)| read == constant)
		.count() == ISO_BMFF_JXL_SIGNATURE.len();
}

/// There are two types of JXL image files: One are simply a JXL codestream,
/// which start with the `JXL_SIGNATURE` bytes 0xFF0A. These can *not* store
/// any metadata.
/// The other type is contained within a ISO BMFF container and are able to 
/// include EXIF metadata. 
/// If this function returns true, the image needs to be converted first before
/// it is able to hold any metadata
fn
starts_with_jxl_signature
(
	file_buffer: &Vec<u8>
)
-> bool
{
	return file_buffer[0..2].iter()
		.zip(JXL_SIGNATURE.iter())
		.filter(|&(read, constant)| read == constant)
		.count() == JXL_SIGNATURE.len();
}

fn
check_signature
(
	file_buffer: &Vec<u8>
)
-> Result<(), std::io::Error>
{
	if starts_with_jxl_signature(file_buffer)
	{
		return io_error!(Other, "Simple JXL codestream file - No metadata!");
	}

	if !starts_with_iso_bmff_signature(file_buffer)
	{
		return io_error!(Other, "This isn't ISO BMFF JXL data!");
	}

	return Ok(());
}

fn
file_check_signature
(
	path: &Path
)
-> Result<File, std::io::Error>
{
	let mut file = open_write_file(path)?;

	let mut signature_buffer = [0u8; 12];
	file.read(&mut signature_buffer)?;
	check_signature(&signature_buffer.to_vec())?;

	return Ok(file);
}


pub(crate) fn
clear_metadata
(
	file_buffer: &mut Vec<u8>
)
-> Result<(), std::io::Error>
{
	check_signature(file_buffer)?;

	let mut position = 0;

	loop
	{
		if position >= file_buffer.len() { return Ok(()); }

		// Get the first 4 bytes at the current cursor position to determine
		// the length of the current box 
		let length_buffer = file_buffer[position..position+4].to_vec();
		let length        = from_u8_vec_macro!(u32, &length_buffer, &Endian::Big) as usize;

		// Next, read the box type
		let type_buffer = file_buffer[position+4..position+8].to_vec();

		if type_buffer.iter()
			.zip(EXIF.iter())
			.filter(|&(read, constant)| read == constant)
			.count()
			.eq(&EXIF.len())
		{
			range_remove(file_buffer, position, position+length);
		}
		else
		{
			// Not an EXIF box so skip it
			position += length;
		}
	}
}

pub(crate) fn
file_clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{
	let mut file = file_check_signature(path)?;

	let mut length_buffer = [0u8; 4];
	let mut type_buffer   = [0u8; 4];

	loop
	{
		let position        = file.stream_position()?;
		let old_file_length = file.metadata().unwrap().len();
		if position >= old_file_length { return Ok(()); }

		file.read_exact(&mut length_buffer)?;
		file.read_exact(&mut type_buffer)?;

		let length = from_u8_vec_macro!(u32, &length_buffer.to_vec(), &Endian::Big) as usize;

		if type_buffer.iter()
			.zip(EXIF.iter())
			.filter(|&(read, constant)| read == constant)
			.count()
			.eq(&EXIF.len())
		{
			// Seek past the EXIF box ...
			perform_file_action!(file.seek_relative((length-8) as i64));


			// ... copy everything from here onwards into a buffer ...
			let mut buffer = Vec::new();
			perform_file_action!(file.read_to_end(&mut buffer));

			// ... seek back to the start of the EXIF box ...
			perform_file_action!(file.seek(std::io::SeekFrom::Start(position)));

			// ... overwrite everything from here onward ...
			perform_file_action!(file.write_all(&buffer));
			perform_file_action!(file.seek(std::io::SeekFrom::Start(position)));

			// ... and finally update the file size - otherwise there will be
			// duplicate bytes at the end!
			perform_file_action!(file.set_len(old_file_length - length as u64));
		}
		else
		{
			// Not an EXIF box so skip it
			assert_eq!(position+8, file.stream_position()?);
			perform_file_action!(file.seek_relative((length-8) as i64));
		}
	}
}



/// Read 
pub(crate) fn
read_metadata
(
	file_buffer: &Vec<u8>
)
-> Result<Vec<u8>, std::io::Error>
{
	check_signature(file_buffer)?;

	let mut cursor = Cursor::new(file_buffer);

	loop
	{
		// Get the first 4 bytes at the current cursor position to determine
		// the length of the current box (and account for the 8 bytes of length
		// and box type)
		let mut length_buffer = [0u8; 4];
		cursor.read_exact(&mut length_buffer)?;
		let length = from_u8_vec_macro!(u32, &length_buffer.to_vec(), &Endian::Big) - 8;

		// Next, read the box type
		let mut type_buffer = [0u8; 4];
		cursor.read_exact(&mut type_buffer)?;

		match type_buffer
		{
			EXIF => {

				let position = cursor.position() as usize;

				// Ignore the next 4 bytes (because that's the minor version???)
				let exif_buffer = file_buffer[position+4..position + length as usize].to_vec();
				return Ok(exif_buffer);
			},
			_ => {
				// Not an EXIF box so skip it
				cursor.seek_relative(length as i64)?;
			}
		}
	}
}

pub(crate) fn
file_read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	let mut file = open_read_file(path)?;

	// Read first 12 bytes and check that we have a ISO BMFF file
	let mut first_12_bytes = [0u8; 12];
	file.read(&mut first_12_bytes).unwrap();
	check_signature(&first_12_bytes.to_vec())?;

	loop
	{
		// Get the first 4 bytes at the current cursor position to determine
		// the length of the current box (and account for the 8 bytes of length
		// and box type)
		let mut length_buffer = [0u8; 4];
		file.read_exact(&mut length_buffer)?;
		let length = from_u8_vec_macro!(u32, &length_buffer.to_vec(), &Endian::Big) - 8;

		// Next, read the box type
		let mut type_buffer = [0u8; 4];
		file.read_exact(&mut type_buffer)?;

		match type_buffer
		{
			EXIF => {

				// Skip the next 4 bytes (which contain the minor version???)
				file.seek_relative(4)?;

				// `length-4` because of the previous relative seek operation
				let mut exif_buffer = vec![0u8; (length-4) as usize];
				file.read_exact(&mut exif_buffer)?;

				return Ok(exif_buffer);
			},
			_ => {
				// Not an EXIF box so skip it
				file.seek_relative(length as i64)?;
			}
		}
	}
}