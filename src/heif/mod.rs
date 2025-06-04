// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/// Note: While the standard 14496-12 (which defines the base ISO BMFF stuff
/// but with focus on video files) states that a `moov` box is *required* on 
/// top level, the Image File Format standard 23008-12 tells us that files with
/// the brand `mif1` do *not* require such a box. 

mod iso_box;

use iso_box::BoxHeader;
use iso_box::IsoBox;

pub(crate) fn
vec_parse_heif
(
    file_buffer: &[u8]
)
-> Result<Vec<IsoBox>, std::io::Error>
{
    todo!()
}


pub(crate) fn
read_metadata
(
    file_buffer: &[u8]
)
-> Result<Vec<u8>, std::io::Error>
{
    vec_parse_heif(file_buffer)?;
    todo!()
}
