// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
I already did have this issue with these images and it does persist with 0.5.1:
using new_from,_vec and write_top_vec I set an "ImageDescription" tag to a file that already did feature this tag and save to a new file.
The I read the new file with the same program and get

thread 'main' panicked at C:\Users\X\.cargo\registry\src\index.crates.io-6f17d22bba15001f\little_exif-0.5.1\src\metadata.rs:566:70:
range end index 730 out of range for slice of length 728
The explorer "Details" sow the new ImageDescription as "Betreff" (with different files I also saw that ImageDesctription had been shown as "Titel" ?!?!?.
*/

/*
Solution: Can't recall :(
*/

use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
extern crate little_exif_0_5_1;
extern crate little_exif_0_6_0_beta_1;

#[test]
#[should_panic(expected = "range end index 737 out of range for slice of length 735")]
fn read_write_read_exif_data_fails() {
    let png_path = Path::new("resources/issue_000025/A0579322.jpg");
    let cpy_path = Path::new("resources/issue_000025/A0579322_copy1.jpg");

    if let Err(error) = remove_file(cpy_path) {
        println!("Could not delete file: {}", error);
    }
    copy(png_path, cpy_path).unwrap();

    let mut metadata = little_exif_0_5_1::metadata::Metadata::new_from_path(png_path).unwrap();
    metadata.set_tag(little_exif_0_5_1::exif_tag::ExifTag::ImageDescription(
        "Hello World!".to_string(),
    ));

    metadata.write_to_file(cpy_path).unwrap();

    let mut tag_counter = 0;
    for _ in little_exif_0_5_1::metadata::Metadata::new_from_path(cpy_path)
        .unwrap()
        .data()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 0);
}

#[test]
fn read_write_read_exif_data_fixed() {
    let png_path = Path::new("resources/issue_000025/A0579322.jpg");
    let cpy_path = Path::new("resources/issue_000025/A0579322_copy2.jpg");

    if let Err(error) = remove_file(cpy_path) {
        println!("Could not delete file: {}", error);
    }
    copy(png_path, cpy_path).unwrap();

    let mut metadata =
        little_exif_0_6_0_beta_1::metadata::Metadata::new_from_path(png_path).unwrap();
    metadata.set_tag(
        little_exif_0_6_0_beta_1::exif_tag::ExifTag::ImageDescription("Hello World!".to_string()),
    );

    metadata.write_to_file(cpy_path).unwrap();

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_0_beta_1::metadata::Metadata::new_from_path(cpy_path).unwrap() {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 41);
}

#[test]
fn read_write_read_exif_data_current() {
    let png_path = Path::new("resources/issue_000025/A0579322.jpg");
    let cpy_path = Path::new("resources/issue_000025/A0579322_copy3.jpg");

    if let Err(error) = remove_file(cpy_path) {
        println!("Could not delete file: {}", error);
    }
    copy(png_path, cpy_path).unwrap();

    let mut metadata = little_exif::metadata::Metadata::new_from_path(png_path).unwrap();
    metadata.set_tag(little_exif::exif_tag::ExifTag::ImageDescription(
        "Hello World!".to_string(),
    ));

    metadata.write_to_file(cpy_path).unwrap();

    let mut tag_counter = 0;
    for _ in &little_exif::metadata::Metadata::new_from_path(cpy_path).unwrap() {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 41);
}
