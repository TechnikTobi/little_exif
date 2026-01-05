// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

use test_case::test_case;

extern crate little_exif;

fn construct_cpy_path_string(input_path: &str) -> String {
    let mut parts: Vec<&str> = input_path.split('.').collect();

    // Store extension for later
    let extension = parts.pop().unwrap_or("");

    let base = parts.join(".");

    return format!("{}_copy.{}", base, extension);
}

// #[test_case("C001.heic")] currently fails
// #[test_case("C002.heic")] currently fails
// #[test_case("C003.heic")] currently fails
#[test_case("C004.heic")]
#[test_case("C005.heic")]
#[test_case("C006.heic")]
#[test_case("C007.heic")]
#[test_case("C008.heic")]
#[test_case("C009.heic")]
#[test_case("C010.heic")]
#[test_case("C011.heic")]
#[test_case("C012.heic")]
#[test_case("C013.heic")]
#[test_case("C014.heic")]
#[test_case("C015.heic")]
#[test_case("C016.heic")]
#[test_case("C017.heic")]
#[test_case("C018.heic")]
#[test_case("C019.heic")]
#[test_case("C020.heic")]
#[test_case("C021.heic")]
#[test_case("C022.heic")]
#[test_case("C023.heic")]
#[test_case("C024.heic")]
#[test_case("C025.heic")]
// #[test_case("C026.heic")] currently fails
// #[test_case("C027.heic")] currently fails
// #[test_case("C028.heic")] currently fails
// #[test_case("C029.heic")] currently fails
// #[test_case("C030.heic")] currently fails
// #[test_case("C031.heic")] currently fails
// #[test_case("C032.heic")] currently fails
// #[test_case("C033.heic")] does not exist as of 2025-11-13
#[test_case("C034.heic")]
// #[test_case("C035.heic")] does not exist as of 2025-11-13
// #[test_case("C036.heic")] currently fails
// #[test_case("C037.heic")] currently fails
// #[test_case("C038.heic")] currently fails
#[test_case("C039.heic")]
#[test_case("C040.heic")]
// #[test_case("C041.heic")] currently fails
#[test_case("C042.heic")]
#[test_case("C043.heic")]
#[test_case("C044.heic")]
#[test_case("C045.heic")]
#[test_case("C046.heic")]
#[test_case("C047.heic")]
#[test_case("C048.heic")]
#[test_case("C049.heic")]
#[test_case("C050.heic")]
#[test_case("C051.heic")]
#[test_case("C052.heic")]
#[test_case("C053.heic")]
fn test_equal_results(testfile: &str) -> Result<(), std::io::Error> {
    let cpy_path_string = construct_cpy_path_string(testfile);

    let img_path = Path::new(testfile);
    let cpy_path = Path::new(cpy_path_string.as_str());

    if let Err(error) = remove_file(cpy_path) {
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
