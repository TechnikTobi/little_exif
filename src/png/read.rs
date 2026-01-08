// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::general_file_io::io_error;
use crate::util::read_4_bytes;
use crate::util::read_be_u32;

/// Assumes the cursor to be positioned at the start of the chunk where the
/// length field is located.
/// The function call advances the cursor by 4 bytes, which is where the 
/// chunk type field is located.
pub(super) fn
read_chunk_length
<T: Seek + Read>
(
	cursor: &mut T
)
-> Result<u32, std::io::Error>
{
	return read_be_u32(cursor);
}

/// Assumes the cursor to be positioned at the start of the chunk's name field.
/// The function call advances the cursor by 4 bytes, which is where the 
/// chunk data is located.
pub(super) fn
read_chunk_name
<T: Seek + Read>
(
	cursor: &mut T
)
-> Result<String, std::io::Error>
{
	let field = read_4_bytes(cursor)?;
	let name  = String::from_utf8(field.to_vec()).unwrap_or_default();
	return Ok(name);
}

/// Assumes the cursor to be positioned at the start of the chunk data
/// Advances the cursor to the start of the chunk's CRC field
pub(super) fn
read_chunk_data
<T: Seek + Read>
(
	cursor:       &mut T,
	chunk_length: usize,
)
-> Result<Vec<u8>, std::io::Error>
{
	let mut chunk_data_buffer = vec![0u8; chunk_length];
	let     bytes_read        = cursor.read(&mut chunk_data_buffer)?;
	
	if bytes_read != chunk_length
	{
		return io_error!(Other, "Could not read chunk data");
	}

	return Ok(chunk_data_buffer);
}

/// Assumes the cursor to be positioned at the start of the chunk CRC field
/// Advances the cursor by 4 bytes (start of next chunk)
pub(super) fn
read_chunk_crc
<T: Seek + Read>
(
	cursor: &mut T
)
-> Result<[u8; 4], std::io::Error>
{
	let field = read_4_bytes(cursor)?;
	return Ok(field);
}