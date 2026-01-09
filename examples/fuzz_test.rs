use std::env;

use little_exif::filetype::FileExtension;
use little_exif::metadata::Metadata;

// Example command
// cargo run --example fuzz_test -- "[82,73,70,70,24,0,0,0,74,70,56,69,0,0,0,13,0,1,0,8,0,8,0,0,0,0,0,0,0,0,0,0]"

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

fn get_bytes_from_payload(payload: &str) -> Vec<u8> {
    let parsed = parse_bytes_from_string(payload);
    if !parsed.is_empty() {
        parsed
    } else {
        payload.as_bytes().to_vec()
    }
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
        match Metadata::new_from_vec(data, *file_type) {
            Ok(_metadata) => {

            }
            Err(e) => {
                eprintln!("new_from_vec error for {:?}: {}", file_type, e);
            }
        }
    }
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
    run_for_file_types(&data);
}
