// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

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
	/// let tag_data = metadata.get_tag_by_hex(0x010e).unwrap().value_as_u8_vec(metadata.get_endian());
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
	->  Option<&mut ImageFileDirectory>
	{
		if self.image_file_directories.iter().filter(|ifd| 
			ifd.get_generic_ifd_nr() == generic_ifd_nr &&
			ifd.get_ifd_type()       == group
		).next().is_none()
		{
			self.image_file_directories.push(
				ImageFileDirectory::new_with_tags(Vec::new(), group, generic_ifd_nr)
			);
			self.sort_data();
		}

		return self.image_file_directories.iter_mut().filter(|ifd| 
			ifd.get_generic_ifd_nr() == generic_ifd_nr &&
			ifd.get_ifd_type()       == group
		).next();
	}
}