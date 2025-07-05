// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::endian::Endian;
use crate::general_file_io::io_error;
use crate::ifd::ExifTagGroup;

use super::ExifTag;
use super::ExifTagFormat;
use super::U8conversion;
use super::INT8U;
use super::INT16U;
use super::INT32U;


pub(crate) fn
decode_tag_with_format_exceptions
(
	raw_tag:  &ExifTag,
	format:    ExifTagFormat,
	raw_data: &Vec<u8>,
	endian:   &Endian,
	hex_tag:   u16,
	group:    &ExifTagGroup
)
-> Result<ExifTag, std::io::Error>
{
	if raw_tag.format().as_u16() != format.as_u16()
	{
		// The expected format and the given format in the file
		// do *not* match. Check special cases (e.g. INT16U -> INT32U)
		// If no special cases match, return an error
		match (raw_tag.format(), format.clone())
		{
			// Expected for tag   VS Decoded from data
			(ExifTagFormat::INT32U, ExifTagFormat::INT16U) => {
				let int16u_data = <INT16U as U8conversion<INT16U>>::from_u8_vec(raw_data, endian);
				let int32u_data = int16u_data.into_iter().map(|x| x as u32).collect::<Vec<u32>>();
				return Ok(raw_tag.set_value_to_int32u_vec(int32u_data).unwrap());
			},

			(ExifTagFormat::INT32U, ExifTagFormat::INT8U) => {
				let int8u_data  = <INT8U as U8conversion<INT8U>>::from_u8_vec(raw_data, endian);
				let int32u_data = int8u_data.into_iter().map(|x| x as u32).collect::<Vec<u32>>();
				return Ok(raw_tag.set_value_to_int32u_vec(int32u_data).unwrap());
			},

			(ExifTagFormat::INT16U, ExifTagFormat::INT32U) => {
				// Not sure how to be more cautious in this case...
				let int32u_data = <INT32U as U8conversion<INT32U>>::from_u8_vec(raw_data, endian);
				let int16u_data = int32u_data.into_iter().map(|x| x as u16).collect::<Vec<u16>>();
				return Ok(raw_tag.set_value_to_int16u_vec(int16u_data).unwrap());
			},

			(ExifTagFormat::INT16U, ExifTagFormat::INT8U) => {
				let int8u_data  = <INT8U as U8conversion<INT8U>>::from_u8_vec(raw_data, endian);
				let int16u_data = int8u_data.into_iter().map(|x| x as u16).collect::<Vec<u16>>();
				return Ok(raw_tag.set_value_to_int16u_vec(int16u_data).unwrap());
			},

			(ExifTagFormat::INT8U, ExifTagFormat::STRING) => {
				if 
					raw_tag.as_u16()    == 0x0005            && // GPSAltitudeRef
					raw_tag.get_group() == ExifTagGroup::GPS
				{
					// The GPSAltitudeRef tag is a strange case. It is the only
					// GPS -Ref tag that is a INT8U, all others are STRINGs
					// with a length of two. 
					// Some images store this as a string nevertheless. 
					// So, we try to convert the string by taking its first
					// character. If it is 0x00 or 0x30 ("0") we set it to 0,
					// if it is 0x01 or 0x31 ("1") we set it to 1, and
					// otherwise we panic and tell the user to open a ticket.

					let first_char = raw_data[0];
					let int8u_data = match first_char
					{
						0x00 | 0x30 => vec![0u8],
						0x01 | 0x31 => vec![1u8],
						_ => panic!("Problem while decoding GPSAltitudeRef. Please open a new issue for little_exif!")
					};

					return Ok(ExifTag::from_u16_with_data(
						0x0005, 
						&ExifTagFormat::INT8U, 
						&int8u_data, 
						&endian, 
						group
					).unwrap());
				}
				else
				{
					return io_error!(Other, format!("Unknown tag for combination INT8U vs STRING while decoding: {:?}", raw_tag));
				}
			},

			// See issue #63
			(ExifTagFormat::UNDEF, ExifTagFormat::STRING) => {
				if 
					raw_tag.as_u16()    == 0x001b            && // GPSProcessingMethod	
					raw_tag.get_group() == ExifTagGroup::GPS
				{
					return Ok(raw_tag.set_value_to_undef(raw_data.to_vec()).unwrap());
				}
				else
				{
					return io_error!(Other, format!("Unknown tag for combination UNDEF vs STRING while decoding: {:?}", raw_tag));
				}
			}

			_ => {
				return io_error!(Other, format!("Illegal format for known tag! Tag: {:?} Expected: {:?} Got: {:?}", raw_tag, raw_tag.format(), format));
			},
		};
	}
	else
	{
		// Format is as expected; set the data by replacing the tag
		return Ok(ExifTag::from_u16_with_data(
			hex_tag, 
			&format, 
			&raw_data, 
			&endian, 
			group
		).unwrap());
	}
}