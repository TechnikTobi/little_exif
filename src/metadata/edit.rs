// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;
use crate::ifd::ExifTagGroup;

use super::Metadata;

impl
Metadata
{
	/// Reduces the `Metadata` struct to the absolute minimum required for 
	/// TIFF compliance without losing important data (see table in exif_tag.rs,
	/// strip and thumbnail data) which is all assumed to be in GENERIC IFDs.
	/// If this is not the case for one of your images, please open a new issue
	pub fn
	reduce_to_a_minimum
	(
		&mut self
	)
	{
		// Only keep GENERIC IFDs
		self.image_file_directories.retain(|ifd| ifd.get_ifd_type() == ExifTagGroup::GENERIC);

		// Remove tags in IFDs that are not important
		for ifd in self.image_file_directories.iter_mut()
		{
			let mut tags_to_be_removed = Vec::new();

			for tag in ifd.get_tags()
			{
				match tag
				{
					ExifTag::StripOffsets(_, _)
					| ExifTag::StripByteCounts(_, _)
					| ExifTag::ThumbnailOffset(_, _)
					| ExifTag::ThumbnailLength(_)
					| ExifTag::ImageWidth(_)
					| ExifTag::ImageHeight(_)
					| ExifTag::BitsPerSample(_)
					| ExifTag::Compression(_)
					| ExifTag::PhotometricInterpretation(_)
					| ExifTag::SamplesPerPixel(_)
					| ExifTag::RowsPerStrip(_)
					| ExifTag::XResolution(_)
					| ExifTag::YResolution(_)
					| ExifTag::ResolutionUnit(_)
					| ExifTag::ColorMap(_)
					=> (),

					_ 
					=> tags_to_be_removed.push(tag.clone()),
				}
			}

			for tag in tags_to_be_removed
			{
				ifd.remove_tag(tag);
			}
		}
	}
}