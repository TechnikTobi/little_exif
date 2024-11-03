// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;
use crate::ifd::ExifTagGroup;

use super::Endian;
use super::ImageFileDirectory;
use super::Metadata;

impl
Metadata
{

	/// Gets the endianness of the metadata
	///
	/// # Examples
	/// ```no_run
	/// use little_exif::metadata::Metadata;
	/// 
	/// let metadata = Metadata::new_from_path(std::path::Path::new("image.png")).unwrap();
	/// let tag_data = metadata.get_tag_by_hex(0x010e, None).next().unwrap().value_as_u8_vec(&metadata.get_endian());
	/// ```
	pub fn
	get_endian
	(
		&self
	)
	-> Endian
	{
		self.endian.clone()
	}

	/// Gets the image file directories stored in the struct
	pub fn
	get_ifds
	(
		&self
	)
	-> &Vec<ImageFileDirectory>
	{
		&self.image_file_directories
	}

	/// Gets an image file directory that is of a specific group an is
	/// associated with a certain generic IFD number
	pub fn 
	get_ifd
	(
		&self,
		group:          ExifTagGroup,
		generic_ifd_nr: u32,
	)
	->  Option<&ImageFileDirectory>
	{
		self.image_file_directories.iter().filter(|ifd| 
			ifd.get_generic_ifd_nr() == generic_ifd_nr &&
			ifd.get_ifd_type()       == group
		).next()
	}

	/// Gets the maximum generic ifd number that any of the struct's IFDs has
	pub fn
	get_max_generic_ifd_number
	(
		&self
	)
	-> u32
	{
		self.image_file_directories.iter()
			.filter(|ifd| ifd.get_ifd_type() == ExifTagGroup::GENERIC)
			.max_by(|ifd1, ifd2| ifd1.get_generic_ifd_nr().cmp(&ifd2.get_generic_ifd_nr()))
			.unwrap()
			.get_generic_ifd_nr()
	}

	/// Gets an image file directory that is of a specific group an is
	/// associated with a certain generic IFD number as a mutable reference. 
	/// If the desired IFD does not exist yet it gets created.
	pub fn 
	get_ifd_mut
	(
		&mut self,
		group:          ExifTagGroup,
		generic_ifd_nr: u32,
	)
	->  &mut ImageFileDirectory
	{
		if self.image_file_directories.iter().filter(|ifd| 
			ifd.get_generic_ifd_nr() == generic_ifd_nr &&
			ifd.get_ifd_type()       == group
		).next().is_none()
		{
			self.create_ifd(group, generic_ifd_nr);
		}

		return self.image_file_directories.iter_mut().filter(|ifd| 
			ifd.get_generic_ifd_nr() == generic_ifd_nr &&
			ifd.get_ifd_type()       == group
		).next().unwrap();
	}
}














impl Metadata
{
	pub fn
	get_tag
	(
		&self,
		tag:   &ExifTag
	)
	-> GetTagIterator
	{
		return self.get_tag_by_hex(tag.as_u16(), Some(tag.get_group()));
	}

	/// Gets a tag from the metadata struct via the hex number and the group
	/// Note: While it is not necessary to provide the group, it may be needed
	/// in some cases as there are tags that have the same tag number, e.g. 
	/// the `InteroperabilityVersion` and the `GPSLatitude` tags.
	pub fn
	get_tag_by_hex
	(
		&self,
		hex:   u16,
		group: Option<ExifTagGroup>,
	)
	-> GetTagIterator
	{
		GetTagIterator 
		{
			metadata:          &self,
			current_ifd_index: 0,
			current_tag_index: 0,
			tag_hex_value:     hex,
			group:             group,
		}
	}
}

pub struct
GetTagIterator<'a>
{
	metadata:          &'a Metadata,
	current_ifd_index: usize,
	current_tag_index: usize,
	tag_hex_value:     u16,
	group:             Option<ExifTagGroup>,
}

impl<'a> Iterator
for GetTagIterator<'a>
{	
	type Item = &'a ExifTag;
	
	fn 
	next
	(
		&mut self
	) 
	-> Option<Self::Item> 
	{
		while self.current_ifd_index < self.metadata.image_file_directories.len()
		{
			// First: Check the group, assuming it is provided
			if let Some(given_group) = self.group
			{
				if given_group != self.metadata.image_file_directories[self.current_ifd_index].get_ifd_type()
				{
					self.current_ifd_index += 1;
					continue;
				}
			}

			// Check the current tag
			// - If it is wrong, increment the tag index
			// - If we can't access that tag, increment IFD index and reset tag index
			if self.current_tag_index < self.metadata.image_file_directories[self.current_ifd_index].get_tags().len()
			{
				self.current_tag_index += 1;

				if self.metadata.image_file_directories[self.current_ifd_index].get_tags()[self.current_tag_index-1].as_u16() == self.tag_hex_value
				{
					return Some(&self.metadata.image_file_directories[self.current_ifd_index].get_tags()[self.current_tag_index-1]);
				}
			}
			else
			{
				self.current_tag_index  = 0;
				self.current_ifd_index += 1;
			}
		}
		return None;
	}
}