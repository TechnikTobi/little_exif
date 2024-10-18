// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use core::panic;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;

use crate::endian::*;
use crate::exif_tag::ExifTag;
use crate::general_file_io::io_error;
use crate::ifd::ExifTagGroup;
use crate::ifd::ImageFileDirectory;
use crate::tiff;
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

		for dir in &dirs
		{
			all_tags.extend(dir.tags.clone());
		}


		let tiffdata = Tiffdata {endian: endian.clone(), image_file_directories: dirs};
		tiffdata.generic_encode_data();

		return Ok((endian, all_tags));
	}

	pub fn
	get_endian
	(
		&self
	)
	-> Endian
	{
		self.endian.clone()
	}

	pub fn
	get_ifds
	(
		&self
	)
	-> &Vec<ImageFileDirectory>
	{
		&self.image_file_directories
	}

	/// Assumes that the data is sorted according to `sort_data`
	pub fn
	generic_encode_data
	(
		self
	)
	{
		// Prepare offset information
		let mut generic_ifd_count = 0;
		let mut ifds_with_offset_info_only: Vec<ImageFileDirectory> = Vec::new();

		for ifd in self.image_file_directories.iter() // .rev()
		{
			// Insert all IFDs as empty IFDs
			ifds_with_offset_info_only.push(
				ImageFileDirectory::new_with_tags(
					vec![], 
					ifd.get_ifd_type(), 
					ifd.get_generic_ifd_nr()
				)
			);

			generic_ifd_count = std::cmp::max(ifd.get_generic_ifd_nr(), generic_ifd_count);

			if let Some((parent_ifd_group, offset_tag)) = ifd.get_offset_tag_for_parent_ifd()
			{
				// Check if the parent IFD is already in the vector
				if let Some(parent_ifd) = ifds_with_offset_info_only
					.iter_mut()
					.find(|parent_ifd| 
						parent_ifd.get_ifd_type() == parent_ifd_group && 
						parent_ifd.get_generic_ifd_nr() == ifd.get_generic_ifd_nr()
					)
				{
					parent_ifd.add_tag(offset_tag);
				}
				else
				{
					panic!("THIS SHOULD NOT HAPPEN! (generic_encode_data)")
				}
			}
		}

		// Now traverse the IFDs, starting with the SubIFDs associated with 
		// IFD0, then IFD0 itself. Next, SubIFDs for IFD1, IFD1 itself, and
		// so on up to IFD-n.
		let generic_ifd_count = self.image_file_directories.iter()
			.filter(|ifd| ifd.get_ifd_type() == ExifTagGroup::GENERIC)
			.max_by(|ifd1, ifd2| ifd1.get_generic_ifd_nr().cmp(&ifd2.get_generic_ifd_nr()))
			.unwrap()
			.get_generic_ifd_nr();
		
		// let mut processed_ifd_indices: Vec<usize> = Vec::new();
		for n in 0..generic_ifd_count
		{
			// self.image_file_directories.iter()
			// 	.enumerate()
			// 	.filter(|(_, ifd)| ifd.get_generic_ifd_nr() == n)
			// 	.filter(|(index, _)| !processed_ifd_indices.contains(&index));

			loop 
			{
				let next_ifds_to_process: Vec<&ImageFileDirectory> = ifds_with_offset_info_only
					.iter()
					.filter(|ifd| ifd.get_generic_ifd_nr() == n)
					.filter(|ifd| ifd.tags.len() == 0)
					.collect();

				if next_ifds_to_process.is_empty()
				{
					break;
				}

				for current_ifd in next_ifds_to_process
				{



					current_ifd.get_offset_tag_for_parent_ifd()
				}
			}
		}



		for offset_ifd in &ifds_with_offset_info_only
		{
			println!("{:?} {:?}", offset_ifd, offset_ifd.tags);
		}

		println!("DONE\n");

	}

	fn
	sort_data
	(
		&mut self
	)
	{
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
 
	fn
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
