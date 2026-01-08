#![no_main]

use libfuzzer_sys::{fuzz_target, Corpus};
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;

fuzz_target!(|data: &[u8]| -> Corpus {
    let file_types = [
        FileExtension::NAKED_JXL,
        FileExtension::JXL,
        FileExtension::JPEG,
        FileExtension::PNG {
            as_zTXt_chunk: false,
        },
        FileExtension::PNG {
            as_zTXt_chunk: true,
        },
        FileExtension::TIFF,
        FileExtension::WEBP,
        FileExtension::HEIF,
    ];

    let mut properly_read_metadata = false;
    for file_type in file_types.iter() {
        let metadata = Metadata::new_from_vec(&data.to_vec(), *file_type);
        if metadata.is_ok() {
            properly_read_metadata = true;
        }
    }

    if properly_read_metadata {
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});
