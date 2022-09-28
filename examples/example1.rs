use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn
main()
{

	// Remove old copy and create new one for writing EXIF data to
	remove_file("examples/copy.png");
	copy("examples/image.png", "examples/copy.png");

	// Create a new Metadata struct
	let mut data = Metadata::new();

	// Set the ImageDescription (IFD0) an ISO (ExifIFD) tag as examples
	data.set_tag(
		ExifTag::ImageDescription("-w 1000 -h 1000 --x_mid=0 --y_mid=0 -z 0.5 -i 1000 -c 8".to_string())
	);
	data.set_tag(
		ExifTag::ISO(vec![2022])
	);

	// Write the metadata to the copy
	let write_result = data.write_to_file(Path::new("examples/copy.png"));

	if write_result.is_err()
	{
		println!("{}", write_result.err().unwrap());
	}
}
