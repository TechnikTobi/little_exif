// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::path::Path;

use crate::general_file_io::open_read_file;
use crate::general_file_io::EXIF_HEADER;

pub(crate) fn
read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	let mut file = open_read_file(path)?;
	let mut buffer = Vec::new();
	file.read_to_end(&mut buffer)?;
	let mut tiff_with_exif_header = Vec::new();
	tiff_with_exif_header.extend(EXIF_HEADER);
	tiff_with_exif_header.append(&mut buffer);

	return Ok(tiff_with_exif_header);
}