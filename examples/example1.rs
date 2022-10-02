use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::exif_tag::ExifTagGroup;

fn
main()
{

	// Remove old copy and create new one for writing EXIF data to
	remove_file("examples/copy.png");
	remove_file("examples/copy.jpg");
	copy("examples/image.png", "examples/copy.png");
	copy("examples/image.jpg", "examples/copy.jpg");

	// Create a new Metadata struct
	let mut data = Metadata::new();

	// Set the ImageDescription (IFD0) an ISO (ExifIFD) tag as examples

	data.set_tag(
		ExifTag::UnknownSTRING("test".to_string(), 0x010d, ExifTagGroup::IFD0)
	);

	data.set_tag(
		ExifTag::ImageDescription("-w 1000 -h 1000 --x_mid=0 --y_mid=0 -z 0.5 -i 1000 -c 8".to_string())
	);

	data.set_tag(
		ExifTag::ISO(vec![2022])
	);

	data.set_tag(
		ExifTag::UnknownSTRING("test".to_string(), 0x010c, ExifTagGroup::IFD0)
	);

	// Write the metadata to the copy
	if let Err(error) = data.write_to_file(Path::new("examples/copy.png"))
	{
		println!("{}", error);
	}
	else
	{
		let png_data = Metadata::new_from_path(Path::new("examples/copy.png"));
		println!("PNG read result:");
		
		for tag in png_data.get_data()
		{
			println!("{:?}", tag);
		}
	}

	if let Err(error) = data.write_to_file(Path::new("examples/copy.jpg"))
	{
		println!("{}", error);
	}
	else
	{
		let jpg_data = Metadata::new_from_path(Path::new("examples/copy.jpg"));
		println!("JPG read result:");

		for tag in jpg_data.get_data()
		{
			println!("{:?}", tag);
		}
	}
}