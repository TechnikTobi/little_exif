// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Could not decode SubIFD GPS:
 Illegal format for known tag! Tag: GPSAltitudeRef("") Expected: STRING Got: INT8U
*/

/*
Solved:
Change type of GPSAltitudeRef tag
Related commits:
- 10fbf8c845568d0ebe03c68f13a8bd2c0eb1da36
*/

use std::path::Path;

/*
#[test]
#[should_panic (expected = "assertion `left == right` failed\n  left: 0\n right: 58")]
fn
read_exif_data_fails()
{
    let path = Path::new("resources/issue_000038/2017_stuttgart_07_.jpeg");

    let mut tag_counter = 0;

    for tag in &little_exif_0_6_0_beta_3::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 58);
}

#[test]
fn
read_exif_data_fixed()
{
    let path = Path::new("resources/issue_000038/2017_stuttgart_07_.jpeg");

    let mut tag_counter = 0;

    for tag in &little_exif_0_6_0_beta_4::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 58);
}
*/

#[test]
fn
read_exif_data_current()
{
    let path = Path::new("resources/issue_000038/2017_stuttgart_07_.jpeg");

    let mut tag_counter = 0;

    for tag in &little_exif::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 58);
}
