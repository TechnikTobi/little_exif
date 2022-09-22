pub enum 
ExifTag
{
	InteropIndex,
	ImageWidth,
	ImageHeight,
	BitsPerSample,
	Compression,
	PhotometricInterpretation,
        ImageDescription,
	Model,
	StripOffsets,
	Orientation,
	SamplesPerPixel,
	RowsPerStrip,
	StripByteCounts,
	XResolution,
	YResolution,
}

impl ExifTag
{
        pub fn
        as_u16
        (
                &self
        )
        -> u16
        {
                match *self
                {
			ExifTag::InteropIndex =>		0x0001,
			ExifTag::ImageWidth =>			0x0100,
			ExifTag::ImageHeight => 		0x0101,
			ExifTag::BitsPerSample =>	 	0x0102,
			ExifTag::Compression => 		0x0103,
			ExifTag::PhotometricInterpretation => 	0x0106,
                        ExifTag::ImageDescription => 		0x010e,
			ExifTag::Model => 			0x0110,
			ExifTag::StripOffsets => 		0x0111,
			ExifTag::Orientation => 		0x0112,
			ExifTag::SamplesPerPixel => 		0x0115,
			ExifTag::RowsPerStrip => 		0x0116,
			ExifTag::StripByteCounts => 		0x0117,
			ExifTag::XResolution => 		0x011a,
			ExifTag::YResolution => 		0x011b,
			_ => todo!()
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
                        ExifTag::ImageDescription => 0x0002,
			_ => todo!()
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
                        1  => 1,
                        2  => 1,
                        3  => 2,
                        4  => 4,
                        5  => 8,
                        6  => 1,
                        7  => 1,
                        8  => 2,
                        9  => 4,
                        10 => 8,
                        11 => 4,
                        12 => 8,
                        _  => panic!("Invalid ExifTag format value"),
                }
        }
}
