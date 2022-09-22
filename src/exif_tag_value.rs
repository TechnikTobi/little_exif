pub enum
ExifTagValue
{
	INT8U(u8),
	STRING(String),
	INT16U(u16),
	INT32U(u32),
	RATIONAL64U(u64),	// ???
	INT8S(i8),
	UNDEF,
	INT16S(i16),
	INT32S(i32),
	RATIONAL64S(i64),	// ??
	FLOAT(f32),
	DOUBLE(f64)
}

impl 
ExifTagValue
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
			ExifTagValue::INT8U(_)		=> 0x0001,
			ExifTagValue::STRING(_)		=> 0x0002,
			ExifTagValue::INT16U(_)		=> 0x0003,
			ExifTagValue::INT32U(_)		=> 0x0004,
			ExifTagValue::RATIONAL64U(_)	=> 0x0005,
			ExifTagValue::INT8S(_)		=> 0x0006,
			ExifTagValue::UNDEF		=> 0x0007,
			ExifTagValue::INT16S(_)		=> 0x0008,
			ExifTagValue::INT32S(_)		=> 0x0009,
			ExifTagValue::RATIONAL64S(_)	=> 0x000a,
			ExifTagValue::FLOAT(_)		=> 0x000b,
			ExifTagValue::DOUBLE(_)		=> 0x000c,
		}
	}

	pub fn
	bytes_per_component
	(
		&self
	)
	-> u32
	{
		match *self
		{
			ExifTagValue::INT8U(_)		=> 1,	// unsigned byte	int8u
			ExifTagValue::STRING(_)		=> 1,	// ascii string		string
			ExifTagValue::INT16U(_)		=> 2,	// unsigned short	int16u
			ExifTagValue::INT32U(_)		=> 4,	// unsigned long	int32u
			ExifTagValue::RATIONAL64U(_)	=> 8,	// unsigned rational	rational64u
			ExifTagValue::INT8S(_)		=> 1,	// signed byte		int8s
			ExifTagValue::UNDEF		=> 1,	// undefined		undef
			ExifTagValue::INT16S(_)		=> 2,	// signed short		int16s
			ExifTagValue::INT32S(_)		=> 4,	// signed long		int32s
			ExifTagValue::RATIONAL64S(_)	=> 8,	// signed rational	rational64s
			ExifTagValue::FLOAT(_)		=> 4,	// single float		float
			ExifTagValue::DOUBLE(_)		=> 8,	// double float		double
		}
	}
}
