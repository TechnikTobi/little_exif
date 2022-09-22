mod exif_tag;

#[cfg(test)]
mod tests {

    use crate::exif_tag::ExifTag;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);

	let some: ExifTag = ExifTag::ImageDescription;
	assert_eq!(some.as_u16(), 0x010e);
	assert_eq!(some.as_string(), String::from("ImageDescription"));
	assert_eq!(ExifTag::from_u16(0x010e).unwrap(), ExifTag::ImageDescription);
	println!("hihi {}", some.as_u16());
    }
}
