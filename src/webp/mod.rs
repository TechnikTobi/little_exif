// Copyright © 2024, 2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub mod file;
pub mod vec;

mod riff_chunk;

pub(crate) const RIFF_SIGNATURE:       [u8; 4] = [0x52, 0x49, 0x46, 0x46];
pub(crate) const WEBP_SIGNATURE:       [u8; 4] = [0x57, 0x45, 0x42, 0x50];
pub(crate) const VP8X_HEADER:          &str    = "VP8X";
pub(crate) const EXIF_CHUNK_HEADER:    &str    = "EXIF";

use std::fs::File;

use crate::endian::Endian;
use crate::general_file_io::io_error;
use crate::io_error_plain;
use crate::u8conversion::from_u8_vec_res_macro;
use crate::u8conversion::to_u8_vec_macro;
use crate::u8conversion::U8conversion;

fn
check_riff_signature
(
    file_buffer: &[u8],
)
-> Result<(), std::io::Error>
{
    let bytes_to_check = match file_buffer.get(0..4)
    {
        Some(bytes) => bytes,
        None => {
            return io_error!(InvalidData, "Can't open WebP file - File too small to contain RIFF signature!");
        }
    };

    if bytes_to_check != RIFF_SIGNATURE {
        return io_error!(
            InvalidData,
            format!("Can't open WebP file - Expected RIFF signature but found {}!", from_u8_vec_res_macro!(String, bytes_to_check, &Endian::Big)?)
        );
    }

    return Ok(());
}

fn
check_webp_signature
(
    file_buffer: &[u8],
)
-> Result<(), std::io::Error>
{
    let Some(buffer_to_check) = file_buffer.get(8..12) else {
        return io_error!(InvalidData, "Can't open WebP file - File too small to contain WEBP signature!");
    };

    if buffer_to_check != WEBP_SIGNATURE
    {
        return io_error!(
            InvalidData, 
            format!("Can't open WebP file - Expected WEBP signature but found {}!", from_u8_vec_res_macro!(String, buffer_to_check, &Endian::Big)?)
        );
    }

    return Ok(());
}

fn
check_byte_count
(
    file_buffer: &[u8],
    opt_file:     Option<&File>,
)
-> Result<(), std::io::Error>
{
    let byte_count = from_u8_vec_res_macro!(
        u32, 
        &file_buffer[4..8], 
        &Endian::Little
    )?.checked_add(8).ok_or(
        io_error_plain!(InvalidData, "Can't open WebP file - Byte count in RIFF header is too large!")
    )?;

    if let Some(file) = opt_file
    {
        if file.metadata()?.len() != byte_count as u64
        {
            return io_error!(InvalidData, "Can't open WebP file - Promised byte count does not correspond with file size!");
        }
    }
    else if file_buffer.len() != byte_count as usize
    {
        return io_error!(InvalidData, format!("Can't handle WebP file buffer - Promised byte count {} does not correspond with file buffer length {}!", byte_count, file_buffer.len()));
    }

    return Ok(());
}

fn
encode_metadata_webp
(
    exif_vec: &[u8],
)
-> Vec<u8>
{
    // Vector storing the data that will be returned
    let mut webp_exif: Vec<u8> = Vec::new();

    // Compute the length of the exif data chunk 
    // This does NOT include the fourCC and size information of that chunk 
    // Also does NOT include the padding byte, i.e. this value may be odd!
    let length = exif_vec.len() as u32;

    // Start with the fourCC chunk head and the size information.
    // Then copy the previously encoded EXIF data 
    webp_exif.extend([0x45, 0x58, 0x49, 0x46]);
    webp_exif.extend(to_u8_vec_macro!(u32, &length, &Endian::Little));
    webp_exif.extend(exif_vec.iter());

    // Add the padding byte if required
    if length % 2 != 0
    {
        webp_exif.extend([0x00]);
    }

    return webp_exif;
}



/// Provides the WebP specific encoding result as vector of bytes to be used
/// by the user (e.g. in combination with another library)
pub(crate) fn
as_u8_vec
(
    general_encoded_metadata: &[u8],
)
-> Vec<u8>
{
    encode_metadata_webp(general_encoded_metadata)
}

fn
get_dimension_info_from_vp8_chunk
(
    payload: &[u8]
)
-> Result<(u32, u32), std::io::Error>
{
    // Get the bytes containing the VP8 frame header info
    // See:
    // VP8 Chunk: https://developers.google.com/speed/webp/docs/riff_container#simple_file_format_lossy
    // VP8 Data Format https://datatracker.ietf.org/doc/html/rfc6386#section-9.1
    // Parsing function function vp8_parse_frame_header: https://datatracker.ietf.org/doc/html/rfc6386#section-20.4

    let header_magic = payload[3..=5].to_vec();
    if 
        header_magic.len() != 3 || 
        !matches!(header_magic.as_slice(), &[0x9d, 0x01, 0x2a]) 
    {
        return io_error!(Other, "Invalid VP8 Frame Header Magic");
    }

    let header_width_bytes  = payload[6..=7].to_vec();
    let header_height_bytes = payload[8..=9].to_vec();

    let width_info  = from_u8_vec_res_macro!(u16, &header_width_bytes,  &Endian::Little)?;
    let height_info = from_u8_vec_res_macro!(u16, &header_height_bytes, &Endian::Little)?;

    let width  = width_info  & ((1 << 14) - 1);
    let height = height_info & ((1 << 14) - 1);

    return Ok((width as u32 -1, height as u32 -1));
}