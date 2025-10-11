// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;
use std::str::FromStr;

extern crate little_exif;
use little_exif::exif_tag::ExifTag;
use little_exif::metadata::Metadata;
use little_exif::filetype;

#[test]
fn
read_exif_data_1()
{
    let path = Path::new("resources/issue_000002/0010_A0420427.JPG");

    let mut tag_counter = 0;

    for tag in &Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 60);
}

#[test]
fn
read_exif_data_2()
{
    let file_path = Path::new("resources/issue_000002/0010_A0420427.JPG");

    let extension = file_path.extension().unwrap();
    let extension = extension.to_str().unwrap();
    let file_type = filetype::FileExtension::from_str(extension).unwrap();
    let content = std::fs::read(file_path).unwrap();
    let metadata = Metadata::new_from_vec(&content, file_type);

    let mut tag_counter = 0;

    for tag in &metadata.unwrap()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_eq!(tag_counter, 60);
}

#[test]
fn
read_and_write_exif_data_1()
-> Result<(), std::io::Error>
{
    let path_orig = Path::new("resources/issue_000002/0010_A0420427.JPG");
    let path_copy = Path::new("resources/issue_000002/0010_A0420427_copy.JPG");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy)
    {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy)?;

    let mut metadata1 = Metadata::new_from_path(&path_orig).unwrap();

    let mut orig_tag_counter = 0;
    for _ in &metadata1
    {
        orig_tag_counter += 1;
    }

    metadata1.set_tag(
        ExifTag::RecommendedExposureIndex(vec![2024])
    );

    // Write metadata to copy
    metadata1.write_to_file(path_copy)?;

    // Read again
    let mut copy_tag_counter = 0;
    for _ in &Metadata::new_from_path(&path_copy).unwrap()
    {
        copy_tag_counter += 1;
    }

    assert_eq!(orig_tag_counter + 1, copy_tag_counter);

    return Ok(());
}