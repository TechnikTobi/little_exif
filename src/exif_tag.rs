// Copyright © 2022 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS

use crate::endian::{U8conversion, Endian};
use crate::exif_tag_format::*;

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
			$format_type:ty,
			$format_enum:ident,
			$component_number:expr,
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
				$tag($format_type),
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
						ExifTag::$tag(_) => $hex_value,
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
						$hex_value => Ok(ExifTag::$tag(<$format_type>::new())),
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
						ExifTag::$tag(_) => String::from(stringify!($tag)),
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
						ExifTag::$tag(_) => $writable,
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
						ExifTag::$tag(_) => ExifTagGroup::$group,
					)*
				}
			}

			
			pub fn
			format
			(
				&self
			)
			-> ExifTagFormat
			{
				match *self
				{
					$(
						ExifTag::$tag(_) => ExifTagFormat::$format_enum,
					)*
				}
			}

			pub fn
			number_of_components
			(
				&self
			)
			-> u32
			{
				match self
				{
					$(
						ExifTag::$tag(value) => {

							// Check if the value has a predefined number of components
							if $component_number.is_some()
							{
								return $component_number.unwrap() as u32;
							}

							// In case we have a string, return its length +1 for 0x00 at the end
							// Otherwise just the containers length of the container
							return value.len() as u32 + (ExifTagFormat::$format_enum == ExifTagFormat::STRING) as u32;
						},
					)*
				}
			}

			pub fn
			is_string
			(
				&self
			)
			-> bool
			{
				match *self
				{
					$(
						ExifTag::$tag(_) => (ExifTagFormat::$format_enum == ExifTagFormat::STRING),
					)*
				}
			}

			pub fn
			value_as_u8_vec
			(
				&self,
				endian: &Endian
			)
			-> Vec<u8>
			{
				match self
				{
					$(
						ExifTag::$tag(value) => value.to_u8_vec(endian),
					)*
				}
			}
		}
	};
}

// This is just a small subset of the available EXIF tags
// Will be expanded in the future
//
// Note regarding non-writable tags: Apart from
// - StripOffsets
// - StripByteCounts
// - Opto-ElectricConvFactor
// - SpatialFrequencyResponse
// - DeviceSettingDescription
// none of them are part of the EXIF 2.32 specification
// (Source: https://exiftool.org/TagNames/EXIF.html )
//
// The format of a tag has to be inserted twice, once as type and the time as enum variant

build_tag_enum![
	// Tag						Tag ID	FormatType		ExifTagFormat	Nr. Components	Writable	Group
	(InteroperabilityIndex,		0x0001,	STRING,			STRING,			Some::<u32>(4),	true,		InteropIFD),
	(ImageWidth,				0x0100,	INT32U,			INT32U,			Some::<u32>(1),	true,		IFD0),
	(ImageHeight,				0x0101,	INT32U,			INT32U,			Some::<u32>(1),	true,		IFD0),
	(BitsPerSample,				0x0102,	INT16U,			INT16U,			Some::<u32>(3),	true,		IFD0),
	(Compression,				0x0103,	INT16U,			INT16U,			Some::<u32>(1),	true,		IFD0),
	(PhotometricInterpretation,	0x0106,	INT16U,			INT16U,			Some::<u32>(1),	true,		IFD0),
	(ImageDescription,			0x010e,	STRING,			STRING,			None::<u32>,	true,		IFD0),
	(Model,						0x0110,	STRING,			STRING,			None::<u32>,	true,		IFD0),
	(StripOffsets,				0x0111,	INT32U,			INT32U,			None::<u32>,	false,		NO_GROUP),
	(Orientation,				0x0112,	INT32U,			INT32U,			Some::<u32>(1),	true,		IFD0)
];
