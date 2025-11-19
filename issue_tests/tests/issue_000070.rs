// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
I have a file that is causing a panic:

thread 'main' panicked at C:\Users\Person\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\little_exif-0.6.14\src\heif\container.rs:67:14:
called `Option::unwrap()` on a `None` value
I've shared the original image here [https://1drv.ms/u/c/28b8b4ed380ef804/EV9G5Q0VDylClDI9Twok9qwB69vXv-DM22iOW5JwqtX0Uw?e=41Yhwf].

It's worked on hundreds of other HEIC files -- the only thing that is obviously different about this one is that it was modified by drawing on the image and then being saved on iOS in the Photos app.
*/

/*
Solution: Added the function FileExtension::auto_detect where the first 32
bytes are used to find out what type of file this is. If this results in a
different type than the file extension tells us, the content takes precedence.
*/

use std::path::Path;

extern crate little_exif_0_6_14;
extern crate little_exif_0_6_20;
extern crate little_exif;

#[test]
#[should_panic (expected = "called `Option::unwrap()` on a `None` value")]
fn
read_exif_data_old_fails()
{
    let heic_path = Path::new("resources/issue_000070/IMG_2762.HEIC");
    // let jpeg_path = Path::new("resources/issue_000070/IMG_2762.JPEG");

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_14::metadata::Metadata::new_from_path(heic_path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 43);
}

#[test]
fn
read_exif_data_old_works_because_it_is_a_jpeg()
{
    // let heic_path = Path::new("resources/issue_000070/IMG_2762.HEIC");
    let jpeg_path = Path::new("resources/issue_000070/IMG_2762.JPEG");

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_14::metadata::Metadata::new_from_path(jpeg_path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 43);
}

#[test]
fn
read_exif_data_fixed()
{
    let heic_path = Path::new("resources/issue_000070/IMG_2762.HEIC");
    // let jpeg_path = Path::new("resources/issue_000070/IMG_2762.JPEG");

    let mut tag_counter = 0;
    for _ in &little_exif_0_6_20::metadata::Metadata::new_from_path(heic_path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 43);
}

#[test]
fn
read_exif_data_current()
{
    let heic_path = Path::new("resources/issue_000070/IMG_2762.HEIC");
    // let jpeg_path = Path::new("resources/issue_000070/IMG_2762.JPEG");

    let mut tag_counter = 0;
    for _ in &little_exif::metadata::Metadata::new_from_path(heic_path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 43);
}