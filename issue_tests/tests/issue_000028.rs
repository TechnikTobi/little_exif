// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Within the 1500 jpg files under test several show library messages:

Could not validate EXIF header!

Error during decoding: Custom { kind: Other, error: "No EXIF data found!" }

Could not get IFD0 tags:
Could not decode SubIFD:
Illegal format for known tag! Tag: FocalLengthIn35mmFormat([]) Expected: INT16U Got: INT32U

Error during decoding: Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }
*/

/*
Solved:
Added conversion function in special cases where two different types are 
"acceptable" for the same tag, e.g. INT16U & INT32U (because camera makers are
too incompetent to stick to the specifications)
- e2951416652f613aa01a102f7b607d6db965a536
*/

use std::path::Path;

#[test]
fn
read_exif_data_1_current_version()
{
    let path1 = Path::new("resources/issue_000028/2017_aachen_abendhimmel_emilio.jpg");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path1).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 46);
}

#[test]
#[should_panic (expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"No EXIF data found!\" }")]
fn
read_exif_data_2_current_version()
{
    let path2 = Path::new("resources/issue_000028/2017_isernhagen_sorento_1.jpg");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path2).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 0);
}

#[test]
#[should_panic (expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"No EXIF data found!\" }")]
fn
read_exif_data_3_current_version()
{
    let path3 = Path::new("resources/issue_000028/2017_stockholm_emilio.jpg");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path3).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 0);
}