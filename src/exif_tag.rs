// Copyright © 2022 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS

use paste::paste;

use crate::endian::{U8conversion, Endian};
use crate::exif_tag_format::*;

#[derive(Debug, Eq, PartialEq, PartialOrd, Hash, Clone, Copy)]
pub enum
ExifTagGroup
{
	NO_GROUP,
	IFD0,
		ExifIFD,
			InteropIFD,
			MakerNotes,
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
		// #[derive(Eq, PartialEq, Hash, Debug)]
		#[derive(PartialEq, Debug)]
		pub enum 
		ExifTag
		{
			$(
				$tag(paste!{[<$format_enum>]}),
			)*
			UNKNOWN_STRING(		 STRING,		u16, ExifTagGroup),
			UNKNOWN_INT8U(		 INT8U,			u16, ExifTagGroup),
			UNKNOWN_INT16U(		 INT16U,		u16, ExifTagGroup),
			UNKNOWN_INT32U(		 INT32U,		u16, ExifTagGroup),
			UNKNOWN_RATIONAL64U( RATIONAL64U,	u16, ExifTagGroup),
			UNKNOWN_INT8S(		 INT8S,			u16, ExifTagGroup),
			UNKNOWN_UNDEF(		 UNDEF,			u16, ExifTagGroup),
			UNKNOWN_INT16S(		 INT16S,		u16, ExifTagGroup),
			UNKNOWN_INT32S(		 INT32S,		u16, ExifTagGroup),
			UNKNOWN_RATIONAL64S( RATIONAL64S,	u16, ExifTagGroup),
			UNKNOWN_FLOAT(		 FLOAT,			u16, ExifTagGroup),
			UNKNOWN_DOUBLE(		 DOUBLE,		u16, ExifTagGroup),
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
					ExifTag::UNKNOWN_STRING(		_, tag, _) => tag,
					ExifTag::UNKNOWN_INT8U(			_, tag, _) => tag,
					ExifTag::UNKNOWN_INT16U(		_, tag, _) => tag,
					ExifTag::UNKNOWN_INT32U(		_, tag, _) => tag,
					ExifTag::UNKNOWN_RATIONAL64U(	_, tag, _) => tag,
					ExifTag::UNKNOWN_INT8S(			_, tag, _) => tag,
					ExifTag::UNKNOWN_UNDEF(			_, tag, _) => tag,
					ExifTag::UNKNOWN_INT16S(		_, tag, _) => tag,
					ExifTag::UNKNOWN_INT32S(		_, tag, _) => tag,
					ExifTag::UNKNOWN_RATIONAL64S(	_, tag, _) => tag,
					ExifTag::UNKNOWN_FLOAT(			_, tag, _) => tag,
					ExifTag::UNKNOWN_DOUBLE(		_, tag, _) => tag,
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
						$hex_value => Ok(ExifTag::$tag(<paste!{[<$format_enum>]}>::new())),
					)*
					_ => Err(String::from("Invalid hex value for EXIF tag - Use 'UNKNOWN_...' instead")),
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
					_ => true,
				}
			}

			pub fn
			is_unknown
			(
				&self
			)
			-> bool
			{
				match *self
				{
					ExifTag::UNKNOWN_STRING(		_, _, _) => true,
					ExifTag::UNKNOWN_INT8U(			_, _, _) => true,
					ExifTag::UNKNOWN_INT16U(		_, _, _) => true,
					ExifTag::UNKNOWN_INT32U(		_, _, _) => true,
					ExifTag::UNKNOWN_RATIONAL64U(	_, _, _) => true,
					ExifTag::UNKNOWN_INT8S(			_, _, _) => true,
					ExifTag::UNKNOWN_UNDEF(			_, _, _) => true,
					ExifTag::UNKNOWN_INT16S(		_, _, _) => true,
					ExifTag::UNKNOWN_INT32S(		_, _, _) => true,
					ExifTag::UNKNOWN_RATIONAL64S(	_, _, _) => true,
					ExifTag::UNKNOWN_FLOAT(			_, _, _) => true,
					ExifTag::UNKNOWN_DOUBLE(		_, _, _) => true,
					_ => false
				}
			}

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
					ExifTag::UNKNOWN_STRING(		_, _, group) => group,
					ExifTag::UNKNOWN_INT8U(			_, _, group) => group,
					ExifTag::UNKNOWN_INT16U(		_, _, group) => group,
					ExifTag::UNKNOWN_INT32U(		_, _, group) => group,
					ExifTag::UNKNOWN_RATIONAL64U(	_, _, group) => group,
					ExifTag::UNKNOWN_INT8S(			_, _, group) => group,
					ExifTag::UNKNOWN_UNDEF(			_, _, group) => group,
					ExifTag::UNKNOWN_INT16S(		_, _, group) => group,
					ExifTag::UNKNOWN_INT32S(		_, _, group) => group,
					ExifTag::UNKNOWN_RATIONAL64S(	_, _, group) => group,
					ExifTag::UNKNOWN_FLOAT(			_, _, group) => group,
					ExifTag::UNKNOWN_DOUBLE(		_, _, group) => group,
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
					ExifTag::UNKNOWN_STRING(		_, _, _) => ExifTagFormat::STRING,
					ExifTag::UNKNOWN_INT8U(			_, _, _) => ExifTagFormat::INT8U,
					ExifTag::UNKNOWN_INT16U(		_, _, _) => ExifTagFormat::INT16U,
					ExifTag::UNKNOWN_INT32U(		_, _, _) => ExifTagFormat::INT32U,
					ExifTag::UNKNOWN_RATIONAL64U(	_, _, _) => ExifTagFormat::RATIONAL64U,
					ExifTag::UNKNOWN_INT8S(			_, _, _) => ExifTagFormat::INT8S,
					ExifTag::UNKNOWN_UNDEF(			_, _, _) => ExifTagFormat::UNDEF,
					ExifTag::UNKNOWN_INT16S(		_, _, _) => ExifTagFormat::INT16S,
					ExifTag::UNKNOWN_INT32S(		_, _, _) => ExifTagFormat::INT32S,
					ExifTag::UNKNOWN_RATIONAL64S(	_, _, _) => ExifTagFormat::RATIONAL64S,
					ExifTag::UNKNOWN_FLOAT(			_, _, _) => ExifTagFormat::FLOAT,
					ExifTag::UNKNOWN_DOUBLE(		_, _, _) => ExifTagFormat::DOUBLE,
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
							return value.len() as u32 + self.is_string() as u32;
						},
					)*
					ExifTag::UNKNOWN_STRING(		value, _, _) => value.len() as u32 + 1,
					ExifTag::UNKNOWN_INT8U(			value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_INT16U(		value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_INT32U(		value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_RATIONAL64U(	value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_INT8S(			value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_UNDEF(			value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_INT16S(		value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_INT32S(		value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_RATIONAL64S(	value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_FLOAT(			value, _, _) => value.len() as u32,
					ExifTag::UNKNOWN_DOUBLE(		value, _, _) => value.len() as u32,
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
					ExifTag::UNKNOWN_STRING(_, _, _) => true,
					_ => false,
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
					ExifTag::UNKNOWN_STRING(		value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_INT8U(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_INT16U(		value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_INT32U(		value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_RATIONAL64U(	value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_INT8S(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_UNDEF(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_INT16S(		value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_INT32S(		value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_RATIONAL64S(	value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_FLOAT(			value, _, _) => value.to_u8_vec(endian),
					ExifTag::UNKNOWN_DOUBLE(		value, _, _) => value.to_u8_vec(endian),
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

	(FileSource,				0xa301,	UNDEF,			None::<u32>,		true,		ExifIFD),
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
