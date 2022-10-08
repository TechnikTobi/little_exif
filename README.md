# little\_exif
A little library for reading and writing EXIF data in pure Rust.

[![version-badge][]][version]&nbsp;
[![license-badge-MIT][]][license-MIT]&nbsp;
[![license-badge-Apache2][]][license-Apache2]&nbsp;
[![license-badge][]][license]&nbsp;

[version-badge]: https://img.shields.io/crates/v/little_exif.svg
[version]: https://crates.io/crates/little_exif
[license-badge-MIT]: https://img.shields.io/badge/license-MIT-blue
[license-MIT]: https://github.com/TechnikTobi/little_exif/blob/main/LICENSE-MIT
[license-badge-Apache2]: https://img.shields.io/badge/license-Apache--2.0-blue
[license-Apache2]: https://github.com/TechnikTobi/little_exif/blob/main/LICENSE-APACHE
[license-badge]: https://img.shields.io/crates/l/little_exif.svg
[license]: https://github.com/TechnikTobi/little_exif#license

## Example

```rust
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

let image_path = std::path::Path::new("image.png");
let mut metadata = Metadata::new_from_path(&image_path);

metadata.set_tag(
	ExifTag::ImageDescription("Hello World!".to_string())
);

metadata.write_to_file(&image_path);
```

## License
Licensed under either of
- Apache License, Version 2.0 (See [LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT License (See [LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.
