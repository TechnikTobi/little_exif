// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::path::Path;

use log::warn;

use crate::filetype::get_file_type;
use crate::filetype::FileExtension;
use crate::general_file_io::io_error;

use crate::general_file_io::open_read_file;
use crate::heif;
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
        // First, try to determine the file type automatically
        let mut cursor = Cursor::new(file_buffer);
        let auto_detected_file_type = FileExtension::auto_detect(&mut cursor);

        if let Some(detected_type) = auto_detected_file_type
        {
            if file_type != detected_type
            {
                warn!(
                    "The supplied file type information ({:?}) and detected ({:?}) do NOT match!",
                    file_type,
                    detected_type
                );
            }
        }
        else
        {
            warn!("Could not automatically detect file type!");
        }

        let raw_pre_decode_general = match file_type
        {
            FileExtension::HEIF
                => heif::read_metadata(file_buffer),
            FileExtension::JPEG 
                =>  jpg::read_metadata(file_buffer),
            FileExtension::JXL
                =>  jxl::read_metadata(file_buffer),
            FileExtension::PNG { as_zTXt_chunk: _ }
                =>  png::read_metadata(file_buffer),
            FileExtension::TIFF
                => tiff::vec::read_metadata(file_buffer),
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
        // First, try to get the type based on the file extension
        let extension_based_file_type_result = get_file_type(path);

        let mut extension_based_file_type = match extension_based_file_type_result 
        {
            Ok(result) => Some(result),
            Err(error) => match error.kind() 
            {
                std::io::ErrorKind::Unsupported => return Err(error),
                _ => None
            },
        };

        // Next, try to use auto detect
        let mut file = open_read_file(path)?;
        let content_based_file_type = FileExtension::auto_detect(&mut file);

        if extension_based_file_type.is_none()
        {
            if content_based_file_type.is_none()
            {
                return io_error!(
                    Other,
                    "Could not determine file type when reading file!"
                );
            }

            extension_based_file_type = content_based_file_type;
        }

        if content_based_file_type.is_some()
        {
            if 
                extension_based_file_type.unwrap() 
                != 
                content_based_file_type.unwrap()
            {
                warn!("File extension and file content yield different file type, content takes precedence");
                extension_based_file_type = content_based_file_type;
            }
        }
        else
        {
            warn!("Could not determine file type based on content, fall back on file extension");
        }

        let file_type = extension_based_file_type.unwrap();

        // Call the file specific decoders as a starting point for obtaining
        // the raw EXIF data that gets further processed
        let raw_pre_decode_general = match file_type
        {
            FileExtension::HEIF
                => heif::file_read_metadata(path),
            FileExtension::JPEG 
                =>  jpg::file_read_metadata(path),
            FileExtension::JXL
                =>  jxl::file_read_metadata(path),
            FileExtension::PNG { as_zTXt_chunk: _ } 
                =>  png::file_read_metadata(path),
            FileExtension::TIFF
                => tiff::file::read_metadata(path),
            FileExtension::WEBP 
                => webp::file::read_metadata(path),
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
    clear_metadata
    (
        file_buffer: &mut Vec<u8>,
        file_type:   FileExtension
    )
    -> Result<(), std::io::Error>
    {
        match file_type
        {
            FileExtension::HEIF
                => heif::clear_metadata(file_buffer),
            FileExtension::JPEG 
                =>  jpg::clear_metadata(file_buffer),
            FileExtension::JXL
                =>  jxl::clear_metadata(file_buffer),
            FileExtension::PNG { as_zTXt_chunk: _ }
                =>  png::clear_metadata(file_buffer),
            FileExtension::TIFF
                => tiff::vec::clear_metadata(file_buffer),
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

    /// Clears the APP12 segment in a JPEG file that contains data resulting
    /// from exporting the file via Photoshop. This may be required in order
    /// for other software to see e.g. the ImageDescription written in the
    /// APP1 exif segment by little_exif
    #[allow(unreachable_patterns)]
    pub fn
    clear_app12_segment
    (
        file_buffer: &mut Vec<u8>,
        file_type:   FileExtension
    )
    -> Result<(), std::io::Error>
    {
        match file_type
        {
            FileExtension::JPEG 
                =>  jpg::clear_segment(file_buffer, 0xec),
            _
                => return io_error!(
                    Other, 
                    format!(
                        "Function 'clear_app12_segment' not available for {:?} (only relevant for JPEG)", 
                        file_type
                    )
                ),
        }
    }

    /// Clears the APP13 segment in a JPEG file that contains data resulting
    /// from exporting the file via Photoshop. This may be required in order
    /// for other software to see e.g. the ImageDescription written in the
    /// APP1 exif segment by little_exif
    #[allow(unreachable_patterns)]
    pub fn
    clear_app13_segment
    (
        file_buffer: &mut Vec<u8>,
        file_type:   FileExtension
    )
    -> Result<(), std::io::Error>
    {
        match file_type
        {
            FileExtension::JPEG 
                =>  jpg::clear_segment(file_buffer, 0xed),
            _
                => return io_error!(
                    Other, 
                    format!(
                        "Function 'clear_app13_segment' not available for {:?} (only relevant for JPEG)", 
                        file_type
                    )
                ),
        }
    }

    /// Clears the APP12 segment in a JPEG file that contains data resulting
    /// from exporting the file via Photoshop. This may be required in order
    /// for other software to see e.g. the ImageDescription written in the
    /// APP1 exif segment by little_exif
    #[allow(unreachable_patterns)]
    pub fn
    file_clear_app12_segment
    (
        path: &Path
    )
    -> Result<(), std::io::Error>
    {
        let file_type = get_file_type(path)?;

        match file_type
        {
            FileExtension::JPEG 
                =>  jpg::file_clear_segment(path, 0xec),
            _
                => return io_error!(
                    Other, 
                    format!(
                        "Function 'file_clear_app12_segment' not available for {:?} (only relevant for JPEG)", 
                        file_type
                    )
                ),
        }
    }

    /// Clears the APP13 segment in a JPEG file that contains data resulting
    /// from exporting the file via Photoshop. This may be required in order
    /// for other software to see e.g. the ImageDescription written in the
    /// APP1 exif segment by little_exif
    #[allow(unreachable_patterns)]
    pub fn
    file_clear_app13_segment
    (
        path: &Path
    )
    -> Result<(), std::io::Error>
    {
        let file_type = get_file_type(path)?;

        match file_type
        {
            FileExtension::JPEG 
                =>  jpg::file_clear_segment(path, 0xed),
            _
                => return io_error!(
                    Other, 
                    format!(
                        "Function 'file_clear_app13_segment' not available for {:?} (only relevant for JPEG)", 
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
            FileExtension::HEIF
                => heif::file_clear_metadata(path),
            FileExtension::JPEG 
                =>  jpg::file_clear_metadata(path),
            FileExtension::JXL
                =>  jxl::file_clear_metadata(path),
            FileExtension::PNG { as_zTXt_chunk: _ }
                =>  png::file_clear_metadata(path),
            FileExtension::TIFF
                => tiff::file::clear_metadata(path),
            FileExtension::WEBP 
                => webp::file::clear_metadata(path),
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
    -> Result<Vec<u8>, std::io::Error>
    {
        let general_encoded_metadata = self.encode()?;

        Ok(match for_file_type
        {
            FileExtension::PNG { as_zTXt_chunk } 
                =>  png::as_u8_vec(&general_encoded_metadata, as_zTXt_chunk),
            FileExtension::JPEG 
                =>  jpg::as_u8_vec(&general_encoded_metadata),
            FileExtension::WEBP 
                 => webp::as_u8_vec(&general_encoded_metadata),
            FileExtension::HEIF 
                => heif::as_u8_vec(&general_encoded_metadata),
            _ => {
                unimplemented!()
            }
        })
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
            FileExtension::HEIF
                => heif::write_metadata(file_buffer, self),
            FileExtension::JPEG 
                =>  jpg::write_metadata(file_buffer, self),
            FileExtension::JXL 
                =>  jxl::write_metadata(file_buffer, self),
            FileExtension::PNG { as_zTXt_chunk: _ }
                =>  png::write_metadata(file_buffer, self),
            FileExtension::TIFF
                => tiff::vec::write_metadata(file_buffer, self),
            FileExtension::WEBP
                => webp::vec::write_metadata(file_buffer, self),
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
            FileExtension::HEIF
                => heif::file_write_metadata(path, self),
            FileExtension::JPEG 
                =>  jpg::file_write_metadata(path, self),
            FileExtension::JXL 
                =>  jxl::file_write_metadata(path, self),
            FileExtension::PNG { as_zTXt_chunk: _ }
                =>  png::file_write_metadata(path, self),
            FileExtension::TIFF
                => tiff::file::write_metadata(path, self),
            FileExtension::WEBP 
                => webp::file::write_metadata(path, self),
            _
                => return io_error!(
                    Other, 
                    format!(
                        "Function 'write_to_file' not yet implemented for {:?}", 
                        file_type
                    )
                ),
        }
    }
}