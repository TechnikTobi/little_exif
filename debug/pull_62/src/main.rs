extern crate little_exif;
use avif_serialize::Aviffy;
use little_exif::metadata::Metadata as ExifMetadata;
use little_exif::exif_tag::ExifTag;
use little_exif::rational::uR64;

use avif_serialize;
use rav1e::prelude::v_frame::plane::PlaneOffset;

use std::fs::File;
use std::io::Write;

use avif_serialize::serialize_to_vec;
use rav1e::Config;
use rav1e::prelude::*;

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

// fn main() 
// {
//     println!("Hello, world!");

//     let data: Vec<u8> = vec![0u8; 20_000];

//     // metadata.parameters.insert(
//     //     "encoding".to_string(),
//     //     Parameter::String("avif".to_string()),
//     // );

//     let mut aviffy = avif_serialize::Aviffy::new();
//     aviffy
//         .full_color_range(false)
//         .set_seq_profile(0)
//         .set_monochrome(true);

//     let aviffy = if let Ok(exif) =
//         metadata_to_exif() // <- this is where as_u8_vec is called
//     {
//         aviffy.set_exif(exif)
//     } else {
//         &mut aviffy
//     };

//     let data = aviffy.to_vec( // <- this gets you the image with exif
//         &data,
//         None,
//         200,
//         100,
//         8,
//     ); 
// }





/*
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
            img.put_pixel(x, y, Rgb([r, 0, b]));
        }
    }

    // Get raw RGB data (pixels in row-major, RGBRGB...)
    let raw_pixels = img.into_raw();

    // Encode to AVIF/HEIF in memory
    let mut aviffy = avif_serialize::Aviffy::new();
    aviffy.set_bit_depth(8);
    aviffy.set_full_color_range(true);

    let avif_bytes = aviffy.to_vec(
        &raw_pixels,
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
*/



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

    // // Convert and expand packed RGB to packed RGBA (A=255)
    // let rgb_packed = img.into_raw();
    // let mut rgba_packed = Vec::with_capacity(width as usize * height as usize * 4);
    // for chunk in rgb_packed.chunks(3) {
    //     rgba_packed.extend_from_slice(chunk);    // R, G, B
    //     rgba_packed.push(255);                   // A = 255 (opaque)
    // }

    // // Now convert packed RGBA to planar RGBA
    // let pixels = rgba_packed;
    // let num_pixels = (width * height) as usize;
    // let mut planar: Vec<u8> = Vec::with_capacity(num_pixels * 4);
    // // R
    // planar.extend(pixels.iter().step_by(4));
    // // G
    // planar.extend(pixels.iter().skip(1).step_by(4));
    // // B
    // planar.extend(pixels.iter().skip(2).step_by(4));
    // // A
    // planar.extend(pixels.iter().skip(3).step_by(4));

    // Encode to AVIF/HEIF in memory
    let mut aviffy = Aviffy::new();
    aviffy.set_bit_depth(8);
    aviffy.set_full_color_range(true);
    // aviffy.set_exif(metadata_to_exif()?);
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