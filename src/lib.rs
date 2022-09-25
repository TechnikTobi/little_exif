mod endian;
mod exif_tag;
mod exif_tag_format;
mod metadata;

mod png;
mod png_chunk;

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
