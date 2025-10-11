// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
Problem statement:
Some `STRING` values don't round-trip correctly when reading and then saving EXIF data, because the serialization [uses `String::as_bytes`](https://github.com/TechnikTobi/little_exif/blob/main/src/u8conversion.rs#L100) whereas the de-serialization [converts the bytes to `char` directly](https://github.com/TechnikTobi/little_exif/blob/main/src/u8conversion.rs#L124) (implicitly assuming the input is plain ASCII).

Although the EXIF standard technically says these fields should be ASCII, it's not uncommon to store UTF-8 in them and in any case having the library read back the same thing it serialized makes sense. Should de-serialization use `String::from_utf8` instead?

This TIFF image reproduces the issue (it has non-ASCII UTF-8 in the `Make` EXIF tag): [20160513-A0012+001.tiff.gz](https://github.com/user-attachments/files/21108762/20160513-A0012%2B001.tiff.gz)
*/

/*
Solved:
Changed U8 string conversion
*/

use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

#[test]
fn
string_tag_round_trip()
{
    let path_orig = Path::new("resources/issue_000065/20160513-A0012+001.tiff");
    let path_copy = Path::new("resources/issue_000065/20160513-A0012+001_copy.tiff");

    // Remove file from previous run and replace it with fresh copy
    if let Err(error) = remove_file(&path_copy)
    {
        println!("{}", error);
    }
    copy(&path_orig, &path_copy).unwrap();

    let mut metadata = Metadata::new_from_path(path_orig).unwrap();

    // Contains the string "Voigtländer" -> Umlaut!
    let orig_metadata = Metadata::new_from_path(path_orig).unwrap();
    let orig_make_tag = orig_metadata.get_tag(&ExifTag::Make("".to_string())).next();

    // "Round-trip", updating some tag so something changes
    metadata.set_tag(
        ExifTag::LensMake("Colour".to_string())
    );
    metadata.write_to_file(path_copy).unwrap();

    // Read again
    let metadata_copy = Metadata::new_from_path(path_copy).unwrap();

    let copy_make_tag = metadata_copy.get_tag(&ExifTag::Make("".to_string())).next();

    assert_eq!(
        orig_make_tag.unwrap().value_as_u8_vec(&orig_metadata.get_endian()), 
        copy_make_tag.unwrap().value_as_u8_vec(&metadata_copy.get_endian())
    );
}