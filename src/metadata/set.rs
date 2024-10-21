// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;

use super::Endian;
use super::ImageFileDirectory;
use super::Metadata;

impl
Metadata
{
	/// Sets the tag in the metadata struct. Tries to determine what IFD the 
	/// tag belongs to and should be inserted into, starting with IFD0.
	/// If the tag should e.g. be inserted into IFD0's EXIF SubIFD and that does
	/// not exist yet, the SubIFD gets created instead of trying to use the
	/// EXIF SubIFD of IFD1.
	/// For more fine-control (e.g. when handling multi-page TIFFs) it is 
	/// strongly advised to instead first get a mutable reference to the 
	/// preferred IFD and calling `set_tag` on that one instead. 
	pub fn
	set_tag
	(
		&mut self,
		input_tag: ExifTag
	)
	{
		self.get_ifd_mut(input_tag.get_group(), 0).set_tag(input_tag);
	}
}