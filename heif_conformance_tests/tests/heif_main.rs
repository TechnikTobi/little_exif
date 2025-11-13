// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;
use std::str::FromStr;

use test_case::test_case;

extern crate little_exif;
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::filetype;

fn 
construct_cpy_path_string
(
    input_path: &str
) 
-> String 
{
    let mut parts: Vec<&str> = input_path.split('.').collect();

    // Store extension for later
    let extension = parts.pop().unwrap_or("");
    
    let base = parts.join(".");
    
    return format!("{}_copy.{}", base, extension);
}

#[test_case("C034.heic")]
fn
test_equal_results
(
    testfile: &str
)
-> Result<(), std::io::Error>
{
    let cpy_path_string = construct_cpy_path_string(testfile);

    let img_path = Path::new(testfile);
    let cpy_path = Path::new(cpy_path_string.as_str());

    if let Err(error) = remove_file(cpy_path)
    {
        println!("Could not delete file: {}", error);
    }
    copy(img_path, cpy_path).unwrap();

    let mut metadata = little_exif::metadata::Metadata::new();
    metadata.set_tag(little_exif::exif_tag::ExifTag::ImageDescription(
        "hello_world".to_string(),
    ));
    metadata.write_to_file(cpy_path)?;

    return Ok(());
}