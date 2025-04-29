// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

/// This gets the keyword of a $TEXT chunk.
/// Fortunately, this is the same for tEXt, zTXt and iTXt, as they all
/// start with a keyword that is followed by a NUL separator
pub(crate) fn
extract_keyword_from_text_chunk_data
(
	chunk_data: &[u8]
)
-> String
{
	let mut keyword_buffer = Vec::new();
	for character in chunk_data
	{
		if *character == 0x00 { break; }
		keyword_buffer.push(*character);
	}
	return String::from_utf8(keyword_buffer).unwrap();
}

