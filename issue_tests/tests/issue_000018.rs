// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Here the file that works with `new_from_path` but not with `new_from_vec`
code:
    let extension = file_path.extension().unwrap();
    let extension = extension.to_str().unwrap();
    let file_type = filetype::FileExtension::from_str(extension).unwrap();
    let mut content = std::fs::read(file_path).unwrap();
    let metadata = Metadata::new_from_vec(&content, file_type);
*/

/*
Solved:
Can't recall :(
*/

use std::path::Path;
use std::str::FromStr;

#[test]
fn
read_exif_data_current()
{
    let jpg_path = Path::new("resources/issue_000018/x.jpg");

    let mut tag_count_path = 0;
    let mut tag_count_vec = 0;

    // Read metadata from file
    for tag in &little_exif::metadata::Metadata::new_from_path(jpg_path).unwrap()
    {
        println!("{:?}", tag);
        tag_count_path += 1;
    }

    // Read metadata from vec
    let extension = jpg_path.extension().unwrap();
    let extension = extension.to_str().unwrap();
    let file_type = little_exif::filetype::FileExtension::from_str(extension).unwrap();
    let content = std::fs::read(jpg_path).unwrap();
    let metadata = little_exif::metadata::Metadata::new_from_vec(&content, file_type);

    for tag in &metadata.unwrap()
    {
        println!("{:?}", tag);
        tag_count_vec += 1;
    }

    assert_eq!(tag_count_path, tag_count_vec);
}
