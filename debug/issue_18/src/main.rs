
use std::str::FromStr;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::filetype::FileExtension;

fn main() 
{
	let jpg_path = Path::new("./rsrc/x.jpg");

	// Read metadata from file
	for tag in Metadata::new_from_path(jpg_path).unwrap().data()
	{
		println!("{:?}", tag);
	}

	// Read metadata from vec
	let extension = jpg_path.extension().unwrap();
	let extension = extension.to_str().unwrap();
	let file_type = FileExtension::from_str(extension).unwrap();
	let mut content = std::fs::read(jpg_path).unwrap();
	let metadata = Metadata::new_from_vec(&content, file_type);
	for tag in metadata.unwrap().data()
	{
		println!("{:?}", tag);
	}
}
