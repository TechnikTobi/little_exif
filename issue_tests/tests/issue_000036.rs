// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
moreover with some files, I get
Error during decoding: Error { kind: UnexpectedEof, message: "failed to fill whole buffer" }
*/

/*
Solved:
There was a typo, see related commit:
- 8965456229fb6e366dd24d5f67d730ee6a7cc086
*/

use std::path::Path;

#[test]
#[should_panic (expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"No EXIF data found!\" }")]
fn
read_exif_data_current()
{
    let path = Path::new("resources/issue_000036/IMG-20180904-WA0002.jpg");

    for tag in &little_exif::metadata::Metadata::new_from_path(path).unwrap()
    {
        println!("{:?}", tag);
    }
}
