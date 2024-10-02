// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;

use crate::endian::Endian;
use crate::u8conversion::*;
use crate::general_file_io::*;

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

pub(crate) fn
read_metadata
(
	file_buffer: &Vec<u8>
)
-> Result<Vec<u8>, std::io::Error>
{
	if starts_with_jxl_signature(file_buffer)
	{
		return io_error!(Other, "Simple JXL codestream file - No metadata!");
	}

	if !starts_with_iso_bmff_signature(file_buffer)
	{
		return io_error!(Other, "This isn't JXL data!");
	}

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
				let exif_buffer = file_buffer[position..position + length as usize].to_vec();
				return Ok(exif_buffer);
			},
			_ => {
				// Not an EXIF box so skip it
				cursor.seek_relative(length as i64)?;
			}
		}
	}

	todo!()
}