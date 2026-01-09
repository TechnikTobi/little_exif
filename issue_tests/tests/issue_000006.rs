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

#[test]
fn
read_exif_data_current()
{
    let jpg_path = Path::new("resources/issue_000006/309465781-9420afb8-2a57-4bae-a188-a719f6d62b1f.JPG");

    match little_exif::metadata::Metadata::new_from_path(jpg_path) {
        Ok(metadata) => {
            println!("metadata--{:#?}", metadata.as_u8_vec(little_exif::filetype::FileExtension::JPEG));
        }
        Err(err) => {
            println!("err ---{:#?}", err);
        }
    }
}