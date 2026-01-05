// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
The file that does:

thread 'main' panicked at C:\Users\Michael Schnell\.cargo\git\checkouts\little_exif-fdd18c59a5ea2fbd\b6ab7bc\src\metadata.rs:591:34:
attempt to subtract with overflow
is
[REDACTED]
*/

/*
Solved:
Can't recall :(
*/

use std::path::Path;

extern crate little_exif;
extern crate little_exif_0_4_3;
extern crate little_exif_0_5_0;

#[test]
#[should_panic(expected = "attempt to subtract with overflow")]
fn read_exif_data_fails() {
    let path = Path::new("resources/issue_000020/2017_emilio_meister_IMG_4436.JPG");

    let mut tag_counter = 0;

    for _ in little_exif_0_4_3::metadata::Metadata::new_from_path(path)
        .unwrap()
        .data()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 47);
}

#[test]
fn read_exif_data_fixed() {
    let path = Path::new("resources/issue_000020/2017_emilio_meister_IMG_4436.JPG");

    let mut tag_counter = 0;

    for _ in little_exif_0_5_1::metadata::Metadata::new_from_path(path)
        .unwrap()
        .data()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 43);
}

#[test]
fn read_exif_data_current() {
    let path = Path::new("resources/issue_000020/2017_emilio_meister_IMG_4436.JPG");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path).unwrap() {
        tag_counter += 1;
    }

    // For some reason, there are now additional tags found?
    assert_eq!(tag_counter, 49);
}
