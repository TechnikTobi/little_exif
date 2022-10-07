use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;
use little_exif::exif_tag::ExifTagGroup;

#[test]
fn
new()
{
	let _ = Metadata::new();
}

#[test]
fn
new_from_path()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample2.png"));
}

#[test]
#[should_panic(expected = "Can't read Metadata - File does not exist!")]
fn
new_from_path_panic_not_existant()
{
	let _ = Metadata::new_from_path(Path::new("sample2.png"));
}

#[test]
#[should_panic(expected = "Can't get extension from given path!")]
fn
new_from_path_panic_no_extension()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample0"));
}

#[test]
#[should_panic(expected = "Can't read Metadata - Unsupported file type!")]
fn
new_from_path_panic_not_supported()
{
	let _ = Metadata::new_from_path(Path::new("tests/sample1.txt"));
}