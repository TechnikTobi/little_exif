// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use core::panic;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

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

		let (endian, dirs) = Self::decode(&mut cursor)?;

		let mut all_tags = Vec::new();

		for dir in &dirs
		{
			all_tags.extend(dir.get_tags().clone());
		}


		let tiffdata = Tiffdata {endian: endian.clone(), image_file_directories: dirs};
		tiffdata.encode()?;

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
	encode
	(
		self
	)
	-> Result<(), std::io::Error>
	{
		// Prepare offset information
		let mut generic_ifd_count = 0;
		let mut ifds_with_offset_info_only: Vec<ImageFileDirectory> = Vec::new();

		for ifd in self.image_file_directories.iter() // .rev()
		{
			ifds_with_offset_info_only.push(
				ImageFileDirectory::new_with_tags(
					vec![], 
					ifd.get_ifd_type(), 
					ifd.get_generic_ifd_nr()
				)
			);
		}

		for ifd in self.image_file_directories.iter()
		{
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
					panic!("THIS SHOULD NOT HAPPEN!");
				}
			}
		}

		// for offset_ifd in &ifds_with_offset_info_only
		// {
		// 	println!("{:?} {:?}", offset_ifd, offset_ifd.get_tags());
		// }

		// Now traverse the IFDs, starting with the SubIFDs associated with 
		// IFD0, then IFD0 itself. Next, SubIFDs for IFD1, IFD1 itself, and
		// so on up to IFD-n.
		let generic_ifd_count = self.image_file_directories.iter()
			.filter(|ifd| ifd.get_ifd_type() == ExifTagGroup::GENERIC)
			.max_by(|ifd1, ifd2| ifd1.get_generic_ifd_nr().cmp(&ifd2.get_generic_ifd_nr()))
			.unwrap()
			.get_generic_ifd_nr();
		
		let mut index_of_previous_ifds_link_section: Option<u64> = None;

		let mut encode_vec     = Vec::from(self.endian.header());
		let mut current_offset = 8;

		for n in 0..=generic_ifd_count
		{
			let filter_result = self.image_file_directories.iter().filter(|ifd|
				ifd.get_generic_ifd_nr() == n &&
				ifd.get_ifd_type()       == ExifTagGroup::GENERIC
			).collect::<Vec<&ImageFileDirectory>>();

			assert!(filter_result.len() <= 1);

			if let Ok((next_link_section, link_vec)) = filter_result.last().unwrap().encode_ifd(
				&self, 
				&mut ifds_with_offset_info_only, 
				&mut encode_vec, 
				&mut current_offset
			)
			{
				if let Some(index) = index_of_previous_ifds_link_section
				{
					let mut cursor = Cursor::new(&mut encode_vec);
					cursor.set_position(index);
					cursor.write_all(&link_vec)?;
				}

				index_of_previous_ifds_link_section = Some(next_link_section);
			}
		}

		Ok(())
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
	decode
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

		Tiffdata::decode(&mut Cursor::new(&image_data))?;

		Ok(())
	}

	#[test]
	fn
	new_test_2()
	-> Result<(), std::io::Error>
	{
		// let image_data = read("tests/multi_page.tif").unwrap();
		let image_data = read("tests/multi_page_mod.tif").unwrap();

		let data = Tiffdata::decode(&mut Cursor::new(&image_data))?;

		for ifd in data.1
		{
			println!("{:?}", ifd);
		}

		Ok(())
	}
}
