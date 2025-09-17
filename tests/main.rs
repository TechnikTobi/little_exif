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
new_from_path_no_data_jxl()
{
	let data = Metadata::new_from_path(Path::new("tests/no_exif.jxl"));
	assert!(data.is_err());
}

#[test]
fn
new_from_path_no_data_jpg()
{
	let data = Metadata::new_from_path(Path::new("tests/no_exif.jpeg"));
	assert!(data.is_err());
}

#[test]
#[should_panic(expected = "File does not exist!")]
fn
new_from_path_panic_not_existent()
{
	let _ = Metadata::new_from_path(Path::new("sample2.png")).unwrap();
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"Could not determine file type when reading file!\" }")]
fn
new_from_path_panic_no_extension()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample0")).unwrap();
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Unsupported, error: \"Unsupported file type: txt - Unknown file type: txt\" }")]
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

	if let Some(iso_tag) = metadata.get_tag(&ExifTag::ISO(vec![0])).next()
	{
		assert_eq!(from_u8_vec_to_u32_le(&iso_tag.value_as_u8_vec(&little_exif::endian::Endian::Little)), 2706);
	}
	else
	{
		panic!("Could not read ISO tag!")
	}

	if let Some(image_description_tag) = metadata.get_tag(&ExifTag::ImageDescription(String::new())).next()
	{
		assert_eq!(String::from_utf8(image_description_tag.value_as_u8_vec(&little_exif::endian::Endian::Little)).unwrap(), "Hello World!\0".to_string());
	}
	else
	{
		panic!("Could not read ImageDescription tag!")
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

	if let Some(iso_tag) = metadata.get_tag(&ExifTag::ISO(vec![0])).next()
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
read_from_file_tiff()
-> Result<(), std::io::Error>
{
	let raw_metadata = Metadata::new_from_path(Path::new("tests/read_sample.tif"));
	if raw_metadata.is_err()
	{
		panic!();
	}

	let metadata = raw_metadata.unwrap();

	if let Some(iso_tag) = metadata.get_tag(&ExifTag::ISO(vec![0])).next()
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
read_from_vec_generic
(
	image_data: &Vec<u8>,
	filetype: little_exif::filetype::FileExtension
)
-> Result<(), std::io::Error>
{
	let raw_metadata = Metadata::new_from_vec(&image_data, filetype);
	if raw_metadata.is_err()
	{
		panic!();
	}

	let metadata = raw_metadata.unwrap();

	// let mut found = false;
	// for tag in &metadata
	// {
	// 	if tag.as_u16() == ExifTag::ISO(vec![0]).as_u16()
	// 	{
	// 		// assert_eq!(from_u8_vec_to_u32_le(&tag.value_as_u8_vec(&metadata.get_endian())), 2706);
	// 		assert_eq!(from_u8_vec_to_u32_le(&tag.value_as_u8_vec(&little_exif::endian::Endian::Little)), 2706);
	// 		found = true;
	// 	}
	// }

	// assert!(found);

	
	if let Some(iso_tag) = metadata.get_tag(&ExifTag::ISO(vec![0])).next()
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
read_from_vec_webp()
-> Result<(), std::io::Error>
{
	return read_from_vec_generic(&read("tests/read_sample.webp").unwrap(), little_exif::filetype::FileExtension::WEBP);
}

#[test]
fn
read_from_vec_jpg()
-> Result<(), std::io::Error>
{
	return read_from_vec_generic(&read("tests/read_sample.jpg").unwrap(), little_exif::filetype::FileExtension::JPEG);
}

#[test]
fn
read_from_vec_jxl()
-> Result<(), std::io::Error>
{
	return read_from_vec_generic(&read("tests/with_exif.jxl").unwrap(), little_exif::filetype::FileExtension::JXL);
}




fn
get_test_metadata()
-> Result<Metadata, std::io::Error>
{
	// Create new metadata struct and fill it
	let mut metadata = Metadata::new();
	assert_eq!(metadata.into_iter().count(), 0);

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
	assert_eq!(metadata.into_iter().count(), 4);

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
			.unwrap()
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
			.unwrap()
			.iter()
			.map(|char_value| *char_value as char)
			.into_iter()
			.collect::<String>()
	);
}

#[test]
fn
clear_metadata_jxl()
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample_copy_no_metadata.jxl")
	{
		println!("{}", error);
	}
	copy("tests/with_exif.jxl", "tests/sample_copy_no_metadata.jxl")?;

	let mut image_data = read("tests/sample_copy_no_metadata.jxl").unwrap();

	// Clear metadata
	Metadata::clear_metadata(&mut image_data, little_exif::filetype::FileExtension::JXL)?;

	std::fs::write("tests/sample_copy_no_metadata.jxl", image_data)?;

	Ok(())
}

#[test]
fn
file_clear_metadata_jxl()
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample_copy_no_metadata2.jxl")
	{
		println!("{}", error);
	}
	copy("tests/with_exif.jxl", "tests/sample_copy_no_metadata2.jxl")?;

	// Clear metadata
	Metadata::file_clear_metadata(Path::new("tests/sample_copy_no_metadata2.jxl"))?;

	Ok(())
}

#[test]
fn
file_write_and_clear_metadata_jpg()
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
file_clear_metadata_png()
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/sample2_copy_no_metadata.png")
	{
		println!("{}", error);
	}
	copy("tests/sample2.png", "tests/sample2_copy_no_metadata.png")?;

	// Clear metadata
	Metadata::file_clear_metadata(Path::new("tests/sample2_copy_no_metadata.png"))?;

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
write_to_file_jxl_no_conversion() 
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/iso_no_exif_copy.jxl")
	{
		println!("{}", error);
	}
	copy("tests/iso_no_exif.jxl", "tests/iso_no_exif_copy.jxl")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;
	
	// Write metadata to file
	metadata.write_to_file(Path::new("tests/iso_no_exif_copy.jxl"))?;

	Ok(())
}

#[test]
fn 
write_to_file_jxl_with_conversion() 
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/no_exif_copy.jxl")
	{
		println!("{}", error);
	}
	copy("tests/no_exif.jxl", "tests/no_exif_copy.jxl")?;

	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;
	
	// Write metadata to file
	metadata.write_to_file(Path::new("tests/no_exif_copy.jxl"))?;

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

#[test]
fn 
write_to_file_tiff_basic() 
-> Result<(), std::io::Error>
{
	// Remove file from previous run and replace it with fresh copy
	if let Err(error) = remove_file("tests/read_sample_copy.tif")
	{
		println!("{}", error);
	}
	copy("tests/read_sample.tif", "tests/read_sample_copy.tif")?;

	// Read original file
	let metadata = Metadata::new_from_path(Path::new("tests/read_sample.tif"))?;

	// Write metadata to file
	metadata.write_to_file(Path::new("tests/read_sample_copy.tif"))?;

	Ok(())
}

fn 
compare_write_to_generic
(
	original_file:  &str,
	copy1_file:     &str,
	copy2_file:     &str,
	file_extension: little_exif::filetype::FileExtension
)
-> Result<(), std::io::Error>
{
	// Create newly created & filled metadata struct
	let metadata = get_test_metadata()?;
	
	if let Err(error) = remove_file(copy1_file)
	{
		println!("{}", error);
	}
	copy(original_file, copy1_file)?;
	metadata.write_to_file(Path::new(copy1_file))?;

	// Now do the same but via the vec-based function
	if let Err(error) = remove_file(copy2_file)
	{
		println!("{}", error);
	}
	copy(original_file, copy2_file)?;
	let mut image_data = read(copy2_file).unwrap();
	metadata.write_to_vec(&mut image_data, file_extension)?;
	std::fs::write(copy2_file, image_data.clone())?;

	// Read first write version back in
	let compare_me = read(copy1_file).unwrap();

	// Compare their lengths
	if compare_me.len() != image_data.len()
	{
		panic!("Lengths differ! file: {} vs vec: {}", compare_me.len(), image_data.len());
	}

	// Compare their contents
	for i in 0..compare_me.len()
	{
		if compare_me[i] != image_data[i]
		{
			panic!("Data differs! file: {} vs vec: {}", compare_me[i], image_data[i]);
		}
	}

	Ok(())
}

#[test]
fn 
compare_write_to_jpg()
-> Result<(), std::io::Error>
{
	return compare_write_to_generic(
		"tests/sample2.jpg",
		"tests/sample2_copy1.jpg",
		"tests/sample2_copy2.jpg",
		little_exif::filetype::FileExtension::JPEG
	);
}

#[test]
fn 
compare_write_to_jxl()
-> Result<(), std::io::Error>
{
	return compare_write_to_generic(
		"tests/no_exif.jxl",
		"tests/no_exif_copy1.jxl",
		"tests/no_exif_copy2.jxl",
		little_exif::filetype::FileExtension::JXL
	);
}

#[test]
fn 
compare_write_to_png()
-> Result<(), std::io::Error>
{
	return compare_write_to_generic(
		"tests/sample2.png",
		"tests/sample2_copy1.png",
		"tests/sample2_copy2.png",
		little_exif::filetype::FileExtension::PNG { as_zTXt_chunk: false }
	);
}

#[test]
fn 
compare_write_to_webp_lossless()
-> Result<(), std::io::Error>
{
	return compare_write_to_generic(
		"tests/sample2_simple_lossless.webp",
		"tests/sample2_simple_lossless_copy1.webp",
		"tests/sample2_simple_lossless_copy2.webp",
		little_exif::filetype::FileExtension::WEBP
	);
}

#[test]
fn 
compare_write_to_webp_extended()
-> Result<(), std::io::Error>
{
	return compare_write_to_generic(
		"tests/sample2_extended.webp",
		"tests/sample2_extended_copy1.webp",
		"tests/sample2_extended_copy2.webp",
		little_exif::filetype::FileExtension::WEBP
	);
}