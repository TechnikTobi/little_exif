
#[allow(non_snake_case)]
pub(crate) fn
write_metadata
(
	file_buffer: &mut Vec<u8>,
	metadata:    &Metadata
)
-> Result<(), std::io::Error>
{
	// First clear the existing metadata
	// This also parses the PNG and checks its validity, so it is safe to
	// assume that is, in fact, a usable PNG file
	let _ = clear_metadata(file_buffer)?;

	let mut IHDR_length = 0u32;
	if let Ok(chunks) = parse_png(file_buffer)
	{
		IHDR_length = chunks[0].length();
	}

	// Encode the data specifically for PNG and open the image file
	let encoded_metadata = encode_metadata_png(&metadata.encode()?);
	let seek_start = 0u64         // Skip ...
	+ PNG_SIGNATURE.len() as u64  // PNG Signature
	+ IHDR_length         as u64  // IHDR data section
	+ 12                  as u64; // rest of IHDR chunk (length, type, CRC)

	// Build data of new chunk using zlib compression (level=8 -> default)
	let mut zTXt_chunk_data: Vec<u8> = vec![0x7a, 0x54, 0x58, 0x74];
	zTXt_chunk_data.extend(RAW_PROFILE_TYPE_EXIF.iter());
	zTXt_chunk_data.extend(compress_to_vec_zlib(&encoded_metadata, 8).iter());

	// Compute CRC and append it to the chunk data
	let crc_struct = Crc::<u32>::new(&CRC_32_ISO_HDLC);
	let checksum = crc_struct.checksum(&zTXt_chunk_data) as u32;
	for i in 0..4
	{
		zTXt_chunk_data.push( (checksum >> (8 * (3-i))) as u8);		
	}

	// Prepare the length of the new chunk (subtracting 8 for type and CRC) for
	// inserting prior to the new chunk
	let     chunk_data_len        = zTXt_chunk_data.len() as u32 - 8;
	let mut chunk_data_len_buffer = [0u8; 4];
	for i in 0..4
	{
		chunk_data_len_buffer[i] = (chunk_data_len >> (8 * (3-i))) as u8;
	}
	
	// Write data of new chunk length and chunk itself
	let insert_position = seek_start as usize;
	insert_multiple_at(file_buffer, insert_position,   &mut chunk_data_len_buffer.to_vec());
	insert_multiple_at(file_buffer, insert_position+4, &mut zTXt_chunk_data);

	return Ok(());
}