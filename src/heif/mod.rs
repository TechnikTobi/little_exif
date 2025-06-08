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
use std::path::Path;

use crate::general_file_io::open_read_file;
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
    return Ok(container.get_exif_data(cursor)?[4..].to_vec());
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