//! # little_exif
//! A small crate for reading and writing (at least some) EXIF data written entirely in Rust. Currently supports only .png and .jp(e)g files and a few dozen tags in IFD0 and ExifIFD.
//! 
//! Interaction is done via the [`Metadata`](metadata/struct.Metadata.html) struct and the [`ExifTag`](exif_tag/enum.ExifTag.html) enum.
//!
//! # Usage
//! ## Write EXIF data
//! ```rust
//! // what use statements are needed here?
//! 
//! let mut exif_data = Metadata::new();
//! data.set_tag(
//!     ExifTag::ImageDescription("Hello World!".to_string())
//! );
//! exif_data.write_to_file(Path::new("image.png"));
//! ```

#![forbid(unsafe_code)]
#![crate_type = "lib"]
#![crate_name = "little_exif"]

mod endian;
mod exif_tag_format;
mod general_file_io;

mod png;
mod png_chunk;
mod jpg;

pub mod exif_tag;
pub mod metadata;

#[cfg(test)]
mod tests {

	use std::path::Path;
	use crate::png::parse_png;

    #[test]
	fn test_two() {

		if let Ok(chunks) = parse_png(Path::new("test.png"))
		{
			assert_eq!(chunks.len(), 3);
		}
		else
		{
			panic!("could not parse png file");
		}

	}

	use std::fs::copy;
	use std::fs::remove_file;
	use crate::metadata::Metadata;
	use crate::exif_tag::ExifTag;

	#[test]
	fn test_three() {

		remove_file("copy.png");
		copy("test.png", "copy.png");

		let mut data = Metadata::new();
		
		data.set_tag(
			ExifTag::ImageDescription("Hello World!SomeMoreTextBlaBlaBla".to_string())
		);
		
		data.set_tag(
			ExifTag::ExposureProgram(vec![1])
		);
		data.set_tag(
			ExifTag::ISO(vec![1308])
		);
		data.set_tag(
			ExifTag::ImageWidth(vec!(4))
		);
		data.set_tag(
			ExifTag::ImageHeight(vec!(6))
		);
		data.set_tag(
			ExifTag::Model("Testcam(1)".to_string())
		);
		data.set_tag(
			ExifTag::ImageUniqueID("12345".to_string())
		);
		data.write_to_file(Path::new("copy.png"));

	}
}
