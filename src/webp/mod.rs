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
use crate::u8conversion::from_u8_vec_macro;
use crate::u8conversion::to_u8_vec_macro;
use crate::u8conversion::U8conversion;

fn
check_riff_signature
(
	file_buffer: &[u8]
)
-> Result<(), std::io::Error>
{
	// Check the RIFF signature
	if !file_buffer[0..4].iter()
		.zip(RIFF_SIGNATURE.iter())
		.filter(|&(read, constant)| read == constant)
		.count() == RIFF_SIGNATURE.len()
	{
		return io_error!(
			InvalidData, 
			format!("Can't open WebP file - Expected RIFF signature but found {}!", from_u8_vec_macro!(String, &file_buffer[0..4], &Endian::Big))
		);
	}

	return Ok(());
}

fn
check_webp_signature
(
	file_buffer: &[u8]
)
-> Result<(), std::io::Error>
{
	if !file_buffer[8..12].iter()
		.zip(WEBP_SIGNATURE.iter())
		.filter(|&(read, constant)| read == constant)
		.count() == WEBP_SIGNATURE.len()
	{
		return io_error!(
			InvalidData, 
			format!("Can't open WebP file - Expected WEBP signature but found {}!", from_u8_vec_macro!(String, &file_buffer[8..12], &Endian::Big))
		);
	}

	return Ok(());
}

fn
check_byte_count
(
	file_buffer: &[u8],
	opt_file: Option<&File>
)
-> Result<(), std::io::Error>
{
	let byte_count = from_u8_vec_macro!(
		u32, 
		&file_buffer[4..8],
		&Endian::Little
	) + 8;

	if let Some(file) = opt_file
	{
		if file.metadata()?.len() != byte_count as u64
		{
			return io_error!(InvalidData, "Can't open WebP file - Promised byte count does not correspond with file size!");
		}	
	}
	else
	{
		if file_buffer.len() != byte_count as usize
		{
			return io_error!(InvalidData, format!("Can't handle WebP file buffer - Promised byte count {} does not correspond with file buffer length {}!", byte_count, file_buffer.len()));
		}
	}

	return Ok(());
}

fn
encode_metadata_webp
(
	exif_vec: &[u8]
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
	general_encoded_metadata: &[u8]
)
-> Vec<u8>
{
	encode_metadata_webp(general_encoded_metadata)
}