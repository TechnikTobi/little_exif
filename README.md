# little\_exif
[![version-badge][]][version]&nbsp;
[![license-badge-MIT][]][license-MIT]&nbsp;
[![license-badge-Apache2][]][license-Apache2]&nbsp;

[version-badge]: https://img.shields.io/crates/v/little_exif.svg
[version]: https://crates.io/crates/little_exif
[license-badge-MIT]: https://img.shields.io/badge/license-MIT-blue
[license-MIT]: https://github.com/TechnikTobi/little_exif/blob/main/LICENSE-MIT
[license-badge-Apache2]: https://img.shields.io/badge/license-Apache--2.0-blue
[license-Apache2]: https://github.com/TechnikTobi/little_exif/blob/main/LICENSE-APACHE

## A little library for reading and writing EXIF data in pure Rust.



## Example

```
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn
main()
{

	let image_path = std::path::Path::new("image.png");
	let mut metadata = Metadata::new_from_path(&image_path);

	metadata.set_tag(
		ExifTag::ImageDescription("Hello World!".to_string())
	);

	metadata.write_to_file(&image_path);
}
```
