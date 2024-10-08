// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::str::FromStr;
use std::path::Path;

use crate::general_file_io::*;

#[derive(Debug, PartialEq)]
#[allow(non_snake_case)]
pub enum
FileExtension
{
	PNG  {as_zTXt_chunk: bool},
	JPEG,
	JXL,
	TIFF,
	WEBP
}

impl 
FromStr 
for 
FileExtension 
{
	type Err = std::io::Error;

	fn 
	from_str
	(
		input: &str
	) 
	-> Result<FileExtension, Self::Err> 
	{
		match input 
		{
			"jpg"   => Ok(FileExtension::JPEG),
			"jpeg"  => Ok(FileExtension::JPEG),
			"jxl"   => Ok(FileExtension::JXL),
			"png"   => Ok(FileExtension::PNG{ as_zTXt_chunk: true}),
			"tif"   => Ok(FileExtension::TIFF),
			"tiff"  => Ok(FileExtension::TIFF),
			"webp"  => Ok(FileExtension::WEBP),
			_       => io_error!(Unsupported, "Unknown file type!")
		}
	}
}



pub(crate) fn
get_file_type
(
	path: &Path
)
-> Result<FileExtension, std::io::Error>
{
	if !path.exists()
	{
		return io_error!(Other, "File does not exist!");
	}

	let raw_file_type_str = path.extension();
	if raw_file_type_str.is_none()
	{
		return io_error!(Other, "Can't get extension from given path!");
	}

	let file_type_str = raw_file_type_str.unwrap().to_str();
	if file_type_str.is_none()
	{
		return io_error!(Other, "Can't convert file type to string!");
	}

	let raw_file_type = FileExtension::from_str(file_type_str.unwrap().to_lowercase().as_str());
	if raw_file_type.is_err()
	{
		return io_error!(Unsupported, "Unsupported file type!");
	}
	else
	{
		return Ok(raw_file_type.unwrap());
	}
}