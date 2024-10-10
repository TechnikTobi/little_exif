// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

//! # little_exif
//! A small crate for reading and writing (some) EXIF data, written entirely in Rust. Currently supports
//! - .png 
//! - .jp(e)g 
//! - .jxl
//! - .webp
//! files and a few dozen tags in IFD0 and ExifIFD. 
//! 
//! Interaction is done via the [`Metadata`](metadata/struct.Metadata.html) struct and the [`ExifTag`](exif_tag/enum.ExifTag.html) enum.
//!
//! # Usage
//! ## Write EXIF data
//! ```no_run
//! use little_exif::metadata::Metadata;
//! use little_exif::exif_tag::ExifTag;
//! 
//! let mut metadata = Metadata::new();
//! metadata.set_tag(
//!     ExifTag::ImageDescription("Hello World!".to_string())
//! );
//! metadata.write_to_file(std::path::Path::new("image.png"));
//! ```

#![forbid(unsafe_code)]
#![crate_type = "lib"]
#![crate_name = "little_exif"]

mod general_file_io;
mod png;
mod jpg;
mod jxl;
mod tiff;
mod webp;
mod util;

pub mod endian;
pub mod rational;
pub mod u8conversion;
pub mod exif_tag;
pub mod exif_tag_format;
pub mod filetype;
pub mod metadata;