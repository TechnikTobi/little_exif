// Copyright © 2022 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS

macro_rules! build_tag_enum {
	( 
		$( (
			$tag:ident, 
			$hex_value:expr,
			$format:expr
		) ),* 
	) 
	=>
	{
		#[derive(PartialEq, Debug)]
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
			format
			(
				&self
			)
			-> u16
			{
				match *self
				{
					$(
						ExifTag::$tag => $format,
					)*
				}
			}

			pub fn
			bytes_per_component
			(
				&self
			)
			-> u32
			{
				match self.format()
				{
					1  => 1,	// unsigned byte	int8u
					2  => 1,	// ascii string		string
					3  => 2,	// unsigned short	int16u
					4  => 4,	// unsigned long	int32u
					5  => 8,	// unsigned rational	rational64u
					6  => 1,	// signed byte		int8s
					7  => 1,	// undefined		undef
					8  => 2,	// signed short		int16s
					9  => 4,	// signed long		int32s
					10 => 8,	// signed rational	rational64s
					11 => 4,	// single float		float
					12 => 8,	// double float		double
					_ => panic!("Invalid EXIF tag format value!"),
				}
			}

		}

	};
}

build_tag_enum![
	// Tag				Tag ID	Format	
	(InteropIndex,			0x0001,	2),
	(ImageWidth,			0x0100,	4),
	(ImageHeight,			0x0101,	4),
	(BitsPerSample,			0x0102,	3),
	(Compression,			0x0103,	3),
	(PhotometricInterpretation,	0x0106,	3),
	(ImageDescription,		0x010e,	2),
	(Model,				0x0110,	2),
	(StripOffsets,			0x0111,	4), // not writable?
	(Orientation,			0x0112,	3)
];
