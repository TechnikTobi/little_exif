// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
When I was reading the photo, the Metadata:: new_from_path method panicked
*/

/*
Solved:
Can't remember :(
*/

use std::path::Path;

extern crate little_exif;
extern crate little_exif_0_3_1;
extern crate little_exif_0_5_0_beta_2;

#[test]
#[should_panic(expected = "assertion failed: u8_vec.len() == 4")]
fn read_exif_data_fails() {
    let jpg_path = Path::new("resources/issue_000006/309465781-9420afb8-2a57-4bae-a188-a719f6d62b1f.JPG");

    match little_exif_0_3_1::metadata::Metadata::new_from_path(jpg_path) {
        Ok(metadata) => {
            println!("metadata--{:#?}", metadata.data());
        }
        Err(err) => {
            println!("err ---{:#?}", err);
        }
    }
}

#[test]
#[should_panic(expected = "from_u8_vec: Mangled EXIF data encountered!")]
fn read_exif_data_last_fail() {
    let jpg_path = Path::new("resources/issue_000006/309465781-9420afb8-2a57-4bae-a188-a719f6d62b1f.JPG");

    match little_exif_0_5_0_beta_1::metadata::Metadata::new_from_path(jpg_path) {
        Ok(metadata) => {
            println!("metadata--{:#?}", metadata.data());
        }
        Err(err) => {
            println!("err ---{:#?}", err);
        }
    }
}

#[test]
fn read_exif_data_fixed() {
    let jpg_path = Path::new("resources/issue_000006/309465781-9420afb8-2a57-4bae-a188-a719f6d62b1f.JPG");

    match little_exif_0_5_0_beta_2::metadata::Metadata::new_from_path(jpg_path) {
        Ok(metadata) => {
            println!("metadata--{:#?}", metadata.data());
        }
        Err(err) => {
            println!("err ---{:#?}", err);
        }
    }
}

#[test]
fn read_exif_data_current() {
    let jpg_path = Path::new("resources/issue_000006/309465781-9420afb8-2a57-4bae-a188-a719f6d62b1f.JPG");

    match little_exif::metadata::Metadata::new_from_path(jpg_path) {
        Ok(metadata) => {
            println!(
                "metadata--{:#?}",
                metadata.as_u8_vec(little_exif::filetype::FileExtension::JPEG)
            );
        }
        Err(err) => {
            println!("err ---{:#?}", err);
        }
    }
}
