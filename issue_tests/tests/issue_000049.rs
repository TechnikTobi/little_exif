// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
- When retrieving the GPSLongitude tag we get 24 u8 values, i.e. works correctly
- When retrieving the GPSLatitude tag we only get 4 u8 values, i.e. does not work correctly 
*/

/*
Solved:
Forgot to check the group when performing a get_tag call, so user gets the tag
InteroperabilityVersion instead, as it has the same hex value as GPSLatitude.
Related commits:
- 8cd8e0d8f638fce9962a56c3fd1bd8ca62fb604d
- a239ad36674e149f6921dd7c115c6a6cd10b1387
*/

use std::path::Path;

#[test]
fn
read_gps_latitude_current()
{
    let     path         = Path::new("resources/issue_000049/382577930-5fd51906-3f81-4371-a968-a83ba43f4b20.jpg");
    let     metadata     = little_exif::metadata::Metadata::new_from_path(path).unwrap();
    let mut tag_iterator = metadata.get_tag(&little_exif::exif_tag::ExifTag::GPSLatitude(Vec::new()));

    match tag_iterator.next() {
        Some(tag) => assert_eq!(
            tag.value_as_u8_vec(&metadata.get_endian()).len(),
            24
        ),
        None => panic!("Tag does not exist"),
    };
}
