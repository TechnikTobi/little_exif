// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::str::FromStr;

#[derive(Debug, PartialEq)]
#[allow(non_snake_case)]
pub enum
FileExtension
{
	PNG  {as_zTXt_chunk: bool},
	JPEG,
	WEBP
}

impl 
FromStr 
for 
FileExtension 
{
	type Err = ();

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
			"png"   => Ok(FileExtension::PNG{ as_zTXt_chunk: true}),
			"webp"  => Ok(FileExtension::WEBP),
			_       => Err(()),
		}
	}
}
