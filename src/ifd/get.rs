// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;

use super::ExifTagGroup;
use super::ImageFileDirectory;

impl
ImageFileDirectory
{

	pub fn
	get_tags
	(
		&self
	)
	-> &Vec<ExifTag>
	{
		return &self.tags;
	}

	pub fn
	get_generic_ifd_nr
	(
		&self
	)
	-> u32
	{
		return self.belongs_to_generic_ifd_nr;
	}

	pub fn
	get_ifd_type
	(
		&self
	)
	-> ExifTagGroup
	{
		return self.ifd_type;
	}

	pub fn
	get_offset_tag_for_parent_ifd
	(
		&self
	)
	-> Option<(ExifTagGroup, ExifTag)>
	{
		match self.ifd_type
		{
			ExifTagGroup::GENERIC  => None,
			ExifTagGroup::EXIF     => Some((ExifTagGroup::GENERIC, ExifTag::ExifOffset(   Vec::new()))),
			ExifTagGroup::GPS      => Some((ExifTagGroup::GENERIC, ExifTag::GPSInfo(      Vec::new()))),
			ExifTagGroup::INTEROP  => Some((ExifTagGroup::EXIF,    ExifTag::InteropOffset(Vec::new()))),
		}
	}

	pub fn
	get_ifd_type_for_offset_tag
	(
		tag: &ExifTag
	)
	-> Option<ExifTagGroup>
	{
		match tag
		{
			ExifTag::ExifOffset(_)    => Some(ExifTagGroup::EXIF),
			ExifTag::GPSInfo(_)       => Some(ExifTagGroup::GPS),
			ExifTag::InteropOffset(_) => Some(ExifTagGroup::INTEROP),
			_ => None
		}
	}

}