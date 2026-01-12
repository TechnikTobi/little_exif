// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
now I used the example code from the readme file to write an image description to this file.

    let mut metadata = Metadata::new();
    metadata.set_tag(ExifTag::ImageDescription("Hello World!".to_string()));
    let res = metadata.write_to_file(std::path::Path::new(&file_path));
    println!("res: {:?}", res);
This does work without an error.

But then trying to read that file again with the "little_exif" code that worked before gets the error message:

thread 'main' panicked at C:\Users\mschnell\.cargo\registry\src\index.crates.io-6f17d22bba15001f\little_exif-0.4.1\src\metadata.rs:461:70:
range end index 2 out of range for slice of length 0
writing to the file made it from length 782701 to length 781906. The picture still can be shown and Windows Explorer indeed show "Hello World!" as "Betreff" (which before had been "Thumbs Annotation" ). But the ThumbsPlus software does not show any fields any more.
Several Fields Explorer shows have been stayed intact, but the fields
Auflösungseinheit
Farbdardstellung
Kamerahersteller
Kameramodell
Blendenzahl
ISO
Lichtwert
Brennweite
Maximale Blende
Messmodus
... (and many more)
Exif-Version (had been 0232)

are empty now.
*/

/*
Solution: Can't recall :(
*/

use std::path::Path;
use std::fs::remove_file;
use std::fs::copy;

/*
#[test]
#[should_panic (expected = "range end index 2 out of range for slice of length 0")]
fn
issue_000055_clear_exif_data_old_version_fails()
{
    let img_path = Path::new("resources/issue_000015/IMG_20240828_184255.jpg");
    let cpy_path = Path::new("resources/issue_000015/IMG_20240828_184255_copy1.jpg");

    if let Err(error) = remove_file(cpy_path)
    {
        println!("Could not delete file: {}", error);
    }
    copy(img_path, cpy_path).unwrap();

    let read_metadata_1 = little_exif_0_4_1::metadata::Metadata::new_from_path(&img_path).unwrap();

    let mut tag_counter = 0;
    for tag in read_metadata_1.data()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_ne!(tag_counter, 0);

    let mut new_metadata = little_exif_0_4_1::metadata::Metadata::new();
    new_metadata.set_tag(little_exif_0_4_1::exif_tag::ExifTag::ImageDescription("Hello World!".to_string()));
    new_metadata.write_to_file(cpy_path).unwrap();

    let read_metadata_2 = little_exif_0_4_1::metadata::Metadata::new_from_path(&cpy_path).unwrap();

    let mut tag_counter = 0;
    for tag in read_metadata_2.data()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_ne!(tag_counter, 0);
}

#[test]
fn
issue_000055_clear_exif_data_fixed()
{
    let img_path = Path::new("resources/issue_000015/IMG_20240828_184255.jpg");
    let cpy_path = Path::new("resources/issue_000015/IMG_20240828_184255_copy2.jpg");

    if let Err(error) = remove_file(cpy_path)
    {
        println!("Could not delete file: {}", error);
    }
    copy(img_path, cpy_path).unwrap();

    let read_metadata_1 = little_exif_0_4_2::metadata::Metadata::new_from_path(&img_path).unwrap();

    let mut tag_counter = 0;
    for tag in read_metadata_1.data()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_ne!(tag_counter, 0);

    let mut new_metadata = little_exif_0_4_2::metadata::Metadata::new();
    new_metadata.set_tag(little_exif_0_4_2::exif_tag::ExifTag::ImageDescription("Hello World!".to_string()));
    new_metadata.write_to_file(cpy_path).unwrap();

    let read_metadata_2 = little_exif_0_4_2::metadata::Metadata::new_from_path(&cpy_path).unwrap();

    let mut tag_counter = 0;
    for tag in read_metadata_2.data()
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_ne!(tag_counter, 0);
}
*/

#[test]
fn
issue_000055_clear_exif_data_current()
{
    let img_path = Path::new("resources/issue_000015/IMG_20240828_184255.jpg");
    let cpy_path = Path::new("resources/issue_000015/IMG_20240828_184255_copy3.jpg");

    if let Err(error) = remove_file(cpy_path)
    {
        println!("Could not delete file: {}", error);
    }
    copy(img_path, cpy_path).unwrap();

    let read_metadata_1 = little_exif::metadata::Metadata::new_from_path(&img_path).unwrap();

    let mut tag_counter = 0;
    for tag in &read_metadata_1
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_ne!(tag_counter, 0);

    let mut new_metadata = little_exif::metadata::Metadata::new();
    new_metadata.set_tag(little_exif::exif_tag::ExifTag::ImageDescription("Hello World!".to_string()));
    new_metadata.write_to_file(cpy_path).unwrap();

    let read_metadata_2 = little_exif::metadata::Metadata::new_from_path(&cpy_path).unwrap();

    let mut tag_counter = 0;
    for tag in &read_metadata_2
    {
        tag_counter += 1;
        println!("{:?}", tag);
    }

    assert_ne!(tag_counter, 0);
}
