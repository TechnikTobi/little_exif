// Copyright © 2022 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use paste::paste;

#[derive(Debug, PartialEq)]
pub enum
Endian
{
	Big,
	Little
}

impl Endian
{
	pub(crate) fn
	header
	(
		&self
	)
	-> [u8; 8]
	{
		match *self
		{
			Endian::Little	=> [0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00],
			Endian::Big		=> [0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x08],
		}
	}
}

pub(crate) trait
U8conversion<T>
{
	fn
	to_u8_vec
	(
		&self,
		endian: &Endian
	)
	-> Vec<u8>;

	fn
	from_u8_vec
	(
		u8_vec: &Vec<u8>,
		endian: &Endian
	)
	-> T;
}

macro_rules! build_u8conversion
{
	(
		$type:ty,
		$number_of_bytes:expr
	)
	=>
	{
		impl U8conversion<$type> for $type
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

			fn
			from_u8_vec
			(
				u8_vec: &Vec<u8>,
				endian: &Endian
			)
			-> $type
			{
				assert!(u8_vec.len() == $number_of_bytes);
				match *endian
				{
					Endian::Little => <paste!{[<$type>]}>::from_le_bytes(u8_vec[0..$number_of_bytes].try_into().unwrap()),
					Endian::Big => <paste!{[<$type>]}>::from_be_bytes(u8_vec[0..$number_of_bytes].try_into().unwrap()),
				}
			}
		}

		impl U8conversion<Vec<$type>> for Vec<$type>
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
					u8_vec.extend(<$type as U8conversion<$type>>::to_u8_vec(value, endian).iter());
				}
				return u8_vec;
			}

			fn
			from_u8_vec
			(
				u8_vec: &Vec<u8>,
				endian: &Endian
			)
			-> Vec<$type>
			{
				assert!(u8_vec.len() % $number_of_bytes == 0);

				let mut result: Vec<$type> = Vec::new();

				for i in 0..(u8_vec.len() / $number_of_bytes)
				{
					result.push(
						<$type>::from_u8_vec(
							&u8_vec[(0 + i*$number_of_bytes)..((i+1)*$number_of_bytes)].to_vec(), 
							endian
					) as $type);
				}

				return result;
			}
		}
	}
}

build_u8conversion![u8,		1];
build_u8conversion![i8,		1];
build_u8conversion![u16,	2];
build_u8conversion![i16,	2];
build_u8conversion![u32,	4];
build_u8conversion![i32,	4];
build_u8conversion![u64,	8];
build_u8conversion![i64,	8];
build_u8conversion![f32,	4];
build_u8conversion![f64,	8];

impl U8conversion<String> for String
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

	fn
	from_u8_vec
	(
		u8_vec: &Vec<u8>,
		_endian: &Endian
	)
	-> String
	{
		assert!(u8_vec.len() % 1 == 0);

		let mut result = String::new();

		for byte in u8_vec
		{
			if *byte > 0
			{
				result.push(*byte as char);
			}
		}

		return result;
	}
}

macro_rules! to_u8_vec_macro {
	($type:ty, $value:expr, $endian:expr)
	=>
	{
		<$type as U8conversion<$type>>::to_u8_vec($value, $endian)
	};
}

macro_rules! from_u8_vec_macro {
	($type:ty, $value:expr, $endian:expr)
	=>
	{
		<$type as U8conversion<$type>>::from_u8_vec($value, $endian)
	}
}

pub(crate) use to_u8_vec_macro;
pub(crate) use from_u8_vec_macro;