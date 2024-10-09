use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main() 
{
	let path_orig = Path::new("rsrc/image.JPG");
	let path_copy = Path::new("rsrc/copy.JPG");

	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file(&path_copy)
	{
		println!("{}", error);
	}
	copy(&path_orig, &path_copy);

	let metadata1 = Metadata::new_from_path(&path_orig).unwrap();

	for tag in metadata1.data()
	{
		println!("{:?}", tag);
	}
	println!("COUNTED {} EXIF TAGS", metadata1.data().len());

	let mut metadata2 = metadata1.clone();
	metadata2.set_tag(
		ExifTag::RecommendedExposureIndex(vec![2024]) // works
	);

	// Write metadata to file
	metadata2.write_to_file(path_copy);

	// Then read it again
	let metadata3 = Metadata::new_from_path(path_copy).unwrap();

	// Read metadata from file
	for tag1 in metadata1.data()
	{
		let mut found = false;
		for tag2 in metadata3.data()
		{
			if tag1 == tag2
			{
				found = true;
			}
		}

		if !found
		{
			println!("COULD NOT FIND {:?}", tag1);
		}
		// println!("{:?}", tag);
	}
	println!("COUNTED {} EXIF TAGS", metadata3.data().len());


}
