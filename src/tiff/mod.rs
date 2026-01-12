// Copyright © 2024, 2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Seek;
use std::io::Read;
use std::io::Write;

use log::warn;

use crate::general_file_io::EXIF_HEADER;
use crate::io_error;
use crate::metadata::Metadata;
use crate::ifd::ExifTagGroup::*;

pub mod file;
pub mod vec;

pub(crate) fn
generic_write_metadata
<T: Seek + Write>
(
    cursor:   &mut T,
    metadata: &Metadata
)
-> Result<(), std::io::Error>
{
    // First, check for required tags
    check_for_required_tags(metadata)?;

    // Does *not* call generic_clear_metadata, as the entire tiff data gets
    // overwritten anyways
    cursor.write_all(&metadata.encode()?)?;

    return Ok(());
}

fn
check_for_required_tags
(
    metadata: &Metadata
)
-> Result<(), std::io::Error>
{
    // First, check tags that are *definitely* required for TIFF compliance

    // ImageWidth: 0x0100
    if metadata.get_tag_by_hex(0x0100, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires ImageWidth (0x0100) tag!");
    }

    // ImageHeight: 0x0101
    if metadata.get_tag_by_hex(0x0101, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires ImageHeight (0x0101) tag!");
    }

    // Compression: 0x0103
    if metadata.get_tag_by_hex(0x0103, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires Compression (0x0103) tag!");
    }

    // PhotometricInterpretation: 0x0106
    if metadata.get_tag_by_hex(0x0106, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires PhotometricInterpretation (0x0106) tag!");
    }

    // StripOffsets: 0x0111
    if metadata.get_tag_by_hex(0x0111, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires StripOffsets (0x0111) tag!");
    }

    // RowsPerStrip: 0x0116
    if metadata.get_tag_by_hex(0x0116, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires RowsPerStrip (0x0116) tag!");
    }

    // StripByteCounts: 0x0117
    if metadata.get_tag_by_hex(0x0117, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires StripByteCounts (0x0117) tag!");
    }

    // XResolution: 0x011A
    if metadata.get_tag_by_hex(0x011A, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires XResolution (0x011A) tag!");
    }

    // YResolution: 0x011B
    if metadata.get_tag_by_hex(0x011B, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires YResolution (0x011B) tag!");
    }

    // ResolutionUnit: 0x0128
    if metadata.get_tag_by_hex(0x0128, Some(GENERIC)).count() == 0
    {
        return io_error!(NotFound, "TIFF requires ResolutionUnit (0x0128) tag!");
    }

    // Now check for tags that are required only by some TIFF variants

    // BitsPerSample: 0x0102
    if metadata.get_tag_by_hex(0x0102, Some(GENERIC)).count() == 0
    {
        warn!("All TIFF variants (except for bilevel graphics) require BitsPerSample (0x0102) tag!");
    }

    // SamplesPerPixel: 0x0115
    if metadata.get_tag_by_hex(0x0115, Some(GENERIC)).count() == 0
    {
        warn!("Full-Color TIFFs require SamplesPerPixel (0x0115) tag!");
    }

    // ColorMap: 0x0140
    if metadata.get_tag_by_hex(0x0140, Some(GENERIC)).count() == 0
    {
        warn!("Palette-Color TIFFs require ColorMap (0x0140) tag!");
    }

    return Ok(())
}

fn
generic_read_metadata
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<Vec<u8>, std::io::Error>
{
    let mut tiff_with_exif_header = Vec::new();
    tiff_with_exif_header.extend(EXIF_HEADER);

    let mut buffer = Vec::new();
    cursor.read_to_end(&mut buffer)?;
    tiff_with_exif_header.append(&mut buffer);
    
    return Ok(tiff_with_exif_header);
}
