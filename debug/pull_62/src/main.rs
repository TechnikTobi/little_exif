extern crate little_exif;
use avif_serialize::Aviffy;
use little_exif::metadata::Metadata as ExifMetadata;
use little_exif::exif_tag::ExifTag;
use little_exif::rational::uR64;

use avif_serialize;

use std::fs::File;
use std::io::Write;

fn 
metadata_to_exif
() 
-> Result<Vec<u8>, std::io::Error>
{
    let mut metadata_exif = ExifMetadata::new();

    metadata_exif.set_tag(ExifTag::Software("dora-rs".to_string()));

    metadata_exif.set_tag(ExifTag::FocalLength(
        vec![
            uR64 
            {
                nominator: 138 as u32,
                denominator: 1,
            }
        ]
    ));

    let vector = metadata_exif.as_u8_vec(little_exif::filetype::FileExtension::HEIF)?;
    return Ok(vector);
}

use image::{RgbImage, Rgb};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Image dimensions
    let width = 512u32;
    let height = 256u32;

    // Create an RGB gradient image (red to blue)
    let mut img = RgbImage::new(width, height);
    for x in 0..width {
        let r = ((width - x - 1) * 255 / (width - 1)) as u8;
        let b = (x * 255 / (width - 1)) as u8;
        for y in 0..height {
            img.put_pixel(x, y, Rgb([r, 200, b]));
        }
    }

    // Encode to AVIF/HEIF in memory
    let mut aviffy = Aviffy::new();
    aviffy.set_bit_depth(8);
    aviffy.set_full_color_range(true);
    aviffy.set_exif(metadata_to_exif()?);
    // aviffy.set_exif(vec![
    //     0x49,
    //     0x49,
    //     0x2a,
    //     0x00,
    //     0x08,
    //     0x00,
    //     0x00,
    //     0x00,

    //     0x00,
    //     0x00,
    // ]);
    // aviffy.set_exif(Vec::new());

    let avif_bytes = aviffy.to_vec(
        &img.into_raw(), // &planar,
        None,
        width,
        height,
        8,
    );

    // Write to disk as ".heif"
    let mut file = File::create("gradient.heif")?;
    file.write_all(&avif_bytes)?;

    println!("Wrote gradient.heif!");

    Ok(())
}