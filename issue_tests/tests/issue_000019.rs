// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Error during decoding: Custom { kind: Other, error: "No EXIF data found!" }
WARNING: Can't read metadata - Create new & empty struct
*/

/*
Solved:
Traversing JPEGs not byte-by-byte but segment-by-segment when reading and searching for the APP1 segment that contains the EXIF data.
Related commits:
- 14ce1d8d8116ea2e5478c84e0e27f7ce0d2499ec
- dd4218f8ed5df07a749d534c5e44325ea1f6084d
- 0c37c0f6d49505111297f52c772ff32802048aa5
- ddf7992b37613905d4cca503b452a85a182c7870
- 74884293055538b486cced49d143c096ef8873a8
- 18bc54ec942eadd4dcedc0af85c6ac87f825cb40
- 961a8c13d89789523b8ce0bcabe0f0bdbfb1cd61
*/

use std::path::Path;

/*
#[test]
#[should_panic (expected = "assertion `left != right` failed\n  left: 0\n right: 0")]
fn
read_exif_data_fails()
{
    let path = Path::new("resources/issue_000019/2019_Stuttgart_Emilio_SL_925-014-126.JPG");

    let mut tag_counter = 0;

    for _ in little_exif_0_5_0::metadata::Metadata::new_from_path(path).unwrap().data()
    {
        tag_counter += 1;
    }

    assert_ne!(tag_counter, 0);
}

#[test]
fn
read_exif_data_fixed()
{
    let path = Path::new("resources/issue_000019/2019_Stuttgart_Emilio_SL_925-014-126.JPG");

    let mut tag_counter = 0;

    for _ in little_exif_0_5_1::metadata::Metadata::new_from_path(path).unwrap().data()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 40);
}
*/

#[test]
fn
read_exif_data_current()
{
    let path = Path::new("resources/issue_000019/2019_Stuttgart_Emilio_SL_925-014-126.JPG");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
    }

    // For some reason, there are now additional tags found?
    assert_eq!(tag_counter, 46);
}
