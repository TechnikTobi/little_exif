#[derive(PartialEq)]
pub enum
Endian
{
	Big,
	Little
}

pub trait
U8conversion<T>
{
	fn
	to_u8_vec
	(
		&self,
		endian: &Endian
	)
	-> Vec<u8>;
}

impl<T: U8conversion<T>> U8conversion<T> for Vec<T>
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
			u8_vec.extend(value.to_u8_vec(endian).iter());
		}
		return u8_vec;
	}
}

impl<T> U8conversion<T> for u8
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

impl<T> U8conversion<T> for i8
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

impl<T> U8conversion<T> for u16
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

impl<T> U8conversion<T> for i16
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

impl<T> U8conversion<T> for u32
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

impl<T> U8conversion<T> for i32
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

impl<T> U8conversion<T> for u64
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

impl<T> U8conversion<T> for i64
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

impl<T> U8conversion<T> for f32
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

impl<T> U8conversion<T> for f64
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

impl<T> U8conversion<T> for String
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

