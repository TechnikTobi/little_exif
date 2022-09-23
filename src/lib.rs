mod endian;
mod exif_tag;
mod exif_tag_format;
mod metadata;

mod png;

#[cfg(test)]
mod tests {

    use crate::exif_tag::ExifTag;
	use crate::exif_tag_value::ExifTagValue;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);

	let some: ExifTag = ExifTag::ImageDescription;
	assert_eq!(some.as_u16(), 0x010e);
	assert_eq!(some.as_string(), String::from("ImageDescription"));
	assert_eq!(ExifTag::from_u16(0x010e).unwrap(), ExifTag::ImageDescription);

	let some_value = ExifTagValue::STRING("Hello :)".to_string());
	let other_value = ExifTagValue::INT8U(0);

	assert_eq!(some.accepts(&some_value), true);
	assert_eq!(some.accepts(&other_value), false);

	println!("hihi {}", some.as_u16());
    }
}
