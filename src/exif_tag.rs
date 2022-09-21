pub enum 
ExifTag
{
	InteropIndex,
	ImageWidth,
	ImageHeight,
	BitsPerSample,
	Compression
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
	PlanarConfiguration,
	ResolutionUnit,
	TransferFunction,
	Software,
	ModifyDate, // Called DateTime by EXIF spec
	Artist,
	
	// ExifIFD
	SubjectDistanceRange,
	ImageUniqueID,
	OwnerName,
	SerialNumber,
	LensInfo,
	LensMake,
	LensModel,
	LensSerialNumber,
	CompositeImage,
	CompositeImageCount,
	CompositeImageExposureTimes,
	Gamma
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
                        ExifTag::ImageDescription => 0x010e,
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
