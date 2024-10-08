use std::fs::copy;
use std::fs::remove_file;
use std::path::Path;


extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main() 
-> Result<(), std::io::Error>
{
    remove_file("./rsrc/copy.jpg")?;
    copy("./rsrc/IMG_20240828_184255.jpg", "./rsrc/copy.jpg")?;
    let jpg_path = Path::new("./rsrc/copy.jpg");

    let read_metadata_1 = Metadata::new_from_path(&jpg_path).unwrap();
    for tag in read_metadata_1.data()
    {
        println!("{:?}", tag);
    }

    let mut metadata = Metadata::new();
    // let mut metadata = Metadata::new_from_path(&jpg_path).unwrap();

    metadata.set_tag(ExifTag::ImageDescription("Hello World!".to_string()));
    let res = metadata.write_to_file(std::path::Path::new(&jpg_path));
    println!("\nres: {:?}\n", res);

    let read_metadata_2 = Metadata::new_from_path(&jpg_path).unwrap();
    for tag in read_metadata_2.data()
    {
        let a_clone = tag.clone();
        println!("{:?}", a_clone);
    }

    Ok(())
}
