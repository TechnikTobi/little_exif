// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
trying to extract GPS infos from a picture I know it has GPS infos, I don't get any
*/

/*
Solved:
Wrote a new decoder as part of release 0.6.0
*/

use std::path::Path;

#[test]
fn
read_gps_latitude_current()
{
    let path     = Path::new("resources/issue_000043/381105553-cb23b235-9905-440a-a85c-13f44d5818d4.jpg");
    let metadata = little_exif::metadata::Metadata::new_from_path(path).unwrap();
    let mut tag  = metadata.get_tag(&little_exif::exif_tag::ExifTag::GPSLatitude(Vec::new()));

    if let Some(unwrapped_tag) = tag.next()
    {
        println!("{:?}", unwrapped_tag);
    }
    else
    {
        panic!("No GPS tag found");
    }
}
