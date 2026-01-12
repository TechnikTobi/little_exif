// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Downloaded a jpg picture from my android s23 and ran rust program in linux: match Metadata::new_from_path(Path::new(&fullfrom))
This gave me an error of: Could not decode SubIFD GPS: Illegal format for known tag! Tag: GPSAltitudeRef([]) Expected: INT8U Got:INT16U
Is there a work around?
*/

/*
Solution: Added conversion for INT16U -> INT8U with asserts regarding bounds
Related commit:
- f428d7cdd7687077ef1dd975a01b78912323e3ab
*/

use std::path::Path;

/*
#[test]
#[should_panic (expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"Could not decode SubIFD GPS:\\n  Illegal format for known tag! Tag: GPSAltitudeRef([]) Expected: INT8U Got: INT16U\" }")]
fn
read_exif_data_fails()
{
    let img_path = Path::new("resources/issue_000074/515375534-6e537b75-8c85-47ca-a62c-27639a90b73c.jpg");

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_18::metadata::Metadata::new_from_path(img_path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 55);
}

#[test]
fn
read_exif_data_fixed()
{
    let img_path = Path::new("resources/issue_000074/515375534-6e537b75-8c85-47ca-a62c-27639a90b73c.jpg");

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_19::metadata::Metadata::new_from_path(img_path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 55);
}
*/

#[test]
fn
read_exif_data_current()
{
    let img_path = Path::new("resources/issue_000074/515375534-6e537b75-8c85-47ca-a62c-27639a90b73c.jpg");

    let mut tag_counter = 0;
    for tag in &little_exif::metadata::Metadata::new_from_path(img_path).unwrap()
    {
        println!("{:?}", tag);
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 55);
}