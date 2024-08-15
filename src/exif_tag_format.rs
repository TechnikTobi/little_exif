// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::rational::*;

pub type INT8U          = Vec<u8>;
pub type STRING         = String;
pub type INT16U         = Vec<u16>;
pub type INT32U         = Vec<u32>;
pub type RATIONAL64U    = Vec<r64u>;
pub type INT8S          = Vec<i8>;
pub type UNDEF          = Vec<u8>;      // got no better idea for this atm
pub type INT16S         = Vec<i16>;
pub type INT32S         = Vec<i32>;
pub type RATIONAL64S    = Vec<r64i>;
pub type FLOAT          = Vec<f32>;
pub type DOUBLE         = Vec<f64>;

#[derive(Debug, PartialEq)]
pub enum
ExifTagFormat
{
	INT8U,          // unsigned byte        int8u
	STRING,         // ascii string         string
	INT16U,         // unsigned short       int16u
	INT32U,         // unsigned long        int32u
	RATIONAL64U,    // unsigned rational    rational64u     should this be u64?
	INT8S,          // signed byte          int8s
	UNDEF,          // undefined            undef
	INT16S,         // signed short         int16s
	INT32S,         // signed long          int32s
	RATIONAL64S,    // signed rational      rational64s     should this be i64?
	FLOAT,          // single float         float
	DOUBLE          // double float         double
}

impl 
ExifTagFormat
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
			ExifTagFormat::INT8U        => 0x0001,
			ExifTagFormat::STRING       => 0x0002,
			ExifTagFormat::INT16U       => 0x0003,
			ExifTagFormat::INT32U       => 0x0004,
			ExifTagFormat::RATIONAL64U  => 0x0005,
			ExifTagFormat::INT8S        => 0x0006,
			ExifTagFormat::UNDEF        => 0x0007,
			ExifTagFormat::INT16S       => 0x0008,
			ExifTagFormat::INT32S       => 0x0009,
			ExifTagFormat::RATIONAL64S  => 0x000a,
			ExifTagFormat::FLOAT        => 0x000b,
			ExifTagFormat::DOUBLE       => 0x000c,
		}
	}

	pub fn
	from_u16
	(
		hex_code: u16
	)
	-> Option<ExifTagFormat>
	{
		match hex_code
		{
			0x0001  => Some(ExifTagFormat::INT8U),
			0x0002  => Some(ExifTagFormat::STRING),
			0x0003  => Some(ExifTagFormat::INT16U),
			0x0004  => Some(ExifTagFormat::INT32U),
			0x0005  => Some(ExifTagFormat::RATIONAL64U),
			0x0006  => Some(ExifTagFormat::INT8S),
			0x0007  => Some(ExifTagFormat::UNDEF),
			0x0008  => Some(ExifTagFormat::INT16S),
			0x0009  => Some(ExifTagFormat::INT32S),
			0x000a  => Some(ExifTagFormat::RATIONAL64S),
			0x000b  => Some(ExifTagFormat::FLOAT),
			0x000c  => Some(ExifTagFormat::DOUBLE),
			_       => None,
		}
	}


	pub fn
	bytes_per_component
	(
		&self
	)
	-> u32
	{
		match self.as_u16()
		{
			0x0001  => 1,
			0x0002  => 1,
			0x0003  => 2,
			0x0004  => 4,
			0x0005  => 8,
			0x0006  => 1,
			0x0007  => 1,
			0x0008  => 2,
			0x0009  => 4,
			0x000a  => 8,
			0x000b  => 4,
			0x000c  => 8,
			_       => panic!("Invalid format value for ExifTagFormat!"),
		}
	}
}
