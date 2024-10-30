use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main()
{
	let no_exif_data_found_path = Path::new("./rsrc/2017_stockholm_emilio.jpg");
	let wrong_format_path       = Path::new("./rsrc/2017_aachen_abendhimmel_emilio.jpg");
	let buffer_problem_path     = Path::new("./rsrc/2017_isernhagen_sorento_1.jpg");

	// // Read metadata from file
	// for tag in &Metadata::new_from_path(no_exif_data_found_path).unwrap()
	// {
	// 	println!("{:?}", tag);
	// }

	// // Read metadata from file
	// for tag in &Metadata::new_from_path(wrong_format_path).unwrap()
	// {
	// 	println!("{:?}", tag);
	// }

	// // Read metadata from file
	// for tag in &Metadata::new_from_path(buffer_problem_path).unwrap()
	// {
	// 	println!("{:?}", tag);
	// }

}
