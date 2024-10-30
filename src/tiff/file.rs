// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;

use crate::general_file_io::open_read_file;
use crate::general_file_io::open_write_file;
use crate::metadata::Metadata;

use super::generic_read_metadata;
use super::generic_write_metadata;

pub(crate) fn
read_metadata
(
	path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
	let mut buffered_file = BufReader::new(open_read_file(path)?);
	return generic_read_metadata(&mut buffered_file);
}

pub(crate) fn
clear_metadata
(
	path: &Path
)
-> Result<(), std::io::Error>
{
	// Read in the data
	let     raw_data = generic_read_metadata(&mut BufReader::new(open_read_file(path)?));
	let mut data     = Metadata::general_decoding_wrapper(raw_data)?;

	// Remove all IFDs that aren't required
	data.reduce_to_a_minimum();

	// Write the reduced data back to the backup cursor
	generic_write_metadata(&mut BufWriter::new(open_write_file(path)?), &data)?;

	return Ok(());
}

pub(crate) fn 
write_metadata
(
	path:     &Path,
	metadata: &Metadata
)
-> Result<(), std::io::Error>
{
	let mut buffered_file = BufWriter::new(open_write_file(path)?);
	return generic_write_metadata(&mut buffered_file, metadata);
}