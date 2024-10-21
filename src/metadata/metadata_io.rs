// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::path::Path;

use crate::filetype::get_file_type;
use crate::filetype::FileExtension;
use crate::general_file_io::io_error;

use crate::jpg;
use crate::jxl;
use crate::png;
use crate::tiff;
use crate::webp;

use super::Metadata;

impl
Metadata
{
	/// Constructs a new `Metadata` object with the metadata from an image that is stored as a `Vec<u8>`
	/// - If unable to handle the file vector (e.g. unsupported file type, etc.), this (currently) panics.
	/// - If unable to decode the metadata, a new, empty object gets created and returned.
	/// # Examples
	/// ```no_run
	/// use std::fs;
	/// use little_exif::metadata::Metadata;
	/// use little_exif::filetype::FileExtension;
	/// 
	/// let file_data = fs::read("image.jpg").unwrap();
	/// let mut metadata: Metadata = Metadata::new_from_vec(&file_data, FileExtension::JPEG).unwrap();
	/// ```
	#[allow(unreachable_patterns)]
	pub fn
	new_from_vec
	(
		file_buffer: &Vec<u8>,
		file_type:   FileExtension
	)
	-> Result<Metadata, std::io::Error>
	{
		let raw_pre_decode_general = match file_type
		{
			FileExtension::JPEG 
				=>  jpg::read_metadata(file_buffer),
			FileExtension::JXL
				=>  jxl::read_metadata(file_buffer),
			FileExtension::PNG { as_zTXt_chunk: _ }
				=>  png::vec::read_metadata(file_buffer),
			FileExtension::WEBP
				=> webp::vec::read_metadata(file_buffer),
			_
				=> return io_error!(
					Other, 
					format!(
						"Function 'new_from_vec' not yet implemented for {:?}", 
						file_type
					)
				),
		};

		return Self::general_decoding_wrapper(raw_pre_decode_general);
	}

	/// Constructs a new `Metadata` object with the metadata from the image at the specified path.
	/// - If unable to read the file (e.g. does not exist, unsupported file type, etc.), this (currently) panics.
	/// - If unable to decode the metadata, a new, empty object gets created and returned.
	///
	/// # Examples
	/// ```no_run
	/// use little_exif::metadata::Metadata;
	/// 
	/// let mut metadata: Metadata = Metadata::new_from_path(std::path::Path::new("image.png")).unwrap();
	/// ```
	#[allow(unreachable_patterns)]
	pub fn
	new_from_path
	(
		path: &Path
	)
	-> Result<Metadata, std::io::Error>
	{
		let file_type = get_file_type(path)?;

		// Call the file specific decoders as a starting point for obtaining
		// the raw EXIF data that gets further processed
		let raw_pre_decode_general = match file_type
		{
			FileExtension::JPEG 
				=>  jpg::file_read_metadata(&path),
			FileExtension::JXL
				=>  jxl::file_read_metadata(&path),
			FileExtension::PNG { as_zTXt_chunk: _ } 
				=>  png::file::read_metadata(&path),
			FileExtension::TIFF
				=> tiff::file::read_metadata(&path),
			FileExtension::WEBP 
				=> webp::file::read_metadata(&path),
			_
				=> return io_error!(
					Other, 
					format!(
						"Function 'new_from_path' not yet implemented for {:?}", 
						file_type
					)
				),
		};

		return Self::general_decoding_wrapper(raw_pre_decode_general);
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
				=>  jpg::write_metadata(file_buffer, &self.encode()?),
			FileExtension::JXL 
				=>  jxl::write_metadata(file_buffer, &self.encode()?),
			FileExtension::PNG { as_zTXt_chunk: _ }
				=>  png::vec::write_metadata(file_buffer, &self.encode()?),
			FileExtension::WEBP
				=> webp::vec::write_metadata(file_buffer, &self.encode()?),
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
				=>  jpg::file_write_metadata(&path, &self.encode()?),
			FileExtension::JXL 
				=>  jxl::file_write_metadata(&path, &self.encode()?),
			FileExtension::PNG { as_zTXt_chunk: _ }
				=>  png::file::write_metadata(&path, &self.encode()?),
			FileExtension::WEBP 
				=> webp::file::write_metadata(&path, &self.encode()?),
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