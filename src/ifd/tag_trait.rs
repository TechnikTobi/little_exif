// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub trait 
Tag 
{
	fn draw(&self);
}

pub enum
ExifTag
{
	ImageDescription,
}

pub enum
GpsTag
{
	GPSVersionID,
}

impl
Tag
for
ExifTag
{
	fn
	draw
	(
		&self
	)
	{
		todo!();
	}
}