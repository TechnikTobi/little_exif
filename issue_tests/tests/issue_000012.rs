// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
I tried to come back to this issue, no particular order:
- this file did not read any data on 0.4.0, did not test on 0.5.1 due to:
- after rebasing my fork I realised that I can't use the latest version of the library
we currently use stable Rust 1.77.2 and this library uses a (then) unstable feature
more info: bug: needlessly modifying images' ICC profiles stoatchat/stoatchat#347 (comment)
*/

/*
Solved:
Can't recall :( However, the minimum Rust version for little_exif has been
updated to 1.65.0
*/

use std::path::Path;

/*
#[test]
#[should_panic (expected = "assertion `left != right` failed\n  left: 0\n right: 0")]
fn
read_exif_data_fails()
{
    let path = Path::new("resources/issue_000012/PXL_20241007_142045194.jpg");

    let mut tag_counter = 0;

    for _ in little_exif_0_4_0::metadata::Metadata::new_from_path(path).unwrap().data()
    {
        tag_counter += 1;
    }

    assert_ne!(tag_counter, 0);
}

#[test]
fn
read_exif_data_fixed()
{
    let path = Path::new("resources/issue_000012/PXL_20241007_142045194.jpg");

    let mut tag_counter = 0;

    for _ in &little_exif_0_6_0::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 71);
}
*/

#[test]
fn
read_exif_data_current()
{
    let path = Path::new("resources/issue_000012/PXL_20241007_142045194.jpg");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
    }

    assert_eq!(tag_counter, 71);
}
