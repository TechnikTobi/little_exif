// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;

use crate::endian::*;
use crate::exif_tag::ExifTag;
use crate::general_file_io::io_error;
use crate::ifd::ExifTagGroup;
use crate::ifd::ImageFileDirectory;
use crate::u8conversion::from_u8_vec_macro;
use crate::u8conversion::U8conversion;

#[derive(Clone)]
pub struct
Tiffdata
{
	endian:                 Endian,
	image_file_directories: Vec<ImageFileDirectory>
}

impl
Tiffdata
{
	// THIS FUNCTION IS ONLY TEMPORARY FOR TEST PURPOSES
	pub(crate) fn
	as_metadata_adapter
	(
		encoded_data: &Vec<u8>
	)
	-> Result<(Endian, Vec<ExifTag>), std::io::Error>
	{
		let mut cursor = Cursor::new(encoded_data);
		cursor.set_position(6);

		let (endian, dirs) = Self::generic_decode_data(&mut cursor)?;

		let mut all_tags = Vec::new();

		for dir in dirs
		{
			all_tags.extend(dir.tags);
		}

		return Ok((endian, all_tags));
	}

	pub(crate) fn
	generic_encode_data
	(
		&mut self
	)
	{
		// Start by sorting the IFDs - Sorting of tags happens later
		self.image_file_directories.sort_by(
			|a, b|
			if a.get_generic_ifd_nr() != b.get_generic_ifd_nr()
			{
				a.get_generic_ifd_nr().cmp(&b.get_generic_ifd_nr())
			}
			else
			{
				if a.get_ifd_type() == b.get_ifd_type()
				{
					panic!("Should not have two different IFDs with same group & number!");
				}
				if a.get_ifd_type() < b.get_ifd_type()
				{
					std::cmp::Ordering::Less
				}
				else
				{
					std::cmp::Ordering::Greater
				}
			}
		);
	}
 
	pub(crate) fn
	generic_decode_data
	(
		data_cursor: &mut Cursor<&Vec<u8>>
	)
	-> Result<(Endian, Vec<ImageFileDirectory>), std::io::Error>
	{
		// Get the start position
		let data_start_position = data_cursor.position();

		// Determine endian
		let mut endian_buffer = vec![0u8; 2];
		data_cursor.read_exact(&mut endian_buffer)?;

		let endian = match endian_buffer[0..2]
		{
			[0x49, 0x49] => { Endian::Little },
			[0x4d, 0x4d] => { Endian::Big },
			_            => { return io_error!(Other, "Illegal endian information!") } 
		};

		// Validate magic number
		let mut magic_number_buffer = vec![0u8; 2];
		data_cursor.read_exact(&mut magic_number_buffer)?;
		if !(
			(endian == Endian::Little && magic_number_buffer == [0x2a, 0x00]) ||
			(endian == Endian::Big    && magic_number_buffer == [0x00, 0x2a])
		)
		{
			return io_error!(Other, "Could not verify magic number!");
		}

		// Get offset to IFD0
		let mut ifd0_offset_buffer = vec![0u8; 4];
		data_cursor.read_exact(&mut ifd0_offset_buffer)?;
		let mut ifd_offset_option = Some(from_u8_vec_macro!(u32, &ifd0_offset_buffer.to_vec(), &endian));

		// Decode all the IFDs
		let mut ifds = Vec::new();
		let mut generic_ifd_nr = 0;
		loop
		{
			if let Some(ifd_offset) = ifd_offset_option
			{
				data_cursor.set_position(data_start_position);
				data_cursor.seek_relative(ifd_offset as i64)?;

				let decode_result = ImageFileDirectory::decode_ifd(
					data_cursor,
					data_start_position,
					&endian,
					&ExifTagGroup::GENERIC,
					generic_ifd_nr,
					&mut ifds
				);

				if let Ok(new_ifd_offset_option) = decode_result
				{
					ifd_offset_option = new_ifd_offset_option;
				}
				else
				{
					return Err(decode_result.err().unwrap());
				}
			}
			else
			{
				break;
			}

			generic_ifd_nr += 1;
		}



		return Ok((endian, ifds));
	}
}


#[cfg(test)]
mod tests 
{
	use std::fs::read;
	use std::io::Cursor;

use super::Tiffdata;

	#[test]
	fn
	new_test_1()
	-> Result<(), std::io::Error>
	{
		let image_data = read("tests/read_sample.tif").unwrap();

		Tiffdata::generic_decode_data(&mut Cursor::new(&image_data))?;

		Ok(())
	}
}
