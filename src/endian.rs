// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

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
			Endian::Little => [0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00],
			Endian::Big    => [0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x08],
		}
	}
}