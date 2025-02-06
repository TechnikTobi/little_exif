

#[allow(non_snake_case)]
pub(crate) fn
write_metadata
(
	path:     &Path,
	metadata: &Metadata
)
-> Result<(), std::io::Error>
{

	// First clear the existing metadata
	// This also parses the PNG and checks its validity, so it is safe to
	// assume that is, in fact, a usable PNG file
	let _ = clear_metadata(path)?;

	let mut IHDR_length = 0u32;
	if let Ok(chunks) = parse_png(path)
	{
		IHDR_length = chunks[0].length();
	}

	// Encode the data specifically for PNG and open the image file
	let encoded_metadata = encode_metadata_png(&metadata.encode()?);
	let seek_start = 0u64         // Skip ...
	+ PNG_SIGNATURE.len() as u64  // PNG Signature
	+ IHDR_length         as u64  // IHDR data section
	+ 12                  as u64; // rest of IHDR chunk (length, type, CRC)

	// Get to first chunk after IHDR, copy all the data starting from there
	let mut file   = open_write_file(path)?;
	let mut buffer = Vec::new();
	perform_file_action!(file.seek(SeekFrom::Start(seek_start)));
	perform_file_action!(file.read_to_end(&mut buffer));
	perform_file_action!(file.seek(SeekFrom::Start(seek_start)));

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

	// Write new data to PNG file
	// Start with length of the new chunk (subtracting 8 for type and CRC)
	let chunk_data_len = zTXt_chunk_data.len() as u32 - 8;
	for i in 0..4
	{
		perform_file_action!(file.write( &[(chunk_data_len >> (8 * (3-i))) as u8] ));
	}

	// Write data of new chunk and rest of PNG file
	perform_file_action!(file.write_all(&zTXt_chunk_data));
	perform_file_action!(file.write_all(&buffer));

	return Ok(());
}
