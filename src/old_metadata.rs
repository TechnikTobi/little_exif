// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details



impl
OldMetadata
{	


	/// Gets the stored tag in the metadata for the given tag. 
	/// Returns `None` if the tag is not present in the metadata struct.
	///
	/// # Examples
	/// ```no_run
	/// use little_exif::metadata::Metadata;
	/// use little_exif::exif_tag::ExifTag;
	/// 
	/// let metadata = Metadata::new_from_path(std::path::Path::new("image.png")).unwrap();
	/// let tag = metadata.get_tag(&ExifTag::ImageDescription(String::new()));
	/// ```
	pub fn
	get_tag
	(
		&self,
		input_tag: &ExifTag
	)
	-> Option<&ExifTag> 
	{
		self.get_tag_by_hex(input_tag.as_u16())
	}

	/// Gets the sorted tag in the metadata by its hex value.
	/// Returns `None`if the tag is not present in the metadata struct.
	/// 
	/// # Examples
	/// ```no_run
	/// // Note that the tag identifier of course does not need to be written in hex format
	/// // Hex notation only used in this example for more clarity
	/// use little_exif::metadata::Metadata;
	/// 
	/// let metadata = Metadata::new_from_path(std::path::Path::new("image.png")).unwrap();
	/// let tag = metadata.get_tag_by_hex(0x010e);
	/// ```
	pub fn
	get_tag_by_hex
	(
		&self,
		input_tag_hex: u16
	)
	-> Option<&ExifTag>
	{
		for tag in &self.data
		{
			if tag.as_u16() == input_tag_hex
			{
				return Some(tag);
			}
		}
		return None;
	}

	/// Sets the tag in the metadata struct. If the tag is already in there it gets replaced
	///
	/// # Examples
	/// ```no_run
	/// use little_exif::metadata::Metadata;
	/// use little_exif::exif_tag::ExifTag;
	/// 
	/// let mut metadata = Metadata::new();
	/// metadata.set_tag(
	///     ExifTag::ISO(vec![1234])
	/// );
	/// ```
	pub fn
	set_tag
	(
		&mut self,
		input_tag: ExifTag,
	)
	{
		self.data.retain(|tag| tag.as_u16() != input_tag.as_u16());
		self.data.push(input_tag);

		// Sort the tags by the IFD they will go into the file later on
		self.data.sort_by(
			|a, b| 
			if a.get_group() == b.get_group() 
			{
				// Same group, but unknown should go last 
				if a.is_unknown() == b.is_unknown()
				{
					std::cmp::Ordering::Equal
				}
				else if !a.is_unknown() && b.is_unknown()
				{
					std::cmp::Ordering::Less
				}
				else
				{
					std::cmp::Ordering::Greater
				}
				
			}
			else
			{
				if a.get_group() < b.get_group()                                // e.g. IFD0 < ExifIFD
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

	#[allow(unreachable_patterns)]
	pub fn
	clear_metadata
	(
		file_buffer: &mut Vec<u8>,
		file_type:   FileExtension
	)
	-> Result<(), std::io::Error>
	{
		match file_type
		{
			FileExtension::JPEG 
				=>  jpg::clear_metadata(file_buffer),
			FileExtension::JXL
				=>  jxl::clear_metadata(file_buffer),
			FileExtension::PNG { as_zTXt_chunk: _ }
				=>  png::vec::clear_metadata(file_buffer),
			FileExtension::WEBP
				=> webp::vec::clear_metadata(file_buffer),
			_
				=> return io_error!(
					Other, 
					format!(
						"Function 'clear_metadata' not yet implemented for {:?}", 
						file_type
					)
				),
		}
	}

}
