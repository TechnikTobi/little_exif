use std::env;
use std::fs;
use std::path::Path;
use std::process::exit;
use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;

// Example command
// cargo run --example fuzz_test -- "[82,73,70,70,24,0,0,0,74,70,56,69,0,0,0,13,0,1,0,8,0,8,0,0,0,0,0,0,0,0,0,0]"
// or
// cargo run --example fuzz_test -- test_images/sample.jpg
// or
// cargo run --example fuzz_test -- test_images/

fn parse_bytes_from_string(s: &str) -> Vec<u8> {
    let mut bytes = Vec::new();

    for token in s.split(|c: char| !c.is_ascii_digit()) {
        if token.is_empty() {
            continue;
        }
        if let Ok(n) = token.parse::<u32>() {
            if n <= 255 {
                bytes.push(n as u8);
            }
        }
    }

    bytes
}

fn get_bytes_from_payload(payload: &str) -> Vec<(Vec<u8>, String)> {
    let p = Path::new(payload);
    if p.is_file() {
        if let Ok(data) = fs::read(p) {
            return vec![(data, payload.to_string())];
        } else {
            eprintln!("Failed to read file '{}', falling back to treating payload as raw bytes", payload);
        }
    } else if p.is_dir() {
        let results: Vec<(Vec<u8>, String)> = match fs::read_dir(p) {
            Ok(entries) => entries.flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        fs::read(&path).ok().map(|data| (data, path.display().to_string()))
                    } else {
                        None
                    }
                })
                .collect(),
            Err(_) => Vec::new(),
        };
        return results;
    }

    let parsed = parse_bytes_from_string(payload);
    if !parsed.is_empty() {
        return vec![(parsed, "Custom byte array".to_string())];
    }

    eprintln!("Failed to parse any bytes from payload '{}'", payload);
    exit(1);
}

fn run_for_file_types(data: &[u8]) {
    let file_types = [
        FileExtension::NAKED_JXL,
        FileExtension::JXL,
        FileExtension::JPEG,
        FileExtension::PNG { as_zTXt_chunk: false },
        FileExtension::PNG { as_zTXt_chunk: true },
        FileExtension::TIFF,
        FileExtension::WEBP,
        FileExtension::HEIF,
    ];

    for file_type in file_types.iter() {
        match Metadata::new_from_vec(&data.to_vec(), *file_type) {
            Ok(metadata) => {
                // process_metadata_strict(data, metadata.clone(), *file_type);
                process_metadata_non_strict(data, metadata.clone(), *file_type);
                clean_metadata(metadata);
            }
            Err(e) => {
                eprintln!("new_from_vec error for {:?}: {}", file_type, e);
            }
        }
    }
}

// TODO - first fix all problems in non-strict mode
#[allow(dead_code)]
fn process_metadata_strict(initial_data: &[u8], metadata: Metadata, file_extension: FileExtension) {
    let mut file_data = initial_data.to_vec();

    metadata.clone().write_to_vec(&mut file_data, file_extension).expect("Writing metadata to same buffer from which it was read should newer fail");
    let new_metadata = Metadata::new_from_vec(&file_data, file_extension).expect("Reading metadata from buffer after writing should not fail in strict mode");
    let tags_old: Vec<_> = metadata.into_iter().cloned().collect();
    let tags_new: Vec<_> = new_metadata.into_iter().cloned().collect();

    assert_eq!(tags_old, tags_new, "Metadata read from buffer after writing should be identical to the original metadata");
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

    assert_eq!(tags_old, tags_new, "Metadata read from buffer after writing should be identical to the original metadata");
}

fn clean_metadata(mut metadata: Metadata) {
    let all_tags = metadata.clone().into_iter().cloned().collect::<Vec<_>>();
    for tag in all_tags {
        metadata.remove_tag(tag);
    }
    let available_tags = metadata.into_iter().cloned().collect::<Vec<_>>();
    assert_eq!(available_tags, Vec::new(), "All tags should have been removed from metadata");
}

fn main() {
    let mut args = env::args();
    let _program = args.next();
    let payload = match args.next() {
        Some(p) => p,
        None => {
            eprintln!("Usage: cargo run --example fuzz_test -- \"[82,73,70]\"");
            return;
        }
    };

    let data = get_bytes_from_payload(&payload);
    for (bytes, name) in data {
        println!("\nProcessing \"{}\"", name);
        run_for_file_types(bytes.as_slice());
    }
}
