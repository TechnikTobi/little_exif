// Copyright © 2022 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

#[test]
fn
new()
{
	let _ = Metadata::new();
}

#[test]
fn
new_from_path()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample2.png"));
}

#[test]
#[should_panic(expected = "Can't read Metadata - File does not exist!")]
fn
new_from_path_panic_not_existant()
{
	let _ = Metadata::new_from_path(Path::new("sample2.png")).unwrap();
}

#[test]
#[should_panic(expected = "Can't get extension from given path!")]
fn
new_from_path_panic_no_extension()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample0")).unwrap();
}

#[test]
#[should_panic(expected = "Can't read Metadata - Unsupported file type!")]
fn
new_from_path_panic_not_supported()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample1.txt")).unwrap();
}



fn
from_u8_vec_to_u32_le
(
	data: &Vec<u8>
)
-> u32
{
	let mut result = 0;
	for i in 0..std::cmp::min(4, data.len())
	{
		result = result + (data[i] as u32) * 256u32.pow(i as u32);
	}
	return result;
}

#[test]
fn
read_from_file_webp()
-> Result<(), std::io::Error>
{
	let raw_metadata = Metadata::new_from_path(Path::new("tests/read_sample.webp"));
	if raw_metadata.is_err()
	{
		panic!();
	}

	let metadata = raw_metadata.unwrap();

	if let Some(iso_tag) = metadata.get_tag(&ExifTag::ISO(vec![0]))
	{
		assert_eq!(from_u8_vec_to_u32_le(&iso_tag.value_as_u8_vec(&little_exif::endian::Endian::Little)), 2706);
	}
	else
	{
		panic!("Could not read ISO tag!")
	}

	Ok(())
}



#[test]
fn
clear_metadata_webp()
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/read_sample_no_exif.webp")
	{
		println!("{}", error);
	}
	copy("tests/read_sample.webp", "tests/read_sample_no_exif.webp")?;

	// Clear the metadata
	little_exif::webp::clear_metadata(Path::new("tests/read_sample_no_exif.webp"))?;

	Ok(())
}



#[test]
fn 
write_to_file() 
-> Result<(), std::io::Error>
{

	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_copy.png")
	{
		println!("{}", error);
	}
	copy("tests/sample2.png", "tests/sample2_copy.png")?;

	// Create new metadata struct and fill it
	let mut metadata = Metadata::new();
	assert_eq!(metadata.data().len(), 0);

	metadata.set_tag(
		ExifTag::ImageDescription("Hello World!".to_string())
	);
	metadata.set_tag(
		ExifTag::ExposureProgram(vec![1])
	);
	metadata.set_tag(
		ExifTag::ISO(vec![2706])
	);
	metadata.set_tag(
		ExifTag::Model("Testcam(1)".to_string())
	);
	assert_eq!(metadata.data().len(), 4);
	
	// Write metadata to file
	assert!(metadata.write_to_file(Path::new("tests/sample2_copy.png")).is_ok());

	Ok(())
}

