// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub mod metadata_io;
pub mod iterator;
pub mod edit;
pub mod get;
pub mod set;

use core::panic;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

use crate::endian::*;
use crate::general_file_io::io_error;
use crate::general_file_io::EXIF_HEADER;
use crate::ifd::ExifTagGroup;
use crate::ifd::ImageFileDirectory;
use crate::u8conversion::from_u8_vec_macro;
use crate::u8conversion::U8conversion;

#[derive(Clone)]
pub struct
Metadata
{
	endian:                 Endian,
	image_file_directories: Vec<ImageFileDirectory>
}

impl
Metadata
{

	/// Constructs a new, empty `Metadata` object.
	/// 
	/// This uses little endian notation by default.
	/// 
	/// # Examples
	/// ```no_run
	/// use little_exif::metadata::Metadata;
	///
	/// let mut metadata: Metadata = Metadata::new();
	/// ```
	pub fn
	new
	()
	-> Metadata
	{
		Metadata { endian: Endian::Little, image_file_directories: Vec::new() }
	}

	/// Creates an IFD in this struct if it does not exist yet.
	/// Also handles that parent IFDs are properly created if they don't exist
	/// yet but are required later on for the encoding process.
	pub fn
	create_ifd
	(
		&mut self,
		ifd_type:       ExifTagGroup,
		generic_ifd_nr: u32,
	)
	{
		if let Some(_) = self.get_ifd(ifd_type, generic_ifd_nr)
		{
			return;
		}

		let new_ifd = ImageFileDirectory::new_with_tags(Vec::new(), ifd_type, generic_ifd_nr);
		
		if let Some((parent_ifd_group, _)) = new_ifd.get_offset_tag_for_parent_ifd()
		{
			self.create_ifd(parent_ifd_group, generic_ifd_nr);
		}

		self.image_file_directories.push(new_ifd);
		self.sort_data();
	}


	pub(crate) fn
	general_decoding_wrapper
	(
		raw_pre_decode_general: Result<Vec<u8>, std::io::Error>
	)
	-> Result<Metadata, std::io::Error>
	{
		if let Ok(pre_decode_general) = raw_pre_decode_general
		{
			let mut pre_decode_cursor = Cursor::new(&pre_decode_general);
			let     decoding_result   = Self::decode(&mut pre_decode_cursor);
			if let Ok((endian, image_file_directories)) = decoding_result
			{
				let mut data = Metadata { endian, image_file_directories };
				data.sort_data();
				return Ok(data);
			}
			else
			{
				eprintln!("{}", decoding_result.err().unwrap());
			}
		}
		else
		{
			eprintln!("Error during decoding: {:?}", raw_pre_decode_general.err().unwrap());
		}

		eprintln!("WARNING: Can't read metadata - Create new & empty struct");
		return Ok(Metadata::new());
	}


	/// Assumes that the data is sorted according to `sort_data`
	pub fn
	encode
	(
		&self
	)
	-> Result<Vec<u8>, std::io::Error>
	{
		// Prepare offset information
		let mut ifds_with_offset_info_only: Vec<ImageFileDirectory> = Vec::new();

		for ifd in self.image_file_directories.iter()
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
			if let Some((parent_ifd_group, offset_tag)) = ifd.get_offset_tag_for_parent_ifd()
			{
				// Check if the parent IFD is already in the vector
				if let Some(parent_ifd) = ifds_with_offset_info_only
					.iter_mut()
					.find(|candidate_ifd| 
						candidate_ifd.get_ifd_type()       == parent_ifd_group && 
						candidate_ifd.get_generic_ifd_nr() == ifd.get_generic_ifd_nr()
					)
				{
					parent_ifd.set_tag(offset_tag);
				}
				else
				{
					// This *can* happen! For example, take a new Metadata
					// struct that is empty and insert a tag that belongs to
					// the Exif SubIFD. Then, IFD0 is still missing in self,
					// does *not* get inserted into `ifds_with_offset_info_only`
					// and can thus not be found in the if let above. 
					ifds_with_offset_info_only.push(
						ImageFileDirectory::new_with_tags(
							vec![offset_tag], 
							parent_ifd_group,
							ifd.get_generic_ifd_nr()
						)
					);
				}
			}
		}

		// Now traverse the IFDs, starting with the SubIFDs associated with 
		// IFD0, then IFD0 itself. Next, SubIFDs for IFD1, IFD1 itself, and
		// so on up to IFD-n.
		let generic_ifd_count = self.get_max_generic_ifd_number();
		
		let mut index_of_previous_ifds_link_section: Option<u64> = Some(4);

		let mut encode_vec     = Vec::from(self.endian.header());
		let mut current_offset = 8;

		for n in 0..=generic_ifd_count
		{
			let filter_result = self.image_file_directories.iter().filter(|ifd|
				ifd.get_generic_ifd_nr() == n &&
				ifd.get_ifd_type()       == ExifTagGroup::GENERIC
			).collect::<Vec<&ImageFileDirectory>>();

			assert!(filter_result.len() <= 1);

			if filter_result.len() == 0 { continue; }

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

		Ok(encode_vec)
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
		let mut data_start_position = data_cursor.position();

		// Check if this starts with the Exif header
		let mut first_6_bytes = vec![0u8; 6];
		data_cursor.read_exact(&mut first_6_bytes)?;

		let starts_with_exif_signature = first_6_bytes[0..6].iter()
			.zip(EXIF_HEADER.iter())
			.filter(|&(read, constant)| read == constant)
			.count() == EXIF_HEADER.len();

		// If those 6 bytes are *not* "Exif  " then we need to rewind as these
		// six bytes should then be the endian information and magic number
		// Otherwise the cursor should now be advanced to this area
		if !starts_with_exif_signature
		{
			data_cursor.seek_relative(-(EXIF_HEADER.len() as i64))?;
		}
		else
		{
			// Otherwise we need to adjust the start position
			data_start_position += EXIF_HEADER.len() as u64;
		}


		// Determine endian
		let mut endian_buffer = vec![0u8; 2];
		data_cursor.read_exact(&mut endian_buffer)?;

		let endian = match endian_buffer[0..2]
		{
			[0x49, 0x49] => { Endian::Little },
			[0x4d, 0x4d] => { Endian::Big },
			[0x68, 0x74] => { return io_error!(Other, "Expected endian information, but found something that suspectedly is XMP data") }
			_            => { return io_error!(Other, format!("Illegal endian information: {:?}", endian_buffer)) } 
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

use super::Metadata;

	#[test]
	fn
	new_test_1()
	-> Result<(), std::io::Error>
	{
		let image_data = read("tests/read_sample.tif").unwrap();

		Metadata::decode(&mut Cursor::new(&image_data))?;

		Ok(())
	}

	#[ignore]
	#[test]
	fn
	new_test_2()
	-> Result<(), std::io::Error>
	{
		// let image_data = read("tests/multi_page.tif").unwrap();
		let image_data = read("tests/multi_page_mod.tif").unwrap();

		let data = Metadata::decode(&mut Cursor::new(&image_data))?;

		for ifd in data.1
		{
			println!("{:?}", ifd);
		}

		Ok(())
	}
}
