// Copyright © 2022 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS

use paste::paste;

use crate::endian::{U8conversion, Endian};
use crate::exif_tag_format::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, PartialOrd, Hash, Clone, Copy)]
pub enum
ExifTagGroup
{
	NO_GROUP,
	IFD0,
		ExifIFD,
			InteropIFD,
			MakerNotesIFD,
		GPSIFD,
	IFD1,
}

macro_rules! build_tag_enum {
	( 
		$( (
			$tag:ident, 
			$hex_value:expr,
			$format_enum:ident,
			$component_number:expr,
			$writable:expr,
			$group:ident
		) ),* 
	) 
	=>
	{
		/// These are the currently supported tags by little_exif. 
		/// Note that for tags that are unknown at the moment a fallback
		/// solution is provided using the `Unknown...` variants. 
		#[derive(PartialEq, Debug)]
		pub enum 
		ExifTag
		{
			$(
				$tag(paste!{[<$format_enum>]}),
			)*
			UnknownINT8U(		INT8U,			u16, ExifTagGroup),
			UnknownSTRING(		STRING,			u16, ExifTagGroup),
			UnknownINT16U(		INT16U,			u16, ExifTagGroup),
			UnknownINT32U(		INT32U,			u16, ExifTagGroup),
			UnknownRATIONAL64U(	RATIONAL64U,	u16, ExifTagGroup),
			UnknownINT8S(		INT8S,			u16, ExifTagGroup),
			UnknownUNDEF(		UNDEF,			u16, ExifTagGroup),
			UnknownINT16S(		INT16S,			u16, ExifTagGroup),
			UnknownINT32S(		INT32S,			u16, ExifTagGroup),
			UnknownRATIONAL64S(	RATIONAL64S,	u16, ExifTagGroup),
			UnknownFLOAT(		FLOAT,			u16, ExifTagGroup),
			UnknownDOUBLE(		DOUBLE,			u16, ExifTagGroup),
		}

		impl ExifTag
		{
			/// Gets the hex value of an EXIF tag
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
					ExifTag::UnknownINT8U(			_, tag, _) => tag,
					ExifTag::UnknownSTRING(			_, tag, _) => tag,
					ExifTag::UnknownINT16U(			_, tag, _) => tag,
					ExifTag::UnknownINT32U(			_, tag, _) => tag,
					ExifTag::UnknownRATIONAL64U(	_, tag, _) => tag,
					ExifTag::UnknownINT8S(			_, tag, _) => tag,
					ExifTag::UnknownUNDEF(			_, tag, _) => tag,
					ExifTag::UnknownINT16S(			_, tag, _) => tag,
					ExifTag::UnknownINT32S(			_, tag, _) => tag,
					ExifTag::UnknownRATIONAL64S(	_, tag, _) => tag,
					ExifTag::UnknownFLOAT(			_, tag, _) => tag,
					ExifTag::UnknownDOUBLE(			_, tag, _) => tag,
				}
			}

			/// Gets the tag for a given hex value. 
			/// The tag is initialized with new, empty data.
			/// If the hex value is unknown, an error is returned.
			/// 
			/// # Examples
			/// ```no_run
			/// use little_exif::exif_tag::ExifTag;
			/// 
			/// let tag = ExifTag::from_u16(0x010e).unwrap();
			/// ```
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
						$hex_value => Ok(ExifTag::$tag(<paste!{[<$format_enum>]}>::new())),
					)*
					_ => Err(String::from("Invalid hex value for EXIF tag - Use 'Unknown...' instead")),
				}
			}

			/// Gets the tag for a given hex value. 
			/// The tag is initalized using the given raw data by converting it
			/// to the appropriate format.
			/// If the hex value is unknown, the other parameters are used to
			/// generate an appropriate unkown tag for the specified format.
			/// 
			/// # Examples
			/// ```no_run
			/// use little_exif::endian::Endian;
			/// use little_exif::exif_tag::ExifTag;
			/// use little_exif::exif_tag::ExifTagGroup;
			/// use little_exif::exif_tag_format::ExifTagFormat;
			/// 
			/// let tag = ExifTag::from_u16_with_data(
			///     0x0113,                   // An unknown tag hex value
			///     &ExifTagFormat::INT8U,
			///     &vec![1u8],
			///     &Endian::Little,
			///     &ExifTagGroup::NO_GROUP
			/// );
			/// ```
			pub fn
			from_u16_with_data
			(
				hex_value: u16,
				format: &ExifTagFormat,
				raw_data: &Vec<u8>,
				endian: &Endian,
				group: &ExifTagGroup,
			)
			-> Result<ExifTag, String>
			{
				match hex_value
				{
					$(
						$hex_value => Ok(ExifTag::$tag(
							<paste!{[<$format_enum>]} as U8conversion<paste!{[<$format_enum>]}>>::from_u8_vec(raw_data, endian)
						)),
					)*
					_ => {
						// In this case, the given hex_value represents a tag that is unknown
						match *format
						{
							ExifTagFormat::INT8U		=> Ok(ExifTag::UnknownINT8U(		<INT8U			as U8conversion<INT8U>>::from_u8_vec(raw_data, endian),			hex_value, *group)),
							ExifTagFormat::STRING		=> Ok(ExifTag::UnknownSTRING(		<STRING			as U8conversion<STRING>>::from_u8_vec(raw_data, endian),		hex_value, *group)),
							ExifTagFormat::INT16U		=> Ok(ExifTag::UnknownINT16U(		<INT16U			as U8conversion<INT16U>>::from_u8_vec(raw_data, endian),		hex_value, *group)),
							ExifTagFormat::INT32U		=> Ok(ExifTag::UnknownINT32U(		<INT32U			as U8conversion<INT32U>>::from_u8_vec(raw_data, endian),		hex_value, *group)),
							ExifTagFormat::RATIONAL64U	=> Ok(ExifTag::UnknownRATIONAL64U(	<RATIONAL64U	as U8conversion<RATIONAL64U>>::from_u8_vec(raw_data, endian),	hex_value, *group)),
							ExifTagFormat::INT8S		=> Ok(ExifTag::UnknownINT8S(		<INT8S			as U8conversion<INT8S>>::from_u8_vec(raw_data, endian),			hex_value, *group)),
							ExifTagFormat::UNDEF		=> Ok(ExifTag::UnknownUNDEF(		<UNDEF			as U8conversion<UNDEF>>::from_u8_vec(raw_data, endian),			hex_value, *group)),
							ExifTagFormat::INT16S		=> Ok(ExifTag::UnknownINT16S(		<INT16S			as U8conversion<INT16S>>::from_u8_vec(raw_data, endian),		hex_value, *group)),
							ExifTagFormat::INT32S		=> Ok(ExifTag::UnknownINT32S(		<INT32S			as U8conversion<INT32S>>::from_u8_vec(raw_data, endian),		hex_value, *group)),
							ExifTagFormat::RATIONAL64S	=> Ok(ExifTag::UnknownRATIONAL64S(	<RATIONAL64S	as U8conversion<RATIONAL64S>>::from_u8_vec(raw_data, endian),	hex_value, *group)),
							ExifTagFormat::FLOAT		=> Ok(ExifTag::UnknownFLOAT(		<FLOAT			as U8conversion<FLOAT>>::from_u8_vec(raw_data, endian),			hex_value, *group)),
							ExifTagFormat::DOUBLE		=> Ok(ExifTag::UnknownDOUBLE(		<DOUBLE			as U8conversion<DOUBLE>>::from_u8_vec(raw_data, endian),		hex_value, *group)),
							
						}
					},
				}
			}

			/// Gives information about whether the data stored in the tag can
			/// be written to file.
			/// Needed e.g. for Offset tags where the given value is useless
			/// and needs to be computed during the write process.
			/// 
			/// # Examples
			/// ```no_run
			/// use little_exif::exif_tag::ExifTag;
			/// 
			/// let writable = ExifTag::ImageDescription(String::new());
			/// let not_writable = ExifTag::ExifOffset(vec![1u32]);
			///
			/// assert_eq!(writable.is_writable(), true);
			/// assert_eq!(not_writable.is_writable(), false);
			/// ```
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
					_ => true,
				}
			}

			/// Checks if the tag is known to little_exif or not
			/// Note that in the future the value returned by this function 
			/// for a specific tag might change as the number of known tags
			/// gets increased
			pub fn
			is_unknown
			(
				&self
			)
			-> bool
			{
				match *self
				{
					ExifTag::UnknownINT8U(			_, _, _) => true,
					ExifTag::UnknownSTRING(			_, _, _) => true,
					ExifTag::UnknownINT16U(			_, _, _) => true,
					ExifTag::UnknownINT32U(			_, _, _) => true,
					ExifTag::UnknownRATIONAL64U(	_, _, _) => true,
					ExifTag::UnknownINT8S(			_, _, _) => true,
					ExifTag::UnknownUNDEF(			_, _, _) => true,
					ExifTag::UnknownINT16S(			_, _, _) => true,
					ExifTag::UnknownINT32S(			_, _, _) => true,
					ExifTag::UnknownRATIONAL64S(	_, _, _) => true,
					ExifTag::UnknownFLOAT(			_, _, _) => true,
					ExifTag::UnknownDOUBLE(			_, _, _) => true,
					_ => false
				}
			}

			/// Checks if the usage of an unknown tag for the tags hex value is
			/// justified or not
			pub fn
			unknown_is_justified
			(
				&self
			)
			-> bool
			{
				if self.is_unknown()
				{
					if let Ok(_) = Self::from_u16(self.as_u16())
					{
						return false;
					}
				}
				return true;
			}

			/// Gets the group (i.e. IFD) the tag belongs to.
			/// Note that this is still somewhat problematic, as for some tags
			/// the value of this function is hard to determine.
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
					ExifTag::UnknownINT8U(			_, _, group) => group,
					ExifTag::UnknownSTRING(			_, _, group) => group,
					ExifTag::UnknownINT16U(			_, _, group) => group,
					ExifTag::UnknownINT32U(			_, _, group) => group,
					ExifTag::UnknownRATIONAL64U(	_, _, group) => group,
					ExifTag::UnknownINT8S(			_, _, group) => group,
					ExifTag::UnknownUNDEF(			_, _, group) => group,
					ExifTag::UnknownINT16S(			_, _, group) => group,
					ExifTag::UnknownINT32S(			_, _, group) => group,
					ExifTag::UnknownRATIONAL64S(	_, _, group) => group,
					ExifTag::UnknownFLOAT(			_, _, group) => group,
					ExifTag::UnknownDOUBLE(			_, _, group) => group,
				}
			}

			/// Gets the format of the data for a tag (e.g. `STRING`, `INT8U`, ...)
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
					ExifTag::UnknownINT8U(			_, _, _) => ExifTagFormat::INT8U,
					ExifTag::UnknownSTRING(			_, _, _) => ExifTagFormat::STRING,
					ExifTag::UnknownINT16U(			_, _, _) => ExifTagFormat::INT16U,
					ExifTag::UnknownINT32U(			_, _, _) => ExifTagFormat::INT32U,
					ExifTag::UnknownRATIONAL64U(	_, _, _) => ExifTagFormat::RATIONAL64U,
					ExifTag::UnknownINT8S(			_, _, _) => ExifTagFormat::INT8S,
					ExifTag::UnknownUNDEF(			_, _, _) => ExifTagFormat::UNDEF,
					ExifTag::UnknownINT16S(			_, _, _) => ExifTagFormat::INT16S,
					ExifTag::UnknownINT32S(			_, _, _) => ExifTagFormat::INT32S,
					ExifTag::UnknownRATIONAL64S(	_, _, _) => ExifTagFormat::RATIONAL64S,
					ExifTag::UnknownFLOAT(			_, _, _) => ExifTagFormat::FLOAT,
					ExifTag::UnknownDOUBLE(			_, _, _) => ExifTagFormat::DOUBLE,
				}
			}

			/// Gets the number of components for the tag. This might be
			/// predefined by the specifications for some tags (e.g.
			/// `BitsPerSample` has to have 3 components of type `INT16U`), for
			/// other tags this depends on the actual data (e.g. most - but not
			/// all - `STRING` format type tags).
			/// Note that for `STRING` format type tags this includes the NUL
			/// terminator (which gets written automatically and should not be
			/// provided by the user). 
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
							return value.len() as u32 + self.is_string() as u32;
						},
					)*
					ExifTag::UnknownINT8U(			value, _, _) => value.len() as u32,
					ExifTag::UnknownSTRING(			value, _, _) => value.len() as u32 + 1,
					ExifTag::UnknownINT16U(			value, _, _) => value.len() as u32,
					ExifTag::UnknownINT32U(			value, _, _) => value.len() as u32,
					ExifTag::UnknownRATIONAL64U(	value, _, _) => value.len() as u32,
					ExifTag::UnknownINT8S(			value, _, _) => value.len() as u32,
					ExifTag::UnknownUNDEF(			value, _, _) => value.len() as u32,
					ExifTag::UnknownINT16S(			value, _, _) => value.len() as u32,
					ExifTag::UnknownINT32S(			value, _, _) => value.len() as u32,
					ExifTag::UnknownRATIONAL64S(	value, _, _) => value.len() as u32,
					ExifTag::UnknownFLOAT(			value, _, _) => value.len() as u32,
					ExifTag::UnknownDOUBLE(			value, _, _) => value.len() as u32,
				}
			}

			/// Checks if the format type of the tag is `STRING`.
			/// Needed for generating the EXIF data to know wheter to add a 
			/// NUL terminator at the end
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
					ExifTag::UnknownSTRING(_, _, _) => true,
					_ => false,
				}
			}

			/// Gets the value stored in the tag as an u8 vector, using the 
			/// given endianness for conversion.
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
					ExifTag::UnknownINT8U(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownSTRING(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownINT16U(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownINT32U(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownRATIONAL64U(	value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownINT8S(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownUNDEF(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownINT16S(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownINT32S(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownRATIONAL64S(	value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownFLOAT(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UnknownDOUBLE(			value, _, _) => value.to_u8_vec(endian),
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
// - Opto-ElectricConvFactor (OECF)
// - SpatialFrequencyResponse
// - DeviceSettingDescription
// none of them are part of the EXIF 2.32 specification
// (Source: https://exiftool.org/TagNames/EXIF.html )

build_tag_enum![
	// Tag						Tag ID	Format			Nr. Components		Writable	Group
	(InteroperabilityIndex,		0x0001,	STRING,			Some::<u32>(4),		true,		InteropIFD),

	(ImageWidth,				0x0100,	INT32U,			Some::<u32>(1),		true,		IFD0),		// IFD1?
	(ImageHeight,				0x0101,	INT32U,			Some::<u32>(1),		true,		IFD0),		// IFD1?
	(BitsPerSample,				0x0102,	INT16U,			Some::<u32>(3),		true,		IFD0),		// IFD1?
	(Compression,				0x0103,	INT16U,			Some::<u32>(1),		true,		IFD0),		// IFD1?

	(PhotometricInterpretation,	0x0106,	INT16U,			Some::<u32>(1),		true,		IFD0),		// IFD1?

	(ImageDescription,			0x010e,	STRING,			None::<u32>,		true,		IFD0),
	(Make,						0x010f,	STRING,			None::<u32>,		true,		IFD0),
	(Model,						0x0110,	STRING,			None::<u32>,		true,		IFD0),
	(StripOffsets,				0x0111,	INT32U,			None::<u32>,		false,		NO_GROUP),	// IFD1?
	(Orientation,				0x0112,	INT32U,			Some::<u32>(1),		true,		IFD0),

	(SamplesPerPixel,			0x0115,	INT16U,			Some::<u32>(1),		true,		IFD0),		// IFD1?
	(RowsPerStrip,				0x0116,	INT32U,			Some::<u32>(1),		true,		IFD0),		// IFD1?
	(StripByteCounts,			0x0117,	INT32U,			None::<u32>,		false,		NO_GROUP),	// IFD1?

	(XResolution,				0x011a,	RATIONAL64U,	Some::<u32>(1),		true,		IFD0),
	(YResolution,				0x011b,	RATIONAL64U,	Some::<u32>(1),		true,		IFD0),
	(PlanarConfiguration,		0x011c,	INT16U,			Some::<u32>(1),		true,		IFD0),		// IFD1?

	(ResolutionUnit,			0x0128,	INT16U,			Some::<u32>(1),		true,		IFD0),		// IFD1?

	(TransferFunction,			0x012d,	INT16U,			Some::<u32>(3),		true,		IFD0),

	(Software,					0x0131,	STRING,			None::<u32>,		true,		IFD0),
	(ModifyDate,				0x0132,	STRING,			Some::<u32>(20),	true,		IFD0),

	(Artist,					0x013b,	STRING,			None::<u32>,		true,		IFD0),

	(WhitePoint,				0x013e,	RATIONAL64U,	Some::<u32>(2),		true,		IFD0),
	(PrimaryChromaticities,		0x013f,	RATIONAL64U,	Some::<u32>(6),		true,		IFD0),

	(ThumbnailOffset,			0x0201,	INT32U,			Some::<u32>(1),		true,		IFD1),		// oh boy, this one seems complicated - the group depends on the file type???
	(ThumbnailLength,			0x0202,	INT32U,			Some::<u32>(1),		true,		IFD1),		// same problems as 0x0201

	(YCbCrCoefficients,			0x0211,	RATIONAL64U,	Some::<u32>(3),		true,		IFD0),		// IFD1?
	(YCbCrSubSampling,			0x0212,	INT16U,			Some::<u32>(2),		true,		IFD0),		// IFD1?
	(YCbCrPositioning,			0x0213,	INT16U,			Some::<u32>(1),		true,		IFD0),		// IFD1?
	(ReferenceBlackWhite,		0x0214,	RATIONAL64U,	Some::<u32>(6),		true,		IFD0),		// IFD1?

	(Copyright,					0x8298,	STRING,			None::<u32>,		true,		IFD0),
	(ExposureTime,				0x829a,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(FNumber,					0x829d,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),

	(ExifOffset,				0x8769,	INT32U,			Some::<u32>(1),		false,		IFD0),

	(ExposureProgram,			0x8822,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(SpectralSensitivity,		0x8824,	STRING,			None::<u32>,		true,		ExifIFD),
	(GPSInfo,					0x8825,	INT32U,			Some::<u32>(1),		true,		IFD0),		// -> GPS Tags: https://exiftool.org/TagNames/GPS.html
	(ISO,						0x8827,	INT16U,			Some::<u32>(2),		true,		ExifIFD),
	(OECF,						0x8828,	UNDEF,			None::<u32>,		false,		NO_GROUP),
	(SensitivityType,			0x8830,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(StandardOutputSensitivity,	0x8831,	INT32U,			Some::<u32>(1),		true,		ExifIFD),
	(RecommendedExposureIndex,	0x8832,	INT32U,			Some::<u32>(1),		true,		ExifIFD),
	(ISOSpeed,					0x8833,	INT32U,			Some::<u32>(1),		true,		ExifIFD),
	(ISOSpeedLatitudeyyy,		0x8834,	INT32U,			Some::<u32>(1),		true,		ExifIFD),
	(ISOSpeedLatitudezzz,		0x8835,	INT32U,			Some::<u32>(1),		true,		ExifIFD),

	(ExifVersion,				0x9000, UNDEF,			Some::<u32>(4),		true,		ExifIFD),	// 4 ASCII chars but without NULL Terminator
	(DateTimeOriginal,			0x9003,	STRING,			Some::<u32>(20),	true,		ExifIFD),
	(CreateDate,				0x9004,	STRING,			Some::<u32>(20),	true,		ExifIFD),
	(OffsetTime,				0x9010,	STRING,			None::<u32>,		true,		ExifIFD),
	(OffsetTimeOriginal,		0x9011,	STRING,			None::<u32>,		true,		ExifIFD),
	(OffsetTimeDigitized,		0x9012,	STRING,			None::<u32>,		true,		ExifIFD),

	(ComponentsConfiguration,	0x9101,	UNDEF,			None::<u32>,		true,		ExifIFD),
	(CompressedBitsPerPixel,	0x9102,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),

	(ShutterSpeedValue,			0x9201, RATIONAL64S,	Some::<u32>(1),		true,		ExifIFD),
	(ApertureValue,				0x9202, RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(BrightnessValue,			0x9203, RATIONAL64S,	Some::<u32>(1),		true,		ExifIFD),
	(ExposureCompensation,		0x9204, RATIONAL64S,	Some::<u32>(1),		true,		ExifIFD),
	(MaxApertureValue,			0x9205, RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(SubjectDistance,			0x9206, RATIONAL64S,	Some::<u32>(1),		true,		ExifIFD),
	(MeteringMode,				0x9207, INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(LightSource,				0x9208, INT16U,			Some::<u32>(1),		true,		ExifIFD),	// -> EXIF LightSource Values: https://exiftool.org/TagNames/EXIF.html#LightSource
	(Flash,						0x9209, INT16U,			Some::<u32>(1),		true,		ExifIFD),	// -> EXIF Flash Values: https://exiftool.org/TagNames/EXIF.html#Flash
	(FocalLength,				0x920a, RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),

	(SubjectArea,				0x9214,	INT16U,			Some::<u32>(4),		true,		ExifIFD),

	(MakerNote,					0x927c,	UNDEF,			None::<u32>,		true,		ExifIFD),
	(UserComment,				0x9286,	UNDEF,			None::<u32>,		true,		ExifIFD),	// First 8 bytes describe the character code (e.g. "JIS" for Japanese characters)
	(SubSecTime,				0x9290,	STRING,			None::<u32>,		true,		ExifIFD),
	(SubSecTimeOriginal,		0x9291,	STRING,			None::<u32>,		true,		ExifIFD),
	(SubSecTimeDigitized,		0x9292,	STRING,			None::<u32>,		true,		ExifIFD),

	(AmbientTemperature,		0x9400,	RATIONAL64S,	Some::<u32>(1),		true,		ExifIFD),
	(Humidity,					0x9401,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(Pressure,					0x9402,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(WaterDepth,				0x9403,	RATIONAL64S,	Some::<u32>(1),		true,		ExifIFD),
	(Acceleration,				0x9404,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(CameraElevationAngle,		0x9405,	RATIONAL64S,	Some::<u32>(1),		true,		ExifIFD),

	(FlashpixVersion,			0xa000,	UNDEF,			Some::<u32>(4),		true,		ExifIFD),
	(ColorSpace,				0xa001,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(ExifImageWidth,			0xa002,	INT32U, 		Some::<u32>(1),		true,		ExifIFD),
	(ExifImageHeight,			0xa003,	INT32U, 		Some::<u32>(1),		true,		ExifIFD),

	(RelatedSoundFile,			0xa004,	STRING,			None::<u32>,		true,		ExifIFD),
	(InteropOffset,				0xa005,	INT32U,			Some::<u32>(1),		true,		ExifIFD),
	(FlashEnergy,				0xa20b,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(SpatialFrequencyResponse,	0xa20c,	INT16U,			Some::<u32>(1),		false,		NO_GROUP),
	(FocalPlaneXResolution,		0xa20e,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(FocalPlaneYResolution,		0xa20f,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(FocalPlaneResolutionUnit,	0xa210,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(SubjectLocation,			0xa214,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(ExposureIndex,				0xa215,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),

	(SensingMethod,				0xa217,	INT16U,			Some::<u32>(1),		true,		ExifIFD),

	(FileSource,				0xa300,	UNDEF,			None::<u32>,		true,		ExifIFD),
	(SceneType,					0xa301,	UNDEF,			None::<u32>,		true,		ExifIFD),
	(CFAPattern,				0xa302,	UNDEF,			None::<u32>,		true,		ExifIFD),

	(CustomRendered,			0xa401,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(ExposureMode,				0xa402,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(WhiteBalance,				0xa403,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(DigitalZoomRatio,			0xa404,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD),
	(FocalLengthIn35mmFormat,	0xa405,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(SceneCaptureType,			0xa406,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(GainControl,				0xa407,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(Contrast,					0xa408,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(Saturation,				0xa409,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(Sharpness,					0xa40a,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(DeviceSettingDescription,	0xa40b,	UNDEF,			None::<u32>,		false,		NO_GROUP),

	(SubjectDistanceRange,		0xa40c,	INT16U,			Some::<u32>(1),		true,		ExifIFD),

	(ImageUniqueID,				0xa420,	STRING,			None::<u32>,		true,		ExifIFD),

	(OwnerName,					0xa430,	STRING,			None::<u32>,		true,		ExifIFD),
	(SerialNumber,				0xa431,	STRING,			None::<u32>,		true,		ExifIFD),
	(LensInfo,					0xa432,	RATIONAL64U,	Some::<u32>(4),		true,		ExifIFD),
	(LensMake,					0xa433,	STRING,			None::<u32>,		true,		ExifIFD),
	(LensModel,					0xa434,	STRING,			None::<u32>,		true,		ExifIFD),
	(LensSerialNumber,			0xa435,	STRING,			None::<u32>,		true,		ExifIFD),
	
	(CompositeImage,			0xa460,	INT16U,			Some::<u32>(1),		true,		ExifIFD),
	(CompositeImageCount,		0xa461,	INT16U,			Some::<u32>(2),		true,		ExifIFD),
	(CompositeImageExposureTimes,	0xa462,	UNDEF,		None::<u32>,		true,		ExifIFD),

	(Gamma,						0xa500,	RATIONAL64U,	Some::<u32>(1),		true,		ExifIFD)
];

impl ExifTag
{
	/// Checks if a tag is for representing the offset to a SubIFD (e.g. ExifIFD).
	/// Needed for generating the exif data for writing, as the value stored in 
	/// the tag variables is useless because it needs to be computed during
	/// the writing process.
	pub fn
	is_offset_tag
	(
		&self
	)
	-> Option<ExifTagGroup>
	{
		match *self
		{
			ExifTag::ExifOffset(_)		=> Some(ExifTagGroup::ExifIFD),
			ExifTag::GPSInfo(_)			=> Some(ExifTagGroup::GPSIFD),
			ExifTag::MakerNote(_)		=> Some(ExifTagGroup::MakerNotesIFD),
			ExifTag::InteropOffset(_)	=> Some(ExifTagGroup::InteropIFD),
			_ => None
		}
	}
}