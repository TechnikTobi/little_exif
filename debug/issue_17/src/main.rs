use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main() 
{
	let jpg_path = Path::new("./rsrc/image1.tif");

	// Read metadata from file
	for tag in &Metadata::new_from_path(jpg_path).unwrap()
	{
		println!("{:?}", tag);
		if tag.is_unknown()
		{
			println!("The previous unknown tag 0x{:x} has a u8 vec with {} elements\n", tag.as_u16(), tag.value_as_u8_vec(&little_exif::endian::Endian::Little).len());
		}
	}
}
