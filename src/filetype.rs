// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::path::Path;
use std::str::FromStr;

use crate::general_file_io::*;

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(non_snake_case, non_camel_case_types)]
pub enum
FileExtension
{
    PNG  {as_zTXt_chunk: bool},
    JPEG,
    JXL,
    NAKED_JXL,  // A JXL codestream without any data
    TIFF,
    WEBP,
    HEIF,
}

impl
FileExtension
{
    pub(crate) fn
    auto_detect
    <T: Seek + Read>
    (
        cursor: &mut T
    )
    -> Option<Self>
    {
        // Read first few bytes (32 bytes because I don't know any better)
        let mut buffer = [0; 32];
        let Ok(n) = cursor.read(&mut buffer) else {
            return None;
        };

        if n < 4
        {
            return None;
        }

        match buffer {
            // PNG
            [0x89, 0x50, 0x4E, 0x47, ..] => {
                return Some(FileExtension::PNG { as_zTXt_chunk: true });
            }

            // JP(E)G
            [0xFF, 0xD8, ..] => {
                return Some(FileExtension::JPEG);
            }

            // TIFF, little endian
            [0x49, 0x49, 0x2A, 0x00, ..] => {
                return Some(FileExtension::TIFF);
            }

            // TIFF, big endian
            [0x4D, 0x4D, 0x00, 0x2A, ..] => {
                return Some(FileExtension::TIFF);
            }

            // WebP
            [0x52, 0x49, 0x46, 0x46, _, _, _, _, 0x57, 0x45, 0x42, 0x50, ..] =>
            {
                return Some(FileExtension::WEBP);
            }

            // A "naked" JXL codestream that can't hold metadata
            // See: https://www.loc.gov/preservation/digital/formats/fdd/fdd000538.shtml
            [0xFF, 0x0A, ..] => {
                return Some(FileExtension::NAKED_JXL);
            }

            // JXL (in ISO_BMFF container)
            // In this case, the JXL file starts with the JXL signature box
            // 4 bytes for length       J     X     L  space more stuff
            [0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A, ..] =>
            {
                return Some(FileExtension::JXL);
            }

            // HEIC/HEIF/AVIF
            // length       f     t     y     p 
              [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63, ..]  // heic
            | [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x66, ..]  // heif
            | [_, _, _, _, 0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66, ..]  // avif
            => 
            {
                return Some(FileExtension::HEIF)
            }

            // TODO: Other HEIF formats, e.g. ftypmif1, see also:
            // https://www.loc.gov/preservation/digital/formats/fdd/fdd000526.shtml

            _ => { 
                return None;
            }
        };
    }
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
        match input.to_lowercase().as_str()
        {
            "heif" | "hif" | "heic" | "avif"
                => Ok(FileExtension::HEIF),
            "jpeg" | "jpg" 
                => Ok(FileExtension::JPEG),
            "jxl" 
                => Ok(FileExtension::JXL),
            "png" 
                => Ok(FileExtension::PNG { as_zTXt_chunk: true}),
            "tiff" | "tif" 
                => Ok(FileExtension::TIFF),
            "webp" 
                => Ok(FileExtension::WEBP),
            _ => io_error!(Unsupported, format!("Unknown file type: {}", input)),
        }
    }
}

pub fn 
get_file_type
(
    path: &Path
) 
-> Result<FileExtension, io::Error> 
{
    if !path.try_exists()? 
    {
        return io_error!(Other, "File does not exist!");
    }

    let raw_file_type_str = path.extension()
        .ok_or(io::Error::new(ErrorKind::Other, "Can't get file extension!"))?;

    let file_type_str = raw_file_type_str.to_str()
        .ok_or(io::Error::new(ErrorKind::Other, "Can't convert extension!"))?;

    FileExtension::from_str(file_type_str.to_lowercase().as_str()).map_err(|e| 
        {
            io::Error::new(
                ErrorKind::Unsupported,
                format!("Unsupported file type: {file_type_str} - {e}"),
            )
        }
    )
}

#[cfg(test)]
mod tests 
{
    use super::*;

    #[test]
    fn str_parse() 
    {
        let table = vec![
            ("png",  FileExtension::PNG { as_zTXt_chunk: true }),
            ("jpg",  FileExtension::JPEG),
            ("jpeg", FileExtension::JPEG),
            ("jxl",  FileExtension::JXL),
            ("tif",  FileExtension::TIFF),
            ("tiff", FileExtension::TIFF),
            ("webp", FileExtension::WEBP),
        ];

        for (input, expected) in table 
        {
            let result = FileExtension::from_str(input);
            assert!(result.is_ok(), "Failed to parse '{}'", input);
            assert_eq!(result.unwrap(), expected, "Parsed value mismatch for '{}'", input);
        }
    }
}