#[derive(PartialEq, Debug)]
pub enum
ExifTagValue
{
	INT8U(u8),			// unsigned byte		int8u
	STRING(String),		// ascii string			string
	INT16U(u16),		// unsigned short		int16u
	INT32U(u32),		// unsigned long		int32u
	RATIONAL64U(u64),	// unsigned rational	rational64u		should this be u64?
	INT8S(i8),			// signed byte			int8s
	UNDEF,				// undefined			undef
	INT16S(i16),		// signed short			int16s
	INT32S(i32),		// signed long			int32s
	RATIONAL64S(i64),	// signed rational		rational64s		should this be i64?
	FLOAT(f32),			// single float			float
	DOUBLE(f64)			// double float			double
}

impl 
ExifTagValue
{

	pub fn
	format
	(
		&self
	)
	-> u16
	{
		match *self
		{
			ExifTagValue::INT8U(_)			=> 0x0001,
			ExifTagValue::STRING(_)			=> 0x0002,
			ExifTagValue::INT16U(_)			=> 0x0003,
			ExifTagValue::INT32U(_)			=> 0x0004,
			ExifTagValue::RATIONAL64U(_)	=> 0x0005,
			ExifTagValue::INT8S(_)			=> 0x0006,
			ExifTagValue::UNDEF				=> 0x0007,
			ExifTagValue::INT16S(_)			=> 0x0008,
			ExifTagValue::INT32S(_)			=> 0x0009,
			ExifTagValue::RATIONAL64S(_)	=> 0x000a,
			ExifTagValue::FLOAT(_)			=> 0x000b,
			ExifTagValue::DOUBLE(_)			=> 0x000c,
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
			0x0001	=> 1,
			0x0002	=> 1,
			0x0003	=> 2,
			0x0004	=> 4,
			0x0005	=> 8,
			0x0006	=> 1,
			0x0007	=> 1,
			0x0008	=> 2,
			0x0009	=> 4,
			0x000a	=> 8,
			0x000b	=> 4,
			0x000c	=> 8,
			_		=> panic!("Invalid format value for ExifTagValue!"),
		}
	}
}
