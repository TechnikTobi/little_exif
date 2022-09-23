use std::path::Path;

pub const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

enum
PngChunkOrdering
{
	FIRST,
	BEFORE_IDAT,
	BEFORE_PLTE_AND_IDAT,
	AFTER_PLTE_BEFORE_IDAT,
	LAST,
	NONE
}

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
		}
	}
}

/*
struct 
PNGdata
{
	
}
*/

pub fn
write_metadata
(
	path: &Path,
	encoded_metadata: &Vec<u8>
)
-> Result<(), String>
{
	Ok(())
}



build_png_chunk_type_enum![
	// Tag	Critical	Multiple	Ordering
	(IHDR,	true,		false,		FIRST),
	(PLTE,	true,		false,		BEFORE_IDAT),
	(IDAT,	true,		true,		NONE),
	(IEND,	true,		false,		LAST),
	(zTXt,	false,		true,		NONE)
];
