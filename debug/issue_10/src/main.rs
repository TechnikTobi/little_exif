use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main() 
{
	let jpg_path = Path::new("./rsrc/copy.jpg");

	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file(&jpg_path)
	{
		println!("{}", error);
	}
	copy("./rsrc/image.jpg", "./rsrc/copy.jpg");

	// let mut metadata = Metadata::new_from_path(&jpg_path).unwrap();
	let mut metadata = Metadata::new();

	metadata.set_tag(
		ExifTag::FNumber(vec![2024.into()])                  // FAILS
		// ExifTag::RecommendedExposureIndex(vec![2024]) // works
		// ExifTag::SpectralSensitivity("awoooo wie viel text geht hier rein?".to_string()) // works
	);

	// Write metadata to file
	metadata.write_to_file(Path::new("./rsrc/copy.jpg"));

	// Read metadata from file
	for tag in Metadata::new_from_path(jpg_path).unwrap().data()
	{
		println!("{:?}", tag);
	}
}
