
macro_rules! build_tag_enum {
	( $( ($tag:ident, $hex_value:expr) ),* ) => {
		pub enum 
		ExifTag
		{
			$(
				$tag,
			)*
		}

		impl ExifTag
		{
			pub fn
			as_u16
			(
				&self
			)
			-> u16
			{
				match *self
				{
					$(
						ExifTag::$tag => $hex_value,
					)*
				}
			}

			pub fn
			as_string
			(
				&self
			)
			-> String
			{
				match *self
				{
					$(
						ExifTag::$tag => String::from(stringify!($tag)),
					)*
				}
			}

		}

	};
}

build_tag_enum![
	(Hallo, 1),
	(Welt, 0xa),
	(Tobi, 0xff)
];
