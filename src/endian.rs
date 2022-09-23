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


impl U8conversion for u8
{
	fn
	to_u8_vec
	(
		&self,
		endian: &Endian
	)
	-> Vec<u8>
	{
		vec![*self]
	}
}

impl U8conversion for i8
{
	fn
	to_u8_vec
	(
		&self,
		endian: &Endian
	)
	-> Vec<u8>
	{
		vec![(*self) as u8]
	}
}

impl U8conversion for u16
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
			Endian::Little => vec![
				*self as u8, 
				(*self >> 8) as u8
			],
			Endian::Big => vec![
				(*self >> 8) as u8,
				*self as u8
			],
		}
	}
}

impl U8conversion for i16
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
			Endian::Little => vec![
				*self as u8, 
				(*self >> 8) as u8
			],
			Endian::Big => vec![
				(*self >> 8) as u8,
				*self as u8
			],
		}
	}
}

impl U8conversion for u32
{
	fn
	to_u8_vec
	(
		&self,
		endian: &Endian
	)
	-> Vec<u8>
	{
		let mut u8_vec: Vec<u8> = Vec::new();

		for i in 0..4
		{
			u8_vec.push((*self >> (8 * (if *endian == Endian::Big {3-i} else {i} ) )) as u8);
		}

		return u8_vec;
	}
}

impl U8conversion for String
{
	fn
	to_u8_vec
	(
		&self,
		endian: &Endian
	)
	-> Vec<u8>
	{
		let mut u8_vec = self.as_bytes().to_vec();
		u8_vec.push(0x00 as u8);
		return u8_vec;
	}
}

