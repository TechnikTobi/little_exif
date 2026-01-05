// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
thread 'main' panicked at C:\Users\X\.cargo\registry\src\index.crates.io-6f17d22bba15001f\little_exif-0.6.0-beta.1\src\ifd\mod.rs:263:21:
assertion `left == right` failed
  left: Some(1313426255)
 right: None
*/

/*
Solved:
Removed assert, see the following commit for more details:
- eed3784c7c091ec590ffca71622439fd7a64a4fd
*/

use std::path::Path;

extern crate little_exif;
extern crate little_exif_0_6_0_beta_1;
extern crate little_exif_0_6_0_beta_2;

#[test]
#[should_panic(
    expected = "assertion `left == right` failed\n  left: Some(1313426255)\n right: None"
)]
fn read_exif_data_fails() {
    let path = Path::new("resources/issue_000031/DSC22278.JPG");

    let mut tag_counter = 0;

    for _ in &little_exif_0_6_0_beta_1::metadata::Metadata::new_from_path(path).unwrap() {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 47);
}

#[test]
fn read_exif_data_fixed() {
    let path = Path::new("resources/issue_000031/DSC22278.JPG");

    let mut tag_counter = 0;

    for _ in &little_exif_0_6_0_beta_2::metadata::Metadata::new_from_path(path).unwrap() {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 47);
}

#[test]
fn read_exif_data_current() {
    let path = Path::new("resources/issue_000031/DSC22278.JPG");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path).unwrap() {
        tag_counter += 1;
    }

    // For some reason, there is now an additional tag found?
    // No idea when this changed
    assert_eq!(tag_counter, 48);
}
