# little\_exif

A little library for reading and writing EXIF data in pure Rust.

[![Build & Test](https://github.com/TechnikTobi/little_exif/actions/workflows/rust.yml/badge.svg)](https://github.com/TechnikTobi/little_exif/actions/workflows/rust.yml)&nbsp;
[![version-badge][]][version]&nbsp;
[![license-badge][]][license]&nbsp;

[version-badge]: https://img.shields.io/crates/v/little_exif.svg
[version]: https://crates.io/crates/little_exif
[license-badge]: https://img.shields.io/crates/l/little_exif.svg
[license]: https://github.com/TechnikTobi/little_exif#license

## Supported Formats
- JPEG / JPG
- JXL
- HEIF / HEIC / HIF
- PNG
- TIFF
- WebP (only lossless and extended)

Your required format is not listed here or you've run into a problem with a file that should be supported? Open up a new issue (ideally with an example image for reproduction in case of a problem) and I'll take a look!

## Example

If the image is stored in a file, located at some given path:

```rust
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

let image_path = std::path::Path::new("image.png");
let mut metadata = Metadata::new_from_path(&image_path);

metadata.set_tag(
    ExifTag::ImageDescription("Hello World!".to_string())
);

metadata.write_to_file(&image_path)?;
```

Alternatively, if the image is stored in a ```Vec<u8>``` variable:

```rust
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::filetype::FileExtension;

let file_type = FileExtension::JPEG;
let mut metadata = Metadata::new_from_vec(&image_vector, file_type);

metadata.set_tag(
    ExifTag::ImageDescription("Hello World!".to_string())
);

metadata.write_to_vec(&mut image_vector, file_type)?;
```

## Testing

To run the tests from a specific file, use e.g.

```bash
cargo test --test issue_000002
```

To run a single test from that file use

```bash
cargo test --test issue_000002 read_and_write_exif_data_1
```

## FAQ

### I tried writing the ImageDescription tag on a JPEG file, but it does not show up. Why?

This could be due to the such called APP12 or APP13 segment stored in the JPEG, likely caused by editing the file using e.g. Photoshop. These segments may store data that image viewers also interpret as an ImageDescription, overriding the EXIF tag. Right now, little_exif can't edit these segments. As a workaround, the functions ```clear_app12_segment``` and ```clear_app13_segment``` can remove these areas from the JPEG:

```rust
// File in a Vec<u8>
Metadata::clear_app12_segment(&mut file_content, file_extension)?;
Metadata::clear_app13_segment(&mut file_content, file_extension)?;

// File at a given path
Metadata::file_clear_app12_segment(&given_path)?;
Metadata::file_clear_app13_segment(&given_path)?;
```


## License

Licensed under either

- Apache License, Version 2.0 (See [LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0) or
- MIT License (See [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
