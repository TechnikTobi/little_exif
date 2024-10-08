use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;

fn main() 
{
	let image_path = Path::new("rsrc/301581895-a7b4390a-e9f4-46cc-b04f-eb1ba677204c.jpg");
	let metadata = Metadata::new_from_path(&image_path).unwrap();

	for tag in metadata.data()
	{
		println!("{:?}", tag);
	}

}
