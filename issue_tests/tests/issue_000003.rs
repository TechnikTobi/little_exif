// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Original problem:
Code that cause panic

match little_exif::metadata::Metadata::new_from_path(&file_entry.path) {
  Ok(mt) => {
      for tag in mt.data() {
          dbg!(tag);
      }
  }
  Err(e) => {
      println!("Error: {:?}", e);
  }
}
Backtrace

		// Assert that we have enough data to unpack
		assert!(2 + IFD_ENTRY_LENGTH as usize * number_of_entries as usize + IFD_END.len() <= encoded_data.len());

exiftool only warning - properly parse entire file

[minor] Skipped unknown 7 bytes after JPEG APP1 segment
*/

/*
Solved: The given JPG file is corrupted; The application segment APP1 is shorter than described (given length: 0xCAE bytes; actual length: 0xCA3 bytes). Can't even be opened using Preview under macOS.
*/

use std::path::Path;

extern crate little_exif_0_3_0;
extern crate little_exif;

#[test]
#[should_panic (expected = "assertion failed: 2 + IFD_ENTRY_LENGTH as usize * number_of_entries as usize + IFD_END.len() <=\n    encoded_data.len()")]
fn
read_exif_data_fails()
{
    let path = Path::new("resources/issue_000003/301581895-a7b4390a-e9f4-46cc-b04f-eb1ba677204c.jpg");

    let mut tag_counter = 0;

    for _ in little_exif_0_3_0::metadata::Metadata::new_from_path(path).unwrap().data()
    {
        tag_counter += 1;
    }

    assert_ne!(tag_counter, 0);
}

#[test]
#[should_panic (expected = "called `Result::unwrap()` on an `Err` value: Custom { kind: Other, error: \"Could not decode SubIFD GPS:\\n  Not enough data to decode IFD! Required: 6150 Available: 124\" }")]
fn
read_exif_data_current_still_fails()
{
    let path = Path::new("resources/issue_000003/301581895-a7b4390a-e9f4-46cc-b04f-eb1ba677204c.jpg");

    let mut tag_counter = 0;

    for _ in &little_exif::metadata::Metadata::new_from_path(path).unwrap()
    {
        tag_counter += 1;
    }

    assert_ne!(tag_counter, 0);
}