use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;

fn main() 
-> Result<(), std::io::Error>
{
	// let path_string = "./rsrc/A0040275_5.tif";
	let path_string = "./rsrc/image1.tif";

	let path = Path::new(path_string);

	copy(path_string, "./rsrc/copy.tif")?;

	// Read metadata from file
	let mut data = Metadata::new_from_path(path).unwrap();

	// Read tags
	for tag in data.into_iter() {
		println!("{:x}", tag.as_u16());
	}

	// Set ImageDescription
	// data.set_tag(ExifTag::ImageDescription("Hello World!".to_string()));

	// Write back to copy
	return data.write_to_file(Path::new("./rsrc/copy.tif"));
}
