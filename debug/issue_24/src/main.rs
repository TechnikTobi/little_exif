use std::fs;
use std::fs::copy;
use std::fs::remove_file;
use std::str::FromStr;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::get_file_type;

fn main()
-> Result<(), std::io::Error>
{
	
	let orig_path = Path::new("./rsrc/2019_Stockholm_Weihegold_6.jpg");

	// Read metadata from file
	for tag in &Metadata::new_from_path(orig_path).unwrap()
	{
		println!("{:?}", tag);
	}

	// Copy file
	remove_file("./rsrc/copy.jpg")?;
	copy("./rsrc/2019_Stockholm_Weihegold_6.jpg", "./rsrc/copy.jpg")?;
	let jpg_path = Path::new("./rsrc/copy.jpg");

	// Modify it
	let mut data = Metadata::new_from_path(orig_path).unwrap();
	data.set_tag(
		ExifTag::ImageDescription("Hello from little_exif!".to_string())
	);

	data.write_to_file(&jpg_path);

	println!("\nREAD AGAIN:");

	// Now read again
	for tag in &Metadata::new_from_path(jpg_path).unwrap()
	{
		println!("{:?}", tag);
	}

	Ok(())
}
