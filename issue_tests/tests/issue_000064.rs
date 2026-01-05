// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// including those who posted code snippets and/or image files for debugging
// and testing purposes to the respective issue on GitHub.
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/*
When I entered this picture, the following error occurred
remove_private_exif error: Metadata::new_from_vec error: failed to fill whole buffer
*/

/*
Solved: Problem was that originally, JXL was only supported with uncompressed
metadata. This file however contains brotli-compressed data that is also
in a different type of box and needs to be decompressed.
See commits:
- f6531e357b95709e0e01a89d4d2ccee68fc0c058: Working on issue #64
- 37decccedd3410e1b5a456b44ee3349f4a370355: Bugfix for issue #64

Later on I realized I only fixed support for vec-based operations but not
file-based, fixed in this commit:
- 209e4f42489d8f37f5bff7360e489b34905e493e
*/

use std::path::Path;

extern crate little_exif;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;

#[test]
fn replace_exif_tag_values() {
    let path_orig = Path::new("resources/issue_000064/a.jxl");
    let path_copy = Path::new("resources/issue_000064/a_copy.jxl");

    let content = std::fs::read(path_orig).unwrap();
    let new_content = remove_private_exif(&content).unwrap();

    std::fs::write(path_copy, new_content).unwrap();

    let mut orig_tag_counter = 0;
    let orig_metadata = Metadata::new_from_path(path_orig).unwrap();
    for tag in &orig_metadata {
        if tag.as_u16() == 0x9004 {
            assert_eq!(
                tag.value_as_u8_vec(&orig_metadata.get_endian()),
                vec![50, 48, 50, 53, 58, 48, 55, 58, 48, 49, 32, 49, 54, 58, 48, 49, 58, 49, 55, 0] // 2025:07:01 16:01:17
            );
        }
        orig_tag_counter += 1;
    }

    let mut copy_tag_counter = 0;
    for tag in &Metadata::new_from_path(path_copy).unwrap() {
        if tag.as_u16() == 0x9004 {
            assert_eq!(
                tag.value_as_u8_vec(&orig_metadata.get_endian()),
                vec![0] // EMPTY STRING
            );
        }
        copy_tag_counter += 1;
    }

    // The actual number of
    assert_eq!(orig_tag_counter, 67);
    assert_eq!(copy_tag_counter, 67);
}

pub fn remove_private_exif(image_vec: &[u8]) -> Result<Vec<u8>, u8> {
    let mut output_vec = image_vec.to_vec();

    let file_type = if let Some(file_type) = guess_image_type(&output_vec) {
        file_type
    } else {
        println!("only process jpeg/png/webp, returning original");
        return Ok(output_vec);
    };

    println!("output_vec length: {}", output_vec.len());

    let mut metadata = match Metadata::new_from_vec(&output_vec, file_type) {
        Ok(m) => m,
        Err(e) => {
            println!("Metadata::new_from_vec error: {}", e);
            return Ok(output_vec);
        }
    };

    println!("exif metadata found, proceeding to clear tags");

    let tags_to_clear = [
        ExifTag::CreateDate(String::new()),
        ExifTag::ModifyDate(String::new()),
        ExifTag::DateTimeOriginal(String::new()),
        ExifTag::OffsetTime(String::new()),
        ExifTag::OffsetTimeOriginal(String::new()),
        ExifTag::OffsetTimeDigitized(String::new()),
        ExifTag::SubSecTime(String::new()),
        ExifTag::SubSecTimeOriginal(String::new()),
        ExifTag::SubSecTimeDigitized(String::new()),
        ExifTag::GPSInfo(Vec::new()),
    ];

    for tag_to_clear in tags_to_clear.iter() {
        if metadata.get_tag(tag_to_clear).count() > 0 {
            match tag_to_clear {
                ExifTag::CreateDate(_) => {
                    println!("Clearing ExifTag::CreateDate");
                    metadata.set_tag(ExifTag::CreateDate("".to_string()))
                }
                ExifTag::ModifyDate(_) => {
                    println!("Clearing ExifTag::ModifyDate");
                    metadata.set_tag(ExifTag::ModifyDate("".to_string()))
                }
                ExifTag::DateTimeOriginal(_) => {
                    println!("Clearing ExifTag::DateTimeOriginal");
                    metadata.set_tag(ExifTag::DateTimeOriginal("".to_string()))
                }
                ExifTag::OffsetTime(_) => {
                    println!("Clearing ExifTag::OffsetTime");
                    metadata.set_tag(ExifTag::OffsetTime("".to_string()))
                }
                ExifTag::OffsetTimeOriginal(_) => {
                    println!("Clearing ExifTag::OffsetTimeOriginal");
                    metadata.set_tag(ExifTag::OffsetTimeOriginal("".to_string()))
                }
                ExifTag::OffsetTimeDigitized(_) => {
                    println!("Clearing ExifTag::OffsetTimeDigitized");
                    metadata.set_tag(ExifTag::OffsetTimeDigitized("".to_string()))
                }
                ExifTag::SubSecTime(_) => {
                    println!("Clearing ExifTag::SubSecTime");
                    metadata.set_tag(ExifTag::SubSecTime("".to_string()))
                }
                ExifTag::SubSecTimeOriginal(_) => {
                    println!("Clearing ExifTag::SubSecTimeOriginal");
                    metadata.set_tag(ExifTag::SubSecTimeOriginal("".to_string()))
                }
                ExifTag::SubSecTimeDigitized(_) => {
                    println!("Clearing ExifTag::SubSecTimeDigitized");
                    metadata.set_tag(ExifTag::SubSecTimeDigitized("".to_string()))
                }
                ExifTag::GPSInfo(_) => {
                    println!("Clearing ExifTag::GPSInfo");
                    metadata.set_tag(ExifTag::GPSInfo(Vec::new()))
                }
                _ => {}
            }
        }
    }

    println!("PRINT TAGS:");
    for tag in &metadata {
        println!("{:?}", tag);
    }

    println!("Metadata::guess_image_type called 2");
    let file_type2 = match guess_image_type(&output_vec) {
        Some(FileExtension::PNG { .. }) => FileExtension::PNG {
            as_zTXt_chunk: false,
        },
        Some(FileExtension::JPEG) => FileExtension::JPEG,
        Some(FileExtension::WEBP) => FileExtension::WEBP,
        _ => file_type,
    };

    println!("detected file type: {:?}", file_type2);
    match metadata.write_to_vec(&mut output_vec, file_type2) {
        Ok(_) => {
            println!("write_to_vec success");
            Ok(output_vec)
        }
        Err(e) => {
            println!("write_to_vec error: {}", e);
            Ok(output_vec)
        }
    }
}

fn guess_image_type(data: &[u8]) -> Option<FileExtension> {
    if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Some(FileExtension::JPEG);
    }

    if data.len() >= 8 && data[0..8] == [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return Some(FileExtension::PNG {
            as_zTXt_chunk: false,
        });
    }

    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some(FileExtension::WEBP);
    }

    if data.len() >= 8 && &data[4..8] == b"JXL " {
        return Some(FileExtension::JXL);
    }

    None
}
