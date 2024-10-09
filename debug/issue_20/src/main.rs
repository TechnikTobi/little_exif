use std::fs::read;
use std::path::Path;

use little_exif::metadata::Metadata;

fn main() 
{
	let jpg_path = Path::new("./rsrc/2017_emilio_meister_IMG_4436.JPG");

	// Read metadata from file
	for tag in Metadata::new_from_path(jpg_path).unwrap().data()
	{
		println!("{:?}", tag);
	}

	// Read metadata from vec
	let image_data = read(&jpg_path).unwrap();
	for tag in Metadata::new_from_vec(&image_data, little_exif::filetype::FileExtension::JPEG).unwrap().data()
	{
		println!("{:?}", tag);
	}
}
