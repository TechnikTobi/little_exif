// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io;
use std::path::Path;
use std::str::FromStr;

use crate::general_file_io::*;

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_snake_case)]
pub enum FileExtension {
    HEIC,
    JPEG,
    JXL,
    PNG { as_zTXt_chunk: bool },
    TIFF,
    WEBP,
}

impl FromStr for FileExtension {
    type Err = std::io::Error;

    fn from_str(input: &str) -> Result<FileExtension, Self::Err> {
        match input {
            "jpg" => Ok(FileExtension::JPEG),
            "jpeg" => Ok(FileExtension::JPEG),
            "jxl" => Ok(FileExtension::JXL),
            "png" => Ok(FileExtension::PNG {
                as_zTXt_chunk: true,
            }),
            "tif" => Ok(FileExtension::TIFF),
            "tiff" => Ok(FileExtension::TIFF),
            "webp" => Ok(FileExtension::WEBP),
            _ => io_error!(Unsupported, format!("Unknown file type: {}", input)),
        }
    }
}

pub fn get_file_type(path: &Path) -> Result<FileExtension, io::Error> {
    if !path.try_exists()? {
        return io_error!(Other, "File does not exist!");
    }

	let raw_file_type_str = path.extension();
	if raw_file_type_str.is_none()
	{
		return io_error!(Other, "Can't get extension from given path!");
	}

    let file_type_str = raw_file_type_str.unwrap().to_str();
    if file_type_str.is_none() {
        return io_error!(Other, "Can't convert file type to string!");
    }

    let raw_file_type = FileExtension::from_str(file_type_str.unwrap().to_lowercase().as_str());
    if raw_file_type.is_err() {
        io_error!(Unsupported, "Unsupported file type!")
    } else {
        Ok(raw_file_type.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_parse() {
        let table = vec![
            ("png", FileExtension::PNG { as_zTXt_chunk: true }),
            ("jpg", FileExtension::JPEG),
            ("jpeg", FileExtension::JPEG),
            ("jxl", FileExtension::JXL),
            ("tif", FileExtension::TIFF),
            ("tiff", FileExtension::TIFF),
            ("webp", FileExtension::WEBP),
        ];
        for (input, expected) in table {
            let result = FileExtension::from_str(input);
            assert!(result.is_ok(), "Failed to parse '{}'", input);
            assert_eq!(result.unwrap(), expected, "Parsed value mismatch for '{}'", input);
        }
    }
}
