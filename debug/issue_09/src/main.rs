use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;

fn main() 
{
	let image_path = Path::new("./rsrc/image.jpeg");
	let mut metadata = Metadata::new_from_path(&image_path).unwrap();

	for tag in &metadata
	{
		println!("{:?}", tag);
	}

	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("./rsrc/sample_copy.jpg")
	{
		println!("{}", error);
	}
	copy("./rsrc/sample.jpg", "./rsrc/sample_copy.jpg");

	// Write metadata to file
	metadata.write_to_file(Path::new("./rsrc/sample_copy.jpg"));
}
