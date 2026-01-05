// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
moreover there is one file (in another testing set) which provides:

Could not get IFD0 tags:
 Could not decode SubIFD:
  Illegal format for known tag! Tag: ExposureCompensation([]) Expected: RATIONAL64S Got: RATIONAL64U
WARNING: Can't read metadata - Create new & empty struct
*/

/*
Solution: Added case for converting R64U into R64S in exif_tag/decode.rs
Related commit:
- 67209de619a73f8a7e188c25fc6cb51634afe313
*/

use std::path::Path;

extern crate little_exif;
extern crate little_exif_0_6_0_beta_1;
extern crate little_exif_0_6_15;

#[test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"Could not decode SubIFD EXIF:\\n  Illegal format for known tag! Tag: ExposureCompensation([]) Expected: RATIONAL64S Got: RATIONAL64U\" }"
)]
fn read_exif_data_fails() {
    let png_path = Path::new("resources/issue_000021/dsc22921.jpg");

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_15::metadata::Metadata::new_from_path(png_path).unwrap() {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 46);
}

#[test]
fn read_exif_data_fixed() {
    let png_path = Path::new("resources/issue_000021/dsc22921.jpg");

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_16_beta_1::metadata::Metadata::new_from_path(png_path).unwrap() {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 46);
}

#[test]
fn read_exif_data_current() {
    let png_path = Path::new("resources/issue_000021/dsc22921.jpg");

    let mut tag_counter = 0;
    for tag in &little_exif::metadata::Metadata::new_from_path(png_path).unwrap() {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 46);
}
