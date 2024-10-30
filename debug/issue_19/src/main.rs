use std::fs::copy;
use std::fs::read;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main() 
{
	let jpg_path = Path::new("./rsrc/2019_Stuttgart_Emilio_SL_925-014-126.JPG");

	// Read metadata from file
	for tag in &Metadata::new_from_path(jpg_path).unwrap()
	{
		println!("{:?}", tag);
	}

	// Read metadata from vec
	let image_data = read(&jpg_path).unwrap();
	for tag in &Metadata::new_from_vec(&image_data, little_exif::filetype::FileExtension::JPEG).unwrap()
	{
		println!("{:?}", tag);
	}
}
