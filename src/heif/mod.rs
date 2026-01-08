// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/// Note: While the standard 14496-12 (which defines the base ISO BMFF stuff
/// but with focus on video files) states that a `moov` box is *required* on 
/// top level, the Image File Format standard 23008-12 tells us that files with
/// the brand `mif1` do *not* require such a box. 

mod box_type;
mod box_header;
mod boxes;
mod container;

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;

use crate::general_file_io::open_read_file;
use crate::general_file_io::open_write_file;

use crate::general_file_io::EXIF_HEADER;
use crate::metadata::Metadata;

use crate::heif::boxes::read_next_box;
use crate::heif::container::HeifContainer;

fn
generic_read_metadata
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<Vec<u8>, std::io::Error>
{
    let container = HeifContainer::construct_from_cursor_unboxed(cursor)?;
    return container.get_exif_data(cursor);
}

pub(crate) fn
read_metadata
(
    file_buffer: &[u8]
)
-> Result<Vec<u8>, std::io::Error>
{
    let mut cursor = Cursor::new(file_buffer);
    return generic_read_metadata(&mut cursor);
}

pub(crate) fn
file_read_metadata
(
    path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
    let mut file = open_read_file(path)?;
    return generic_read_metadata(&mut file);
}



pub(crate) fn
write_metadata
(
	file_buffer: &mut Vec<u8>,
	metadata:    &Metadata
)
-> Result<(), std::io::Error> 
{
    let mut cursor    = Cursor::new(file_buffer);
    let mut container = HeifContainer::construct_from_cursor_unboxed(&mut cursor)?;

    return container.generic_write_metadata(cursor.get_mut(), metadata);
}

pub(crate) fn
file_write_metadata
(
    path:     &Path,
    metadata: &Metadata
)
-> Result<(), std::io::Error>
{
    // Load the entire file into memory instead of performing multiple read, 
    // seek and write operations
    let mut file = open_write_file(path)?;
    let mut file_buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut file_buffer)?;

    let mut cursor    = Cursor::new(file_buffer);
    let mut container = HeifContainer::construct_from_cursor_unboxed(&mut cursor)?;

    container.generic_write_metadata(cursor.get_mut(), metadata)?;

    // Seek back to start, write the file and adjust its length, possibly 
    // truncating the file if new contents are shorter
    file.seek(std::io::SeekFrom::Start(0))?;
    file.write_all(cursor.get_mut())?;
    file.set_len(cursor.get_ref().len() as u64)?;

    return Ok(());
}

/// Encodes the given metadata into a vector of bytes that can be used as
/// an exif box in an HEIF file.
pub(crate) fn 
as_u8_vec
(
    general_encoded_metadata: &[u8]
) 
-> Vec<u8> 
{
    let mut data_buffer: Vec<u8> = Vec::new();

    // Length of the EXIF HEADER
    data_buffer.extend(vec![0u8, 0u8, 0u8, 6u8]);

    // Actual EXIF HEADER
    data_buffer.extend(EXIF_HEADER.iter());

    // And the exif data itself
    data_buffer.extend(general_encoded_metadata.iter());

    return data_buffer;
}

pub(crate) fn
clear_metadata
(
    file_buffer: &mut Vec<u8>
)
-> Result<(), std::io::Error>
{
    let mut cursor    = Cursor::new(file_buffer);
    let mut container = HeifContainer::construct_from_cursor_unboxed(&mut cursor)?;

    return container.generic_clear_metadata(cursor.get_mut());
}

pub(crate) fn
file_clear_metadata
(
    path: &Path
)
-> Result<(), std::io::Error>
{
    // Load the entire file into memory instead of performing multiple read, 
    // seek and write operations
    let mut file = open_write_file(path)?;
    let mut file_buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut file_buffer)?;

    let mut cursor    = Cursor::new(file_buffer);
    let mut container = HeifContainer::construct_from_cursor_unboxed(&mut cursor)?;

    container.generic_clear_metadata(cursor.get_mut())?;

    // Seek back to start, write the file and adjust its length, possibly 
    // truncating the file if new contents are shorter
    file.seek(std::io::SeekFrom::Start(0))?;
    file.write_all(cursor.get_mut())?;
    file.set_len(cursor.get_ref().len() as u64)?;

    return Ok(());
}
