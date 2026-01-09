// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;
use crate::ifd::ExifTagGroup;
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

	/// Removes a tag from the metadata struct, based on its hex value and 
	/// associated group. If, for whatever reason, this tag appears in multiple 
	/// IFDs, all instances will be removed, assuming the groups match. 
	/// The count of calls on `remove_tag` gets returned. If this is zero,
	/// no removals were performed.
	#[allow(clippy::needless_pass_by_value)]
	pub fn
	remove_tag
	(
		&mut self,
		remove_me: ExifTag
	)
	-> usize
	{
		self.remove_tag_by_hex_group(
			remove_me.as_u16(),
			remove_me.get_group()
		)
	}

	/// Removes a tag from the metadata struct, based on its hex value and
	/// associated group. If, for whatever reason, this tag appears in multiple
	/// IFDs, all instances will be removed, assuming the groups match.
	/// The count of calls on `remove_tag` gets returned. If this is zero,
	/// no removals were performed.
	#[allow(clippy::needless_pass_by_value)]
	pub fn
	remove_tag_by_hex_group
	(
		&mut self,
		tag_hex: u16,
		tag_group: ExifTagGroup
	)
		-> usize
	{
		let mut removed_count = 0;

		// Traverse all IFD numbers up to the max. known one
		for ifd_number in 0..=self.get_max_generic_ifd_number()
		{
			// Does this IFD exist?
			if self.get_ifd(tag_group, ifd_number).is_some()
			{
				// If so, get it as mutable and call remove_tag on it
				self.get_ifd_mut(
					tag_group, ifd_number
				).remove_tag(tag_hex);

				removed_count += 1;
			}
		}

		return removed_count;
	}
}