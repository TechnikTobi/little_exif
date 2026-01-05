// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
moreover with some files, I get
Error during decoding: Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }
*/

/*
Solved:
There was a typo, see related commit:
- 8965456229fb6e366dd24d5f67d730ee6a7cc086
*/

use std::path::Path;

extern crate little_exif;
extern crate little_exif_0_6_0_beta_3;
extern crate little_exif_0_6_0_beta_4;

#[test]
#[should_panic(expected = "assertion `left == right` failed\n  left: 0\n right: 58")]
fn read_exif_data_fails() {
    let path = Path::new("resources/issue_000036/IMG-20180904-WA0002.jpg");

    let mut tag_counter = 0;

    for tag in &little_exif_0_6_0_beta_1::metadata::Metadata::new_from_path(path).unwrap() {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 58);
}

#[test]
fn read_exif_data_fixed() {
    let path = Path::new("resources/issue_000036/IMG-20180904-WA0002.jpg");

    let mut tag_counter = 0;

    for tag in &little_exif_0_6_0_beta_4::metadata::Metadata::new_from_path(path).unwrap() {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 0);
}

#[test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"No EXIF data found!\" }"
)]
fn read_exif_data_current() {
    let path = Path::new("resources/issue_000036/IMG-20180904-WA0002.jpg");

    for tag in &little_exif::metadata::Metadata::new_from_path(path).unwrap() {
        println!("{:?}", tag);
    }
}
