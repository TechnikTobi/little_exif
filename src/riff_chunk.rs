// Copyright © 2023 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

#[allow(non_snake_case)]
pub(crate) struct
RiffChunkDescriptor
{
	fourCC:  String, // The 4 byte long header at the start of the chunk
	size:    usize,  // Chunk size WITHOUT the 8 bytes for the header and size section
}

impl
RiffChunkDescriptor
{
	#[allow(non_snake_case)]
	pub fn
	new
	(
		fourCC: String,
		size:   usize
	)
	-> RiffChunkDescriptor
	{
		RiffChunkDescriptor
		{
			fourCC: fourCC,
			size:   size
		}
	}

	pub fn
	len
	(
		&self
	)
	-> usize
	{
		self.size
	}

	pub fn
	header
	(
		&self
	)
	-> String
	{
		self.fourCC.clone()
	}
}

/*
pub(crate) struct
RiffChunk
{
	descriptor: RiffChunkDescriptor,
	payload:    Vec<u8>
}

impl
RiffChunk
{
	pub fn
	new
	(
		fourCC:  String,
		size:    usize,
		payload: Vec<u8>
	)
	-> RiffChunk
	{
		RiffChunk
		{
			descriptor: RiffChunkDescriptor::new(fourCC, size),
			payload:    payload
		}
	}
}
*/