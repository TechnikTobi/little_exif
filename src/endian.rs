#[derive(PartialEq)]
pub enum
Endian
{
	Big,
	Little
}

pub trait
U8conversion
{
	fn
	to_u8_vec
	(
		&self,
		endian: &Endian
	)
	-> Vec<u8>;
}

macro_rules! build_u8conversion
{
	(
		$type:ty
	)
	=>
	{
		impl U8conversion for $type
		{
			fn
			to_u8_vec
			(
				&self,
				endian: &Endian
			)
			-> Vec<u8>
			{
				match *endian
				{
					Endian::Little => self.to_le_bytes().to_vec(),
					Endian::Big => self.to_be_bytes().to_vec(),
				}
			}
		}

		impl U8conversion for Vec<$type>
		{
			fn
			to_u8_vec
			(
				&self,
				endian: &Endian
			)
			-> Vec<u8>
			{
				let mut u8_vec = Vec::new();
				for value in self
				{
					// u8_vec.extend(value.to_u8_vec(endian).iter());
					u8_vec.extend(<$type as U8conversion>::to_u8_vec(value, endian).iter());
				}
				return u8_vec;
			}
		}
	}
}

build_u8conversion![u8];
build_u8conversion![i8];
build_u8conversion![u16];
build_u8conversion![i16];
build_u8conversion![u32];
build_u8conversion![i32];
build_u8conversion![u64];
build_u8conversion![i64];
build_u8conversion![f32];
build_u8conversion![f64];

impl U8conversion for String
{
	fn
	to_u8_vec
	(
		&self,
		_endian: &Endian
	)
	-> Vec<u8>
	{
		let mut u8_vec = self.as_bytes().to_vec();
		u8_vec.push(0x00 as u8);
		return u8_vec;
	}
}

