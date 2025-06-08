// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/// Note: While the standard 14496-12 (which defines the base ISO BMFF stuff
/// but with focus on video files) states that a `moov` box is *required* on 
/// top level, the Image File Format standard 23008-12 tells us that files with
/// the brand `mif1` do *not* require such a box. 

mod box_type;
mod box_header;
mod boxes;

use std::io::Read;
use std::io::Seek;
use std::path::Path;

use crate::general_file_io::open_read_file;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::read_next_box;


// pub(crate) fn
// vec_parse_heif
// (
//     file_buffer: &[u8]
// )
// -> Result<Vec<IsoBox>, std::io::Error>
// {
//     todo!()
// }

fn
generic_parse_heif
<T: Seek + Read>
(
	cursor: &mut T
)
-> Result<Vec<Box<dyn GenericIsoBox>>, std::io::Error>
{
    let mut boxes = Vec::new();

    loop 
    {
        if let Ok(next_box) = read_next_box(cursor)
        {
            boxes.push(next_box);
        }
        else
        {
            break;
        }
    }

    println!("HEIF!");

	return Ok(boxes);
}


pub(crate) fn
read_metadata
(
    file_buffer: &[u8]
)
-> Result<Vec<u8>, std::io::Error>
{
    // vec_parse_heif(file_buffer)?;
    // println!("HEIF!");
    todo!()
}

pub(crate) fn
file_read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
    /* 
	// Parse the PNG - if this fails, the read fails as well
	let parse_png_result = file_parse_png(path)?;

	// Parsed PNG is Ok to use - Open the file and go through the chunks
	let mut file = file_check_signature(path).unwrap();

	return generic_read_metadata(&mut file, &parse_png_result);
    */

    let mut file = open_read_file(path)?;
    generic_parse_heif(&mut file)?;

    todo!()
}