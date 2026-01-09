// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
The remove_tag operation, originally introduced in 0.6.12, doesn't work
*/

/*
Solved:
Didn't traverse all IFDs. Fixed in 0.6.13
Related commits:
- 92637c7d0539570bf063ab2925fa67592b0f0463
*/

use std::path::Path;

#[test]
fn
remove_tag_current()
{
    let path = Path::new("resources/issue_000066/464717007-c0dff257-5c39-4b7f-8908-9eeebb5a627c.jpeg");

    let mut metadata = little_exif::metadata::Metadata::new_from_path(path).unwrap();

    let mut original_tag_counter = 0;
    for _ in &metadata
    {
        original_tag_counter += 1;
    }

    metadata.remove_tag(little_exif::exif_tag::ExifTag::CreateDate("".to_string()));
    metadata.remove_tag(little_exif::exif_tag::ExifTag::ModifyDate("".to_string()));
    metadata.remove_tag(little_exif::exif_tag::ExifTag::DateTimeOriginal("".to_string()));
    
    // not present in file -> ignore it
    // metadata.remove_tag(little_exif::exif_tag::ExifTag::OffsetTime("".to_string()));
    
    // not present in file -> ignore it
    // metadata.remove_tag(little_exif::exif_tag::ExifTag::OffsetTimeOriginal("".to_string()));
    
    metadata.remove_tag(little_exif::exif_tag::ExifTag::SubSecTime("".to_string()));
    metadata.remove_tag(little_exif::exif_tag::ExifTag::SubSecTimeOriginal("".to_string()));
    metadata.remove_tag(little_exif::exif_tag::ExifTag::SubSecTimeDigitized("".to_string()));
    
    // not present in file -> ignore it
    // metadata.remove_tag(little_exif::exif_tag::ExifTag::Copyright("".to_string()));

    let mut new_tag_counter = 0;
    for _ in &metadata
    {
        new_tag_counter += 1;
    }

    assert_eq!(new_tag_counter, original_tag_counter-6);
}