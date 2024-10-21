// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::path::Path;

use crate::endian::*;
use crate::exif_tag_format::INT16U;
use crate::ifd::ImageFileDirectory;
use crate::jxl;
use crate::tiff;
use crate::tiffdata::Tiffdata;
use crate::u8conversion::*;
use crate::exif_tag::ExifTag;
use crate::ifd::ExifTagGroup;
use crate::exif_tag_format::ExifTagFormat;
use crate::filetype::FileExtension;
use crate::filetype::get_file_type;
use crate::general_file_io::*;

use crate::jpg;
use crate::png;
use crate::webp;

const IFD_ENTRY_LENGTH: u32     = 12;
const IFD_END:          [u8; 4] = [0x00, 0x00, 0x00, 0x00];

#[derive(Clone)]
pub struct
OldMetadata
{
	data:   Vec<ExifTag>,
	endian: Endian 
}

impl
OldMetadata
{	
	/// Gets a shared reference to the list of all tags currently stored in the object.
	///
	/// # Examples
	/// ```no_run
	/// use little_exif::metadata::Metadata;
	/// 
	/// let metadata = Metadata::new_from_path(std::path::Path::new("image.png")).unwrap();
	/// for tag in metadata.data()
	/// {
	///     // do something with the tags	
	/// }
	/// ```
	pub fn
	data
	(
		&self
	)
	-> &Vec<ExifTag>
	{
		&self.data
	}

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
	-> &Endian
	{
		&self.endian
	}

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

	/// Converts the metadata into a file specific vector of bytes
	/// Only to be used in combination with some other library/code that is
	/// able to handle the specific file type.
	/// Simply writing this to a file often is not enough, e.g. with WebP you
	/// have to determine where to write this, update the file size information
	/// and so on - check file type specific implementations or documentation
	/// for further details
	#[allow(unreachable_patterns)]
	pub fn
	as_u8_vec
	(
		&self,
		for_file_type: FileExtension
	)
	-> Vec<u8>
	{
		let general_encoded_metadata = self.encode_metadata_general();

		match for_file_type
		{
			FileExtension::PNG { as_zTXt_chunk } 
				=>  png::as_u8_vec(&general_encoded_metadata, as_zTXt_chunk),
			FileExtension::JPEG 
				=>  jpg::as_u8_vec(&general_encoded_metadata),
			FileExtension::WEBP 
				=> webp::as_u8_vec(&general_encoded_metadata),
			_
				=> Vec::new(),
		}
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

	#[allow(unreachable_patterns)]
	pub fn
	file_clear_metadata
	(
		path: &Path
	)
	-> Result<(), std::io::Error>
	{
		let file_type = get_file_type(path)?;

		match file_type
		{
			FileExtension::JPEG 
				=>  jpg::file_clear_metadata(&path),
			FileExtension::JXL
				=>  jxl::file_clear_metadata(&path),
			FileExtension::PNG { as_zTXt_chunk: _ }
				=>  png::file::clear_metadata(&path),
			FileExtension::WEBP 
				=> webp::file::clear_metadata(&path),
			_
				=> return io_error!(
					Other, 
					format!(
						"Function 'file_clear_metadata' not yet implemented for {:?}", 
						file_type
					)
				),
		}
	}

	/// Writes the metadata to an image stored as a Vec<u8>
	/// For now, this only works for JPGs
	#[allow(unreachable_patterns)]
	pub fn
	write_to_vec
	(
		&self,
		file_buffer: &mut Vec<u8>,
		file_type:   FileExtension
	)
	-> Result<(), std::io::Error>
	{
		match file_type
		{
			FileExtension::JPEG 
				=>  jpg::write_metadata(file_buffer, &self.encode_metadata_general()),
			FileExtension::JXL 
				=>  jxl::write_metadata(file_buffer, &self.encode_metadata_general()),
			FileExtension::PNG { as_zTXt_chunk: _ }
				=>  png::vec::write_metadata(file_buffer, &self.encode_metadata_general()),
			FileExtension::WEBP
				=> webp::vec::write_metadata(file_buffer, &self.encode_metadata_general()),
			_
				=> return io_error!(
					Other, 
					format!(
						"Function 'file_clear_metadata' not yet implemented for {:?}", 
						file_type
					)
				),
		}
	}

	/// Writes the metadata to the specified file.
	/// This could return an error for multiple reasons:
	/// - The file does not exist at the given path
	/// - Interpreting the given path fails
	/// - The file type is not supported
	#[allow(unreachable_patterns)]
	pub fn
	write_to_file
	(
		&self,
		path: &Path
	)
	-> Result<(), std::io::Error>
	{
		let file_type = get_file_type(path)?;

		match file_type
		{
			FileExtension::JPEG 
				=>  jpg::file_write_metadata(&path, &self.encode_metadata_general()),
			FileExtension::JXL 
				=>  jxl::file_write_metadata(&path, &self.encode_metadata_general()),
			FileExtension::PNG { as_zTXt_chunk: _ }
				=>  png::file::write_metadata(&path, &self.encode_metadata_general()),
			FileExtension::WEBP 
				=> webp::file::write_metadata(&path, &self.encode_metadata_general()),
			_
				=> return io_error!(
					Other, 
					format!(
						"Function 'file_clear_metadata' not yet implemented for {:?}", 
						file_type
					)
				),
		}
	}

}
