// Copyright © 2022 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS

use crate::exif_tag_value::ExifTagValue;

pub enum
ExifTagGroup
{
	NO_GROUP,
	All,
	ExifIFD,
	IFD0,
	IFD1,
	IFD2,
	InteropIFD,
	MakerNotes,
	SubIFD,
	SubIFD1,
	SubIFD2,
}

macro_rules! build_tag_enum {
	( 
		$( (
			$tag:ident, 
			$hex_value:expr,
			$format:expr,
			$writable:expr,
			$group:ident
		) ),* 
	) 
	=>
	{
		#[derive(Eq, PartialEq, Hash, Debug)]
		pub enum 
		ExifTag
		{
			$(
				$tag,
			)*
		}

		impl ExifTag
		{
			// Gets the hex value of an EXIF tag
			pub fn
			as_u16
			(
				&self
			)
			-> u16
			{
				match *self
				{
					$(
						ExifTag::$tag => $hex_value,
					)*
				}
			}

			// Gets the EXIF tag for a given hex value
			pub fn
			from_u16
			(
				hex_value: u16
			)
			-> Result<ExifTag, String>
			{
				match hex_value
				{
					$(
						$hex_value => Ok(ExifTag::$tag),
					)*
					_ => Err(String::from("Invalid hex value for EXIF tag")),
				}
			}

			// Gets the String representation of an EXIF tag
			pub fn
			as_string
			(
				&self
			)
			-> String
			{
				match *self
				{
					$(
						ExifTag::$tag => String::from(stringify!($tag)),
					)*
				}
			}

			pub fn
			is_writable
			(
				&self
			)
			-> bool
			{
				match *self
				{
					$(
						ExifTag::$tag => $writable,
					)*
				}
			}

			pub fn
			get_group
			(
				&self
			)
			-> ExifTagGroup
			{
				match *self
				{
					$(
						ExifTag::$tag => ExifTagGroup::$group,
					)*
				}
			}

			pub fn
			format
			(
				&self
			)
			-> u16
			{
				match *self
				{
					$(
						ExifTag::$tag => $format.format(),
					)*
				}
			}

			pub fn
			accepts
			(
				&self,
				value: &ExifTagValue
			)
			-> bool
			{
				return self.format() == value.format();
			}	
		}
	};
}

// This is just a small subset of the available EXIF tags
// Will be expanded in the future
// Note regarding non-writable tags: Apart from
// - StripOffsets
// - StripByteCounts
// - Opto-ElectricConvFactor
// - SpatialFrequencyResponse
// - DeviceSettingDescription
// none of them are part of the EXIF 2.32 specification
// (Source: https://exiftool.org/TagNames/EXIF.html )
build_tag_enum![
	// Tag						Tag ID	Format									Writable	Group
	(InteropIndex,				0x0001,	ExifTagValue::STRING("".to_string()),	true,		InteropIFD),
	(ImageWidth,				0x0100,	ExifTagValue::INT32U(0),				true,		IFD0),
	(ImageHeight,				0x0101,	ExifTagValue::INT32U(0),				true,		IFD0),
	(BitsPerSample,				0x0102,	ExifTagValue::INT16U(0),				true,		IFD0),
	(Compression,				0x0103,	ExifTagValue::INT16U(0),				true,		IFD0),
	(PhotometricInterpretation,	0x0106,	ExifTagValue::INT16U(0),				true,		IFD0),
	(ImageDescription,			0x010e,	ExifTagValue::STRING("".to_string()),	true,		IFD0),
	(Model,						0x0110,	ExifTagValue::STRING("".to_string()),	true,		IFD0),
	(StripOffsets,				0x0111,	ExifTagValue::INT32U(0),				false,		NO_GROUP),
	(Orientation,				0x0112,	ExifTagValue::INT32U(0),				true,		IFD0)
];
