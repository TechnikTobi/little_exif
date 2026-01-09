// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
this PNG file causes little_exif to panic when running clear_metadata()
*/

/*
Solution: Can't recall :( Maybe something regarding offsets?
*/

use std::path::Path;
use std::fs::remove_file;
use std::fs::copy;
use std::fs::read;

#[test]
fn
issue_000055_clear_exif_data_current()
{
    let png_path = Path::new("resources/issue_000055/437726296-e38cf0e2-93c9-4e43-9786-6003e167d39c.png");
    let cpy_path = Path::new("resources/issue_000055/437726296-e38cf0e2-93c9-4e43-9786-6003e167d39c_copy2.png");

    if let Err(error) = remove_file(cpy_path)
    {
        println!("Could not delete file: {}", error);
    }
    copy(png_path, cpy_path).unwrap();

    let mut image_data = read(png_path).unwrap();

    little_exif::metadata::Metadata::clear_metadata(
        &mut image_data, 
        little_exif::filetype::FileExtension::PNG { as_zTXt_chunk: false }
    ).unwrap();

    std::fs::write(
        cpy_path.as_os_str(),
        image_data
    ).unwrap();
}
