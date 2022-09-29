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
	copy("examples/image.png", "examples/copy.png");

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
	let write_result = data.write_to_file(Path::new("examples/copy.png"));

	let encoded_data = data.encode_metadata_png();
	let decoded_data1 = Metadata::decode_metadata_png(&encoded_data);
	let decoded_data2 = Metadata::decode_metadata_general(&decoded_data1).unwrap();

	for tag in &decoded_data2
	{
		println!("{:?} {}", tag.get_group(), tag.is_unknown());
	}

	if write_result.is_err()
	{
		println!("{}", write_result.err().unwrap());
	}
}
