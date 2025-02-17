use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main() 
{
	let png1_path = Path::new("./rsrc/test1.png");
	let png2_path = Path::new("./rsrc/test2.png");
	let png3_path = Path::new("./rsrc/test3.png");

	for tag in &Metadata::new_from_path(png1_path).unwrap()
	{
		println!("{:?}", tag);
	}

	println!("");

	for tag in &Metadata::new_from_path(png2_path).unwrap()
	{
		println!("{:?}", tag);
	}

	println!("");

	for tag in &Metadata::new_from_path(png3_path).unwrap()
	{
		println!("{:?}", tag);
	}

	/*
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
		// ExifTag::SpectralSensitivity("how much text can I fit in here?".to_string()) // works
	);

	// Write metadata to file
	metadata.write_to_file(Path::new("./rsrc/copy.jpg"));

	// Read metadata from file
	for tag in &Metadata::new_from_path(jpg_path).unwrap()
	{
		println!("{:?}", tag);
	}
	*/
}
