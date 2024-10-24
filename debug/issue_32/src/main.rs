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
	/*
	let orig_path = Path::new("./rsrc/ds218351.jpg");

	// Read metadata from file
	for tag in &Metadata::new_from_path(orig_path).unwrap()
	{
		println!("{:?}", tag);
	}

	// Copy file
	remove_file("./rsrc/copy.jpg")?;
	copy("./rsrc/ds218351.jpg", "./rsrc/copy.jpg")?;
	let jpg_path = Path::new("./rsrc/copy.jpg");

	// Modify it
	let mut data = Metadata::new_from_path(orig_path).unwrap();
	data.set_tag(
		ExifTag::ImageDescription("Hallo Welt!".to_string())
	);

	data.write_to_file(&jpg_path);

	// Now read again
	for tag in &Metadata::new_from_path(jpg_path).unwrap()
	{
		println!("{:?}", tag);
	}
	*/

	let source_file_path = "./rsrc/ds218351.jpg";

	let mut file_content = fs::read(&source_file_path)?; // .context(source_token.source.clone())?;
	let file_extension = get_file_type(Path::new(&source_file_path))?;
	let mut metadata = Metadata::new_from_vec(&file_content, file_extension)?;

	metadata.set_tag(ExifTag::ImageDescription("test".to_string()));
	metadata.write_to_vec(&mut file_content, file_extension)?;

	fs::write("./rsrc/copy.jpg", file_content)?;

	Ok(())
}
