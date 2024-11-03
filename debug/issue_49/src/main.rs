use std::path::Path;

extern crate little_exif;
use little_exif::metadata::Metadata;
use little_exif::exif_tag::ExifTag;

fn main() 
{
    let jpg_path = Path::new("./rsrc/image.jpg");
    let metadata = Metadata::new_from_path(jpg_path).unwrap();

    let mut tag_iterator = metadata.get_tag(&ExifTag::GPSLatitude(Vec::new()));

    match tag_iterator.next() {
        Some(tag) => println!("Tag: {:?}", tag),
        None      => panic!("Tag does not exist"),
    };
}
