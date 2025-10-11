// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
0.6.3. fails with an out of bounds error when extracting exif for PNGs like below.
*/

/*
Solved:
Had some trouble with endian information and missed a PNG chunk type.
Related commits:
- 575bd6a88377482ffcee3652133451373e273127
- f563d3c369ebc3b57aa06e1fa5487024ba108851
*/

use std::path::Path;

extern crate little_exif_0_6_3;
extern crate little_exif;

#[test]
#[should_panic (expected = "Out of bounds access")]
fn
issue_000054_read_exif_data_old_version_fails()
{
    let path = Path::new("resources/issue_000054/437532191-50f650a1-788c-44a4-a535-526d10d297ec.png");

    let mut tag_counter = 0;

    for tag in &little_exif_0_6_3::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 15);
}

#[test]
fn
issue_000054_read_exif_data_fixed()
{
    let path = Path::new("resources/issue_000054/437532191-50f650a1-788c-44a4-a535-526d10d297ec.png");

    let mut tag_counter = 0;

    for tag in &little_exif::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 15);
}
