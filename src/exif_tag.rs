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

type INT8U			= Vec<u8>;
type STRING			= String;
type INT16U			= Vec<u16>;
type INT32U			= Vec<u32>;
type RATIONAL64U	= Vec<u64>; // ???
type INT8S			= Vec<i8>;
type UNDEF			= Vec<u8>;	// got no better idea for this atm
type INT16S			= Vec<i16>;
type INT32S			= Vec<i32>;
type RATIONAL64S	= Vec<i64>; // ???
type FLOAT			= Vec<f32>;
type DOUBLE			= Vec<f64>;

macro_rules! build_tag_enum {
	( 
		$( (
			$tag:ident, 
			$hex_value:expr,
			$format:ty,
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
				$tag($format),
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
						$hex_value => Ok(ExifTag::$tag(_)),
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

			/*
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
			components
			(
				&self
			)
			-> u32
			{
				match *self
				{
					$(
						ExifTag::$tag(_)
					)*
				}
			}
			*/
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
	// Tag						Tag ID	Format			Nr. Components	Writable	Group
	(InteroperabilityIndex,		0x0001,	STRING,			Some::<u32>(4),	true,		InteropIFD),
	(ImageWidth,				0x0100,	INT32U,			Some::<u32>(1),	true,		IFD0),
	(ImageHeight,				0x0101,	INT32U,			Some::<u32>(1),	true,		IFD0),
	(BitsPerSample,				0x0102,	INT16U,			Some::<u32>(3),	true,		IFD0),
	(Compression,				0x0103,	INT16U,			Some::<u32>(1),	true,		IFD0),
	(PhotometricInterpretation,	0x0106,	INT16U,			Some::<u32>(1),	true,		IFD0),
	(ImageDescription,			0x010e,	STRING,			None::<u32>,	true,		IFD0),
	(Model,						0x0110,	STRING,			None::<u32>,	true,		IFD0),
	(StripOffsets,				0x0111,	INT32U,			None::<u32>,	false,		NO_GROUP),
	(Orientation,				0x0112,	INT32U,			Some::<u32>(1),	true,		IFD0)
];
