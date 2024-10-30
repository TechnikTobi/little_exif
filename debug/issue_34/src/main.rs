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
	// let source_file_path = "./rsrc/A0040453.JPG";
	let source_file_path = "./rsrc/dsc61076.jpg";

	let mut file_content = fs::read(&source_file_path)?; // .context(source_token.source.clone())?;
	let file_extension = get_file_type(Path::new(&source_file_path))?;
	let mut metadata = Metadata::new_from_vec(&file_content, file_extension)?;

	metadata.set_tag(ExifTag::ImageDescription("Hello from little_exif (again)!".to_string()));
	// metadata.write_to_vec(&mut file_content, file_extension)?;

	Metadata::clear_app13_segment(&mut file_content, file_extension)?;

	fs::write("./rsrc/copy.jpg", file_content)?;

	Ok(())
}
