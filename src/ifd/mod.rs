// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub mod tag_trait;

use std::io::Cursor;
use std::io::Read;

use crate::endian::*;
use crate::general_file_io::io_error;
use crate::u8conversion::from_u8_vec_macro;
use crate::u8conversion::U8conversion;

use tag_trait::Tag;

/// Useful constants for dealing with IFDs: The length of a single IFD entry is
/// equal to 12 bytes, as the entry consists of the tags hex value (2 byte), 
/// the format (2 byte), the number of components (4 byte) and the value/offset
/// section (4 byte).
/// The four zeros tell us that this is the last IFD in its sequence and there
/// is no link to another IFD
const IFD_ENTRY_LENGTH: u32     = 12;
const IFD_END_NO_LINK:  [u8; 4] = [0x00, 0x00, 0x00, 0x00];

/// The different types of Image File Directories (IFD). A generic IFD is one
/// without further specialization, like e.g. IFD0. The generic IFDs start
/// with IFD0, which is located via the offset at the start of the TIFF data. 
/// The next IFD (in this case: IFD1) is then located via the link offset at
/// the end of IFD0. The generic IFD variant comes with an ID field that 
/// indicates the position of the IFD (e.g. 0u32 for IFD0).
/// Other IFDs, like e.g. the ExifIFD, are linked via offset tags (in case of 
/// the ExifIFD offset: 0x8769) that are located in the respective generic IFD 
/// (most of them in IFD0).
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_snake_case)]
pub enum
IfdType
{
	GENERIC  {id: u32},
	EXIF,
}

pub struct
ImageFileDirectory
{
	tags:     Vec<Box<dyn Tag>>,
	ifd_type: IfdType,
}

impl
ImageFileDirectory
{
	pub(crate) fn
	generic_decode_ifd
	(
		data_cursor: &mut Cursor<Vec<u8>>,
		endian:      &Endian
	)
	-> Result<Option<()>, std::io::Error>
	{
		// Backup the entry position where this IFD started
		let data_cursor_entry_position = data_cursor.position();

		// Check if there is enough data to decode an IFD
		if (data_cursor.get_ref().len() as i64 - data_cursor_entry_position as i64) < 6i64
		{
			return Ok(None);
		}

		// The first two bytes give us the number of entries in this IFD
		let mut number_of_entries_buffer = vec![0u8; 2];
		data_cursor.read_exact(&mut number_of_entries_buffer)?;
		let number_of_entries = from_u8_vec_macro!(u16, &number_of_entries_buffer.to_vec(), endian);

		// Check that there is enough data to unpack
		if (0
			+ 2
			+ IFD_ENTRY_LENGTH as usize * number_of_entries as usize 
			+ IFD_END_NO_LINK.len()
		) <= (
			data_cursor.get_ref().len() as i64 - data_cursor_entry_position as i64
		) as usize
		{
			return io_error!(Other, "Not enough data to decode IFD!");
		}

		// loop through the entries - assumes that the value stored in
		// `number_of_entries` is correct
		for i in 0..number_of_entries
		{
			// Read the entry into a buffer
			let mut entry_buffer = vec![0u8; IFD_ENTRY_LENGTH as usize];
			data_cursor.read_exact(&mut entry_buffer)?;

			// Decode the first 8 bytes with the tag, format and component number
			let hex_tag              = from_u8_vec_macro!(u16, &entry_buffer[0..2].to_vec(), endian);
			let hex_format           = from_u8_vec_macro!(u16, &entry_buffer[2..4].to_vec(), endian);
			let hex_component_number = from_u8_vec_macro!(u32, &entry_buffer[4..8].to_vec(), endian);

			// To get the Tag we also need the current IFD 


		}
		


		todo!()
	}
}