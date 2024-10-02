// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::copy;
use std::fs::read;
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
fn
new_from_path_no_data()
{
	let data = Metadata::new_from_path(Path::new("tests/no_exif.jxl")).unwrap();
	assert_eq!(data.data().len(), 0);
}


#[test]
#[should_panic(expected = "File does not exist!")]
fn
new_from_path_panic_not_existent()
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
#[should_panic(expected = "Unsupported file type!")]
fn
new_from_path_panic_not_supported()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample1.txt")).unwrap();
}

#[test]
fn
new_from_vec()
{
	let image_data = read("tests/read_sample.jpg").unwrap();

	let _ = Metadata::new_from_vec(&image_data, little_exif::filetype::FileExtension::JPEG).unwrap();
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
read_from_file_jxl()
-> Result<(), std::io::Error>
{
	let raw_metadata = Metadata::new_from_path(Path::new("tests/with_exif.jxl"));
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
read_from_vec_jpg()
-> Result<(), std::io::Error>
{
	let image_data = read("tests/read_sample.jpg").unwrap();

	let raw_metadata = Metadata::new_from_vec(&image_data, little_exif::filetype::FileExtension::JPEG);
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
read_from_vec_jxl()
-> Result<(), std::io::Error>
{
	let image_data = read("tests/with_exif.jxl").unwrap();

	let raw_metadata = Metadata::new_from_vec(&image_data, little_exif::filetype::FileExtension::JXL);
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




fn
get_test_metadata()
-> Result<Metadata, std::io::Error>
{
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

	return Ok(metadata);
}

#[test]
fn 
as_u8_vec_png()
{
	println!(
		"as_u8_vec_png: {}", 
		get_test_metadata()
			.unwrap()
			.as_u8_vec(little_exif::filetype::FileExtension::PNG { as_zTXt_chunk: false })
			.iter()
			.map(|char_value| *char_value as char)
			.into_iter()
			.collect::<String>()
	);
}

#[test]
#[allow(non_snake_case)]
fn 
as_u8_vec_png_zTXt()
{
	println!(
		"as_u8_vec_png_zTXt:             {}", 
		get_test_metadata()
			.unwrap()
			.as_u8_vec(little_exif::filetype::FileExtension::PNG { as_zTXt_chunk: true })
			.iter()
			.map(|char_value| *char_value as char)
			.into_iter()
			.collect::<String>()
	);
}

#[test]
fn
file_clear_metadata_jpg()
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_copy_no_metadata.jpg")
	{
		println!("{}", error);
	}
	copy("tests/sample2.jpg", "tests/sample2_copy_no_metadata.jpg")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;

	// Write metadata to file
	metadata.write_to_file(Path::new("tests/sample2_copy_no_metadata.jpg"))?;

	// Clear metadata
	Metadata::file_clear_metadata(Path::new("tests/sample2_copy_no_metadata.jpg"))?;

	Ok(())
}

#[test]
fn 
write_to_file_jpg() 
-> Result<(), std::io::Error>
{

	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_copy.jpg")
	{
		println!("{}", error);
	}
	copy("tests/sample2.jpg", "tests/sample2_copy.jpg")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;
	
	// Write metadata to file
	metadata.write_to_file(Path::new("tests/sample2_copy.jpg"))?;

	Ok(())
}

#[test]
fn 
write_to_file_png() 
-> Result<(), std::io::Error>
{

	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_copy.png")
	{
		println!("{}", error);
	}
	copy("tests/sample2.png", "tests/sample2_copy.png")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;
	
	// Write metadata to file
	metadata.write_to_file(Path::new("tests/sample2_copy.png"))?;

	Ok(())
}

#[ignore]
#[test]
fn 
write_to_file_webp_simple_lossy() 
-> Result<(), std::io::Error>
{
	// Currently not active as the "VP8 " converter does not exist yet!

	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_simple_lossy_copy.webp")
	{
		println!("{}", error);
	}
	copy("tests/sample2_simple_lossy.webp", "tests/sample2_simple_lossy_copy.webp")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;
	
	// Write metadata to file
	metadata.write_to_file(Path::new("tests/sample2_simple_lossy_copy.webp"))?;

	Ok(())
}

#[test]
fn 
write_to_file_webp_simple_lossless() 
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_simple_lossless_copy.webp")
	{
		println!("{}", error);
	}
	copy("tests/sample2_simple_lossless.webp", "tests/sample2_simple_lossless_copy.webp")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;
		
	// Write metadata to file
	metadata.write_to_file(Path::new("tests/sample2_simple_lossless_copy.webp"))?;

	Ok(())
}


#[test]
fn 
write_to_file_webp_extended() 
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_extended_copy.webp")
	{
		println!("{}", error);
	}
	copy("tests/sample2_extended.webp", "tests/sample2_extended_copy.webp")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;

	// Write metadata to file
	metadata.write_to_file(Path::new("tests/sample2_extended_copy.webp"))?;

	Ok(())
}