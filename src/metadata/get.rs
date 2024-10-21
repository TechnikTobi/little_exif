// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

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
}