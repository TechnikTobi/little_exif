// Copyright © 2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Description

Calling write_to_file() on a Metadata object created with Metadata::new() panics, even without setting any tags.

Steps to Reproduce

Take a JPEG image without EXIF metadata
Metadata::new_from_path() fails with "No EXIF data found!"
Create empty metadata with Metadata::new()
Call write_to_file(path) — panics (even without setting any tags)
  use little_exif::metadata::Metadata;
  use std::path::Path;

  fn main() {
      let path = Path::new("jpeg_without_exif.jpg");

      // This fails: "No EXIF data found!"
      let result = Metadata::new_from_path(path);
      assert!(result.is_err());

      // Create empty metadata
      let metadata = Metadata::new();

      // PANICS — even with no tags set!
      metadata.write_to_file(path).unwrap();
  }
Panic Output

thread 'main' panicked at little_exif-0.6.20/src/metadata/get.rs:73:14:
called Option::unwrap() on a None value

Expected Behavior

write_to_file() should either:

Successfully write (empty) EXIF data to the file, or
Return an Err explaining the limitation
Actual Behavior

Panics with unwrap() on None.

Environment

little_exif: 0.6.20
Rust: 1.92.0
*/

/*
Solved:
The function for getting the maximum generic IFD number did not handle the case
where no generic IFDs were present yet. This caused an unwrap() on None while
writing the empty metadata to file.
*/

use std::path::Path;
use std::fs::copy;
use std::fs::remove_file;

extern crate little_exif_0_6_20;
extern crate little_exif_0_5_0_beta_2;
extern crate little_exif;

#[test]
#[should_panic (expected = "called `Option::unwrap()` on a `None` value")]
fn
write_exif_data_fails()
{
    let path_orig = Path::new("resources/issue_000076/2017_stockholm_emilio.jpg");
    let path_copy = Path::new("resources/issue_000076/2017_stockholm_emilio_copy.jpg");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy)
    {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy).unwrap();

    // Ensure that the JPEG does not have any EXIF data as stated in the
    // original problem description: "Take a JPEG image without EXIF metadata"
    let exif_read_result = little_exif_0_6_20::metadata::Metadata::new_from_path(path_copy);
    assert!(exif_read_result.is_err());
    assert_eq!(
        exif_read_result.unwrap_err().get_ref().unwrap().to_string(),
        "No EXIF data found!".to_string()
    );

    let metadata = little_exif_0_6_20::metadata::Metadata::new();
    metadata.write_to_file(path_copy).unwrap();
}


#[test]
fn
write_exif_data_fixed()
{
    let path_orig = Path::new("resources/issue_000076/2017_stockholm_emilio.jpg");
    let path_copy = Path::new("resources/issue_000076/2017_stockholm_emilio_copy2.jpg");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy)
    {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy).unwrap();

    // Ensure that the JPEG does not have any EXIF data as stated in the
    // original problem description: "Take a JPEG image without EXIF metadata"
    let exif_read_result = little_exif_0_6_21::metadata::Metadata::new_from_path(path_copy);
    assert!(exif_read_result.is_err());
    assert_eq!(
        exif_read_result.unwrap_err().get_ref().unwrap().to_string(),
        "No EXIF data found!".to_string()
    );

    let metadata = little_exif_0_6_21::metadata::Metadata::new();
    metadata.write_to_file(path_copy).unwrap();
}


#[test]
fn
write_exif_data_current()
{
    let path_orig = Path::new("resources/issue_000076/2017_stockholm_emilio.jpg");
    let path_copy = Path::new("resources/issue_000076/2017_stockholm_emilio_copy3.jpg");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy)
    {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy).unwrap();

    // Ensure that the JPEG does not have any EXIF data as stated in the
    // original problem description: "Take a JPEG image without EXIF metadata"
    let exif_read_result = little_exif::metadata::Metadata::new_from_path(path_copy);
    assert!(exif_read_result.is_err());
    assert_eq!(
        exif_read_result.unwrap_err().get_ref().unwrap().to_string(),
        "No EXIF data found!".to_string()
    );

    let metadata = little_exif::metadata::Metadata::new();
    metadata.write_to_file(path_copy).unwrap();
}
