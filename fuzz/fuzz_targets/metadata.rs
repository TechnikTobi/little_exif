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
        if let Ok(metadata) = metadata {
            // process_metadata_strict(data, metadata.clone(), *file_type);
            process_metadata_non_strict(data, metadata.clone(), *file_type);
            clean_metadata(metadata);
        }
        properly_read_metadata = true;
    }

    if properly_read_metadata {
        Corpus::Keep
    } else {
        Corpus::Reject
    }
});

// TODO - first fix all problems in non-strict mode
#[allow(dead_code)]
fn process_metadata_strict(initial_data: &[u8], metadata: Metadata, file_extension: FileExtension) {
    let mut file_data = initial_data.to_vec();

    metadata.clone().write_to_vec(&mut file_data, file_extension).expect("Writing metadata to same buffer from which it was read should newer fail");
    let new_metadata = Metadata::new_from_vec(&file_data, file_extension).expect("Reading metadata from buffer after writing should not fail in strict mode");
    let tags_old: Vec<_> = metadata.into_iter().cloned().collect();
    let tags_new: Vec<_> = new_metadata.into_iter().cloned().collect();

    pretty_assertions::assert_eq!(tags_old, tags_new, "Metadata read from buffer after writing should be identical to the original metadata");
}

fn process_metadata_non_strict(initial_data: &[u8], metadata: Metadata, file_extension: FileExtension) {
    let mut file_data = initial_data.to_vec();

    if metadata.clone().write_to_vec(&mut file_data, file_extension).is_err() {
        return;
    }
    let Ok(new_metadata) = Metadata::new_from_vec(&file_data, file_extension) else {
        return;
    };
    let tags_old: Vec<_> = metadata.into_iter().cloned().collect();
    let tags_new: Vec<_> = new_metadata.into_iter().cloned().collect();

    pretty_assertions::assert_eq!(tags_old, tags_new, "Metadata read from buffer after writing should be identical to the original metadata");
}

fn clean_metadata(mut metadata: Metadata) {
    let all_tags = metadata.clone().into_iter().cloned().collect::<Vec<_>>();
    for tag in all_tags {
        metadata.remove_tag(tag);
    }
    let available_tags = metadata.into_iter().cloned().collect::<Vec<_>>();
    pretty_assertions::assert_eq!(available_tags, Vec::new(), "All tags should have been removed from metadata");
}