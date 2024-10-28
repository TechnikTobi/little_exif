// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Seek;
use std::io::Read;
use std::io::Write;

use crate::general_file_io::EXIF_HEADER;
use crate::metadata::Metadata;

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
	// Does *not* call generic_clear_metadata, as the entire tiff data gets
	// overwritten anyways
	cursor.write_all(&metadata.encode()?)?;

	return Ok(());
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
