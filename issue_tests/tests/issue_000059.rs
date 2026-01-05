// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
When I run the program the first time everything looks ok. But subsequent runs results in:
thread 'main' panicked at /home/francis/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/little_exif-0.6.5/src/png/text.rs:25:46:
called `Result::unwrap()` on an `Err` value: FromUtf8Error { bytes: [82, 97, 119, 32, 112, 114, 111, 102, 105, 108, 101, 32, 116, 121, 112, 101, 32, 101, 120, 105, 102, 120, 156, 101, 138, 209, 13, 128, 64, 8, 67, 255, 153, 194, 17, 56, 2, 28, 140, 99, 206, 35, 113, 3, 199, 183, 81, 19, 63, 238, 17, 218, 166, 41, 205, 235, 44, 218, 30, 172, 145, 90, 15, 79, 119, 6, 154, 154, 178, 35, 4, 191, 52, 252, 132, 10, 252, 248, 170, 193, 11, 26, 110, 62, 112, 37, 108, 221, 171, 11, 178, 74, 251, 23, 116, 3, 175, 236, 24, 162], error: Utf8Error { valid_up_to: 22, error_len: Some(1) } }
*/

/*
Solved:
Problem with the way zTXt chunks are constructed, has to do with information on compression levels and magic value prefixes
Related commits:
- ac1e6c7ae0faed4fe2d61f934eccfb939834363e
- 96f76ee647a1b3b715d60128d0f5d7fb49a2fe7e
*/

use std::fs::{copy, remove_file};
use std::path::Path;

extern crate little_exif;
extern crate little_exif_0_6_5;
extern crate little_exif_0_6_6;

#[test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: FromUtf8Error { bytes: [82, 97, 119, 32, 112, 114, 111, 102, 105, 108, 101, 32, 116, 121, 112, 101, 32, 101, 120, 105, 102, 120, 156, 101, 136, 75, 10, 128, 48, 12, 68, 247, 57, 133, 71, 72, 210, 105, 108, 142, 35, 82, 193, 27, 120, 124, 7, 45, 116, 209, 7, 243, 225, 73, 127, 238, 75, 182, 15, 20, 65, 221, 91, 100, 132, 18, 36, 210, 15, 158, 166, 63, 198, 116, 182, 115, 235, 80, 167, 46, 192, 224, 40, 110, 211, 200, 11, 39, 239, 20, 43], error: Utf8Error { valid_up_to: 22, error_len: Some(1) } }"
)]
fn read_exif_data_prior_to_bugfix() {
    let path_orig = Path::new("resources/issue_000059/447912738-6fd9f973-a793-4f09-97e0-2a8ad4f46e25.png");
    let path_copy = Path::new("resources/issue_000059/447912738-6fd9f973-a793-4f09-97e0-2a8ad4f46e25_copy1.png");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy) {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy).unwrap();

    let mut metadata = little_exif_0_6_5::metadata::Metadata::new();
    metadata.set_tag(little_exif_0_6_5::exif_tag::ExifTag::ImageDescription(
        "ABC!".to_string(),
    ));
    metadata.write_to_file(&path_copy).unwrap();

    // Read metadata from file
    let mut tag_counter = 0;
    for tag in &little_exif_0_6_5::metadata::Metadata::new_from_path(path_copy).unwrap() {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 1);

    // Update and read again
    metadata.set_tag(little_exif_0_6_5::exif_tag::ExifTag::ImageDescription(
        "XYZ!".to_string(),
    ));
    metadata.write_to_file(&path_copy).unwrap();

    let mut tag_counter = 0;
    for tag in &little_exif_0_6_5::metadata::Metadata::new_from_path(path_copy).unwrap() {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 1);
}

#[test]
fn read_exif_data_fixed() {
    let path_orig = Path::new("resources/issue_000059/447912738-6fd9f973-a793-4f09-97e0-2a8ad4f46e25.png");
    let path_copy = Path::new("resources/issue_000059/447912738-6fd9f973-a793-4f09-97e0-2a8ad4f46e25_copy2.png");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy) {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy).unwrap();

    let mut metadata = little_exif_0_6_6::metadata::Metadata::new();
    metadata.set_tag(little_exif_0_6_6::exif_tag::ExifTag::ImageDescription(
        "ABC!".to_string(),
    ));
    metadata.write_to_file(&path_copy).unwrap();

    // Read metadata from file
    let mut tag_counter = 0;
    for tag in &little_exif_0_6_6::metadata::Metadata::new_from_path(path_copy).unwrap() {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 1);

    // Update and read again
    metadata.set_tag(little_exif_0_6_6::exif_tag::ExifTag::ImageDescription(
        "XYZ!".to_string(),
    ));
    metadata.write_to_file(&path_copy).unwrap();

    let mut tag_counter = 0;
    for tag in &little_exif_0_6_6::metadata::Metadata::new_from_path(path_copy).unwrap() {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 1);
}

#[test]
fn read_exif_data_current() {
    let path_orig = Path::new("resources/issue_000059/447912738-6fd9f973-a793-4f09-97e0-2a8ad4f46e25.png");
    let path_copy = Path::new("resources/issue_000059/447912738-6fd9f973-a793-4f09-97e0-2a8ad4f46e25_copy3.png");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy) {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy).unwrap();

    let mut metadata = little_exif::metadata::Metadata::new();
    metadata.set_tag(little_exif::exif_tag::ExifTag::ImageDescription("ABC!".to_string()));
    metadata.write_to_file(&path_copy).unwrap();

    // Read metadata from file
    let mut tag_counter = 0;
    for tag in &little_exif::metadata::Metadata::new_from_path(path_copy).unwrap() {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 1);

    // Update and read again
    metadata.set_tag(little_exif::exif_tag::ExifTag::ImageDescription("XYZ!".to_string()));
    metadata.write_to_file(&path_copy).unwrap();

    let mut tag_counter = 0;
    for tag in &little_exif::metadata::Metadata::new_from_path(path_copy).unwrap() {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 1);
}
