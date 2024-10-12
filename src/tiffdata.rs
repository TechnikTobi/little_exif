// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;

use crate::endian::*;
use crate::general_file_io::io_error;
use crate::ifd::ImageFileDirectory;
use crate::u8conversion::from_u8_vec_macro;
use crate::u8conversion::U8conversion;

#[derive(Clone)]
pub struct
Tiffdata
{
	endian: Endian,
}

impl
Tiffdata
{
	pub(crate) fn
	generic_decode_data
	(
		data_cursor: &mut Cursor<Vec<u8>>
	)
	// -> Result<(Endian, Vec<ImageFileDirectory>), std::io::Error>
	-> Result<(), std::io::Error>
	{
		// Determine endian
		let mut endian_buffer = vec![0u8; 2];
		data_cursor.read_exact(&mut endian_buffer)?;

		let endian = match endian_buffer[0..2]
		{
			[0x49, 0x49] => { Endian::Little },
			[0x4d, 0x4d] => { Endian::Big },
			_            => { return io_error!(Other, "Illegal endian information!") } 
		};

		// Validate magic number
		let mut magic_number_buffer = vec![0u8; 2];
		data_cursor.read_exact(&mut magic_number_buffer)?;
		if !(
			(endian == Endian::Little && magic_number_buffer == [0x2a, 0x00]) ||
			(endian == Endian::Big    && magic_number_buffer == [0x00, 0x2a])
		)
		{
			return io_error!(Other, "Could not verify magic number!");
		}

		// Get offset to IFD0
		let mut ifd0_offset_buffer = vec![0u8; 4];
		data_cursor.read_exact(&mut ifd0_offset_buffer)?;
		let mut ifd_offset_option = Some(from_u8_vec_macro!(u32, &ifd0_offset_buffer.to_vec(), &endian));

		// Decode all the IFDs
		let mut ifds = Vec::new();
		loop
		{
			if let Some(ifd_offset) = ifd_offset_option
			{
				data_cursor.set_position(pos);

				decode_ifd(
					data_cursor:         &mut Cursor<Vec<u8>>,
					data_begin_position:      usize,                                        // Stays the same for all calls to this function while decoding
					endian:              &    Endian,
					group:               &    ExifTagGroup,
					generic_ifd_nr:           u32,                                          // Reuse value for recursive calls; only gets incremented by caller
					insert_into:         &mut Vec<ImageFileDirectory>,                      // Stays the same for all calls to this function while decoding
				)
			}
			else
			{
				break;
			}
		}



		Ok(())
	}



}


#[cfg(test)]
mod tests 
{
	use std::fs::read;
	use std::io::Cursor;

use super::Tiffdata;

	#[test]
	fn
	new_test_1()
	-> Result<(), std::io::Error>
	{
		let image_data = read("tests/read_sample.tif").unwrap();

		Tiffdata::generic_decode_data(&mut Cursor::new(image_data))?;

		Ok(())
	}
}
