// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
When trying to process png files with certain types of chunks, such as iTXt chunks, the following error occurs:
Error writing EXIF ​​data: Invalid PNG chunk name
*/

/*
Solved:
Missed some PNG chunk types
*/

use std::path::Path;

extern crate little_exif_0_6_2;
extern crate little_exif;

#[test]
#[should_panic (expected = "assertion failed: metadata.is_err()")]
fn
read_exif_data_png1_old_version_fails()
{
    let png_path = Path::new("resources/issue_000052/test1.png");
    let metadata = little_exif_0_6_2::metadata::Metadata::new_from_path(png_path);
    assert!(metadata.is_err());
}

#[test]
#[should_panic (expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"No metadata found!\" }")]
fn
read_exif_data_png1_new_version_fails()
{
    // This is supposed to panic as there is not metadata in this file
    let png_path = Path::new("resources/issue_000052/test1.png");
    let _ = little_exif::metadata::Metadata::new_from_path(png_path).unwrap();
}

#[test]
#[should_panic (expected = "assertion failed: metadata.is_err()")]
fn
read_exif_data_png2_old_version_fails()
{
    let png_path = Path::new("resources/issue_000052/test2.png");
    let metadata = little_exif_0_6_2::metadata::Metadata::new_from_path(png_path);
    assert!(metadata.is_err());
}

#[test]
fn
read_exif_data_png2_new_version_works()
{
    let png_path = Path::new("resources/issue_000052/test2.png");
    let _ = little_exif::metadata::Metadata::new_from_path(png_path).unwrap();
}

#[test]
#[should_panic (expected = "assertion failed: metadata.is_err()")]
fn
read_exif_data_png3_old_version_fails()
{
    let png_path = Path::new("resources/issue_000052/test3.png");
    let metadata = little_exif_0_6_2::metadata::Metadata::new_from_path(png_path);
    assert!(metadata.is_err());
}

#[test]
#[should_panic (expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"No metadata found!\" }")]
fn
read_exif_data_png3_new_version_fails()
{
    let png_path = Path::new("resources/issue_000052/test3.png");
    let _ = little_exif::metadata::Metadata::new_from_path(png_path).unwrap();
}