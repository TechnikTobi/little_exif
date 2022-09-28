pub enum
PngChunkOrdering
{
	FIRST,
	BEFORE_IDAT,
	BEFORE_PLTE_AND_IDAT,
	AFTER_PLTE_BEFORE_IDAT,
	LAST,
	NONE
}

/// This macro builds the enum for the different type of PNG chunks
macro_rules! build_png_chunk_type_enum {
	(
		$( (
			$tag:ident,
			$critical:expr,
			$multiple:expr,
			$ordering:ident
		) ),*
	)
	=>
	{
		pub enum
		PngChunk
		{
			$(
				$tag(u32),
			)*
		}

		impl PngChunk
		{
			pub fn
			is_critical
			(
				&self
			)
			-> bool
			{
				match *self
				{
					$(
						PngChunk::$tag(_) => $critical,
					)*
				}
			}

			pub fn
			allows_multiple
			(
				&self
			)
			-> bool
			{
				match *self
				{
					$(
						PngChunk::$tag(_) => $multiple,
					)*
				}
			}

			pub fn
			ordering
			(
				&self
			)
			-> PngChunkOrdering
			{
				match *self
				{
					$(
						PngChunk::$tag(_) => PngChunkOrdering::$ordering,
					)*
				}
			}

			pub fn
			length
			(
				&self
			)
			-> u32
			{
				match *self
				{
					$(
						PngChunk::$tag(length) => length,
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
						PngChunk::$tag(_) => String::from(stringify!($tag)),
					)*
				}
			}

			pub fn
			from_string
			(
				string_name: &String,
				length: u32
			)
			-> Result<PngChunk, String>
			{
				match &(*string_name.as_str())
				{
					$(
						stringify!($tag) => Ok(PngChunk::$tag(length)),
					)*
					_ => Err("Invalid chunk name".to_string()),
				}
			}
		}
	}
}

build_png_chunk_type_enum![
	// Tag	Critical	Multiple	Ordering
	(IHDR,	true,		false,		FIRST),
	(PLTE,	true,		false,		BEFORE_IDAT),
	(IDAT,	true,		true,		NONE),
	(IEND,	true,		false,		LAST),
	(zTXt,	false,		true,		NONE)
];
