// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::path::Path;

use log::debug;

use crate::endian::*;
use crate::metadata::Metadata;
use crate::u8conversion::*;
use crate::general_file_io::*;
use super::riff_chunk::RiffChunk;
use super::riff_chunk::RiffChunkDescriptor;
use super::*;

/// A WebP file starts as follows
/// - The RIFF signature: ASCII characters "R", "I", "F", "F"  -> 4 bytes
/// - The file size starting at offset 8                       -> 4 bytes
/// - The WEBP signature: ASCII characters "W", "E", "B", "P"  -> 4 bytes
/// 
/// This function checks these 3 sections and their correctness after making
/// sure that the file actually exists and can be opened. 
/// Finally, the file struct is returned for further processing
fn
check_signature
(
    path: &Path
)
-> Result<File, std::io::Error>
{
    let mut file = open_write_file(path)?;

    // Get the first 12 bytes that are required for the following checks
    let mut first_12_bytes = [0u8; 12];
    file.read(&mut first_12_bytes)?;
    let first_12_bytes_vec = first_12_bytes.to_vec();
    
    // Perform checks
    check_riff_signature(&first_12_bytes_vec             )?;
    check_byte_count(    &first_12_bytes_vec, Some(&file))?;
    check_webp_signature(&first_12_bytes_vec             )?;

    // Signature is valid - can proceed using the file as WebP file
    return Ok(file);
}



/// Gets the next RIFF chunk, starting at the current file cursor
/// Advances the cursor to the start of the next chunk
fn
get_next_chunk
<T: Seek + Read>
(
    file: &mut T
)
-> Result<RiffChunk, std::io::Error>
{
    // Read the start of the chunk
    let mut chunk_start = [0u8; 8];
    let mut bytes_read = file.read(&mut chunk_start).unwrap();

    // Check that indeed 8 bytes were read
    if bytes_read != 8
    {
        return io_error!(UnexpectedEof, "Could not read start of chunk");
    }

    // Construct name of chunk and its length
    let chunk_name = String::from_utf8(chunk_start[0..4].to_vec());
    let mut chunk_length = from_u8_vec_macro!(u32, &chunk_start[4..8], &Endian::Little);

    // Account for the possible padding byte
    chunk_length += chunk_length % 2;

    // Read RIFF chunk data
    let mut chunk_data_buffer = vec![0u8; chunk_length as usize];
    bytes_read = file.read(&mut chunk_data_buffer).unwrap();
    if bytes_read != chunk_length as usize
    {
        return io_error!(
            Other, 
            format!("Could not read RIFF chunk data! Expected {chunk_length} bytes but read {bytes_read}")
        );
    }

    if let Ok(parsed_chunk_name) = chunk_name
    {
        return Ok(RiffChunk::new(
            parsed_chunk_name as String, 
            chunk_length      as usize,
            chunk_data_buffer as Vec<u8>
        ));
    }
    else
    {
        return io_error!(Other, "Could not parse RIFF fourCC chunk name!");
    }
}



/// Gets a descriptor of the next RIFF chunk, starting at the current file
/// cursor position. Advances the cursor to the start of the next chunk
/// Relies on `get_next_chunk` by basically calling that function and throwing
/// away the actual payload
fn
get_next_chunk_descriptor
<T: Seek + Read>
(
    file: &mut T
)
-> Result<RiffChunkDescriptor, std::io::Error>
{
    let next_chunk_result = get_next_chunk(file)?;
    return Ok(next_chunk_result.descriptor());
}



/// "Parses" the WebP file by checking various properties:
/// - Can the file be opened and is the signature valid, including the file size?
/// - Are the chunks and their size descriptions OK? Relies on the local subroutine `get_next_chunk_descriptor`
pub(crate) fn
parse_webp
(
    path: &Path
)
-> Result<Vec<RiffChunkDescriptor>, std::io::Error>
{
    let file_result = check_signature(path);
    let mut chunks = Vec::new();

    if file_result.is_err()
    {
        return Err(file_result.err().unwrap());
    }

    let mut file = file_result.unwrap();

    // The amount of data we expect to read while parsing the chunks
    let expected_length = file.metadata().unwrap().len();

    // How much data we have parsed so far.
    // Starts with 12 bytes: 
    // - 4 bytes for RIFF signature
    // - 4 bytes for file size
    // - 4 bytes for WEBP signature
    // These bytes are already read in by the `check_signature` subroutine
    let mut parsed_length = 12u64;

    loop
    {
        let next_chunk_descriptor_result = get_next_chunk_descriptor(&mut file);
        if let Ok(chunk_descriptor) = next_chunk_descriptor_result
        {
            // The parsed length increases by the length of the chunk's 
            // header (4 byte) + it's size section (4 byte) and the payload
            // size, which is noted by the aforementioned size section
            parsed_length += 4u64 + 4u64 + chunk_descriptor.len() as u64;

            // Add the chunk descriptor
            chunks.push(chunk_descriptor);
            
            if parsed_length == expected_length
            {
                // In this case we don't expect any more data to be in the file
                break;
            }
        }
        else
        {
            // This is the case when the read of the next chunk descriptor 
            // fails due to not being able to fetch 8 bytes for the header and
            // chunk size information, indicating that there is no further data
            // in the file and we are done with parsing.
            // If the subroutine fails due to other reasons, the error gets
            // propagated further.
            if next_chunk_descriptor_result.as_ref().err().unwrap().kind() == std::io::ErrorKind::UnexpectedEof
            {
                break;
            }
            else
            {
                return Err(next_chunk_descriptor_result.err().unwrap());
            }
        }
    }

    return Ok(chunks);
}



fn
check_exif_in_file
(
    path: &Path
)
-> Result<(File, Vec<RiffChunkDescriptor>), std::io::Error>
{
    // Parse the WebP file - if this fails, we surely can't read any metadata
    let parsed_webp_result = parse_webp(path)?;

    // Next, check if this is an Extended File Format WebP file
    // In this case, the first Chunk SHOULD have the type "VP8X"
    // Otherwise, the file is either invalid ("VP8X" at wrong location) or a 
    // Simple File Format WebP file which don't contain any EXIF metadata.
    if let Some(first_chunk) = parsed_webp_result.first()
    {
        // Compare the chunk descriptor header.
        if first_chunk.header().to_lowercase() != VP8X_HEADER.to_lowercase()
        {
            return io_error!(
                Other, 
                format!("Expected first chunk of WebP file to be of type 'VP8X' but instead got {}!", first_chunk.header())
            );
        }
    }
    else
    {
        return io_error!(Other, "Could not read first chunk descriptor of WebP file!");
    }

    // Finally, check the flag by opening up the file and reading the data of
    // the VP8X chunk
    // Regarding the seek:
    // - RIFF + file size + WEBP -> 12 byte
    // - VP8X header             ->  4 byte
    // - VP8X chunk size         ->  4 byte
    let mut file = check_signature(path).unwrap();
    let mut flag_buffer = vec![0u8; 4usize];
    file.seek(SeekFrom::Start(12u64 + 4u64 + 4u64))?;
    if file.read(&mut flag_buffer).unwrap() != 4
    {
        return io_error!(Other, "Could not read flags of VP8X chunk!");
    }

    // Check the 5th bit of the 32 bit flag_buffer. 
    // For further details see the Extended File Format section at
    // https://developers.google.com/speed/webp/docs/riff_container#extended_file_format
    if flag_buffer[0] & 0x08 != 0x08
    {
        return io_error!(Other, "No EXIF chunk according to VP8X flags!");
    }

    return Ok((file, parsed_webp_result));
}



/// Reads the raw EXIF data from the WebP file. Note that if the file contains
/// multiple such chunks, the first one is returned and the others get ignored.
pub(crate) fn
read_metadata
(
    path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
    // Check the file signature, parse it, check that it has a VP8X chunk and
    // the EXIF flag is set there
    let (mut file, parse_webp_result) = check_exif_in_file(path).unwrap();

    // At this point we have established that the file has to contain an EXIF
    // chunk at some point. So, now we need to find & return it
    // Start by seeking to the start of the first chunk and visiting chunk after
    // chunk via checking the type and seeking again to the next chunk via the
    // size information
    file.seek(SeekFrom::Start(12u64))?;
    let mut header_buffer = vec![0u8; 4usize];
    let mut chunk_index = 0usize;
    loop
    {
        // Read the chunk type into the buffer
        if file.read(&mut header_buffer).unwrap() != 4
        {
            return io_error!(Other, "Could not read chunk type while traversing WebP file!");
        }
        let chunk_type = String::from_u8_vec(&header_buffer.to_vec(), &Endian::Little);

        // Check that this is still the type that we expect from the previous
        // parsing over the file
        // TODO: Maybe remove this part?
        let expected_chunk_type = parse_webp_result.get(chunk_index).unwrap().header();
        if chunk_type != expected_chunk_type
        {
            return io_error!(
                Other, 
                format!("Got unexpected chunk type! Expected {} but got {}", 
                    expected_chunk_type, 
                    chunk_type
                )
            );
        }

        // Get the size of this chunk from the previous parsing process and skip
        // the 4 bytes regarding the size
        let chunk_size = parse_webp_result.get(chunk_index).unwrap().len();
        file.seek(std::io::SeekFrom::Current(4))?;

        if chunk_type.to_lowercase() == EXIF_CHUNK_HEADER.to_lowercase()
        {
            // Read the EXIF chunk's data into a buffer
            let mut payload_buffer = vec![0u8; chunk_size];
            file.read(&mut payload_buffer)?;

            // Add the 6 bytes of the EXIF_HEADER as Prefix for the generic EXIF
            // data parser that is called on the result of this read function
            // Otherwise the result would directly start with the Endianness
            // information, leading to a failed EXIF header signature check in 
            // the function `decode_metadata_general`
            let mut raw_exif_data = EXIF_HEADER.to_vec();
            raw_exif_data.append(&mut payload_buffer);

            return Ok(raw_exif_data);
        }
        else
        {
            // Skip the entire chunk
            file.seek(std::io::SeekFrom::Current(chunk_size as i64))?;

            // Note that we have to seek another byte in case the chunk is of 
            // uneven size to account for the padding byte that must be included
            file.seek(std::io::SeekFrom::Current(chunk_size as i64 % 2))?;
        }

        // Update for next loop iteration
        chunk_index += 1;
    }
}



fn
update_file_size_information
(
    file:  &mut File,
    delta: i32
)
-> Result<(), std::io::Error>
{
    // Note from the documentation:
    // As the size of any chunk is even, the size given by the RIFF header is also even.

    // Update the file size information, first by reading in the current value...
    file.seek(SeekFrom::Start(4))?;
    let mut file_size_buffer = [0u8; 4];

    // ...converting it to u32 representation...
    file.read(&mut file_size_buffer)?;
    let old_file_size = from_u8_vec_macro!(u32, &file_size_buffer, &Endian::Little);

    // ...adding the delta byte count (and performing some checks)...
    if delta < 0
    {
        assert!(old_file_size as i32 > delta);
    }
    let new_file_size = (old_file_size as i32 + delta) as u32;

    assert!(old_file_size % 2 == 0);
    assert!(new_file_size % 2 == 0);

    // ...and writing back to file...
    file.seek(SeekFrom::Start(4))?;
    file.write_all(&to_u8_vec_macro!(u32, &new_file_size, &Endian::Little))?;

    Ok(())
}



fn
convert_to_extended_format
(
    file: &mut File
)
-> Result<(), std::io::Error>
{
    // Start by getting the first chunk of the WebP file
    file.seek(SeekFrom::Start(12))?;
    let first_chunk_result = get_next_chunk(file);

    // Check that this get operation was successful
    if first_chunk_result.is_err()
    {
        return Err(first_chunk_result.err().unwrap());
    }

    let first_chunk = first_chunk_result.unwrap();

    // Find out what simple type of WebP file we are dealing with
    let (width, height) = match first_chunk.descriptor().header().as_str()
    {
        "VP8" 
            => {debug!("VP8 !"); todo!()},
        "VP8L"
            => get_dimension_info_from_vp8l_chunk(first_chunk.payload()),
        _ 
            => io_error!(Other, "Expected either 'VP8 ' or 'VP8L' chunk for conversion!")
    }?;

    let width_vec  = to_u8_vec_macro!(u32, &width,  &Endian::Little);
    let height_vec = to_u8_vec_macro!(u32, &height, &Endian::Little);

    let mut vp8x_chunk = vec![
        0x56, 0x50, 0x38, 0x58, // ASCII chars "V", "P", "8", "X"                  -> 4 byte
        0x0A, 0x00, 0x00, 0x00, // size of this chunk (32 + 24 + 24 bit = 10 byte) -> 4 byte
        0x00, 0x00, 0x00, 0x00, // Flags and reserved area                         -> 4 byte
    ];

    // Add the two 24 bits for width and height information
    for byte in  width_vec.iter().take(3) { vp8x_chunk.push(*byte); }
    for byte in height_vec.iter().take(3) { vp8x_chunk.push(*byte); }

    // Write the VP8X chunk, first by reading the file (except for the header)
    // into a buffer...
    let mut buffer = Vec::new();
    file.seek(SeekFrom::Start(12u64))?;
    file.read_to_end(&mut buffer)?;

    // ...actually writing the VP8X chunk data...
    file.seek(SeekFrom::Start(12u64))?;
    file.write(&vp8x_chunk)?;

    // ...and writing back the file contents
    file.write(&buffer)?;

    // Finally, update the file size information
    update_file_size_information(file, 18)?;

    Ok(())
}



fn
get_dimension_info_from_vp8l_chunk
(
    payload: &[u8],
)
-> Result<(u32, u32), std::io::Error>
{
    // Get the 4 bytes containing the dimension information
    // (although we only need 28 bits)
    // Starting at byte 1 instead of 0 due to the 0x2F byte
    // See: https://developers.google.com/speed/webp/docs/webp_lossless_bitstream_specification#3_riff_header
    let width_height_info_buffer = payload[1..5].to_vec();
    
    // Convert to a single u32 number for bit-mask operations
    let width_height_info = from_u8_vec_macro!(u32, &width_height_info_buffer, &Endian::Little);
    
    let mut width  = 0;
    let mut height = 0;

    // Get the first 14 bit to construct the width
    for bit_index in 0..14
    {
        width  |= ((width_height_info >> (27 - bit_index)) & 0x01) << (13 - (bit_index % 14));
    }

    // Get the next 14 bit to construct the height
    for bit_index in 14..28
    {
        height |= ((width_height_info >> (27 - bit_index)) & 0x01) << (13 - (bit_index % 14));
    }

    return Ok((width, height));
}



fn
set_exif_flag
(
    path:  &Path,
    exif_flag_value: bool
)
-> Result<(), std::io::Error>
{
    // Parse the WebP file - if this fails, we surely can't read any metadata
    let parsed_webp_result = parse_webp(path)?;

    // Open the file for further processing
    let mut file = check_signature(path).unwrap();

    // Next, check if this is an Extended File Format WebP file
    // In this case, the first Chunk SHOULD have the type "VP8X"
    // Otherwise we have to create the VP8X chunk!
    if let Some(first_chunk) = parsed_webp_result.first()
    {
        // Compare the chunk descriptor header and call chunk creator if required
        if first_chunk.header().to_lowercase() != VP8X_HEADER.to_lowercase()
        {
            convert_to_extended_format(&mut file)?;
        }
    }
    else
    {
        return io_error!(Other, "Could not read first chunk descriptor of WebP file!");
    }	

    // At this point we know that we have a VP8X chunk at the expected location
    // So, read in the flags and set the EXIF flag according to the given bool
    let mut flag_buffer = vec![0u8; 4usize];
    file.seek(SeekFrom::Start(12u64 + 4u64 + 4u64))?;
    if file.read(&mut flag_buffer).unwrap() != 4
    {
        return io_error!(Other, "Could not read flags of VP8X chunk!");
    }

    // Mask the old flag by either or-ing with 1 at the EXIF flag position for
    // setting it to true, or and-ing with 1 everywhere but the EXIF flag pos
    // to set it to false
    flag_buffer[0] = if exif_flag_value
    {
        flag_buffer[0] | 0x08
    }
    else
    {
        flag_buffer[0] & 0b11110111
    };

    // Write flag buffer back to the file
    file.seek(SeekFrom::Start(12u64 + 4u64 + 4u64))?;
    file.write_all(&flag_buffer)?;

    Ok(())
}



pub(crate) fn
clear_metadata
(
    path: &Path
)
-> Result<(), std::io::Error>
{
    // Check the file signature, parse it, check that it has a VP8X chunk and
    // the EXIF flag is set there
    let exif_check_result = check_exif_in_file(path);
    if exif_check_result.is_err()
    {
        match exif_check_result.as_ref().err().unwrap().to_string().as_str()
        {
            "No EXIF chunk according to VP8X flags!"
                => return Ok(()),
            "Expected first chunk of WebP file to be of type 'VP8X' but instead got VP8L!"
                => return Ok(()),
            _
                => return Err(exif_check_result.err().unwrap())
        }
    }

    let (mut file, parse_webp_result) = exif_check_result.unwrap();

    // Compute a delta of how much the file size information has to change
    let mut delta = 0i32;

    // Skip the WEBP signature
    file.seek(std::io::SeekFrom::Current(4i64))?;

    for parsed_chunk in parse_webp_result
    {
        // At the start of each iteration, the file cursor is at the start of
        // the fourCC section of a chunk

        // Compute how many bytes this chunk has
        let parsed_chunk_byte_count = 
            4u64                            // fourCC section of EXIF chunk
            + 4u64                          // size information of EXIF chunk
            + parsed_chunk.len() as u64     // actual size of EXIF chunk data
            + parsed_chunk.len() as u64 % 2 // accounting for possible padding byte
        ;

        // Not an EXIF chunk, seek to next one and continue
        if parsed_chunk.header().to_lowercase() != EXIF_CHUNK_HEADER.to_lowercase()
        {
            file.seek(std::io::SeekFrom::Current(parsed_chunk_byte_count as i64))?;
            continue;
        }

        // Get the current size of the file in bytes
        let old_file_byte_count = file.metadata().unwrap().len();

        // Get a backup of the current cursor position
        let exif_chunk_start_cursor_position = SeekFrom::Start(file.seek(SeekFrom::Current(0)).unwrap());

        // Skip the EXIF chunk ...
        file.seek(std::io::SeekFrom::Current(parsed_chunk_byte_count as i64))?;

        // ...and copy everything afterwards into a buffer...
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // ...and seek back to where the EXIF chunk is located...
        file.seek(exif_chunk_start_cursor_position)?;

        // ...and overwrite the EXIF chunk...
        file.write_all(&buffer)?;

        // ...and finally update the size of the file
        file.set_len(old_file_byte_count - parsed_chunk_byte_count)?;

        // Additionally, update the size information that gets written to the 
        // file header after this loop
        delta -= parsed_chunk_byte_count as i32;
    }

    // Update file size information
    update_file_size_information(&mut file, delta)?;
    
    // Set the flags in the VP8X chunk. First, read in the current flags
    set_exif_flag(path, false)?;

    return Ok(());
}



/// Writes the given generally encoded metadata to the WebP image file at 
/// the specified path. 
/// Note that *all* previously stored EXIF metadata gets removed first before
/// writing the "new" metadata. 
pub(crate) fn
write_metadata
(
    path:     &Path,
    metadata: &Metadata
)
-> Result<(), std::io::Error>
{
    // Clear the metadata from the file and return if this results in an error
    clear_metadata(path)?;

    // Encode the general metadata format to WebP specifications
    let encoded_metadata = encode_metadata_webp(&metadata.encode()?);

    // Open the file...
    let mut file = check_signature(path)?;

    // ...and find a location where to put the EXIF chunk
    // This is done by requesting a chunk descriptor as long as we find a chunk
    // that is both known and should be located *before* the EXIF chunk
    let pre_exif_chunks = [
        "VP8X",
        "VP8",
        "VP8L",
        "ICCP",
        "ANIM"
    ];

    loop
    {
        // Request a chunk descriptor. If this fails, check the error 
        // Depending on its type, either continue normally or return it
        let chunk_descriptor_result = get_next_chunk_descriptor(&mut file);

        if let Ok(chunk_descriptor) = chunk_descriptor_result
        {
            let mut chunk_type_found_in_pre_exif_chunks = false;

            // Check header of chunk descriptor against any of the known chunks
            // that should come before the EXIF chunk
            for pre_exif_chunk in &pre_exif_chunks
            {
                chunk_type_found_in_pre_exif_chunks |= pre_exif_chunk.to_lowercase() == chunk_descriptor.header().to_lowercase();
            }

            if !chunk_type_found_in_pre_exif_chunks
            {
                break;
            }
        }
        else
        {
            match chunk_descriptor_result.as_ref().err().unwrap().kind()
            {
                std::io::ErrorKind::UnexpectedEof
                    => break, // No further chunks, place EXIF chunk here
                _
                    => return Err(chunk_descriptor_result.err().unwrap())
            }
        }
    }

    // Next, read remaining file into a buffer...
    let current_file_cursor = SeekFrom::Start(file.seek(SeekFrom::Current(0)).unwrap());
    let mut read_buffer = Vec::new();
    file.read_to_end(&mut read_buffer)?;

    // ...and write the EXIF chunk at the previously found location...
    file.seek(current_file_cursor)?;
    file.write_all(&encoded_metadata)?;

    // ...and writing back the remaining file content
    file.write_all(&read_buffer)?;

    // Update the file size information by adding the byte count of the EXIF chunk
    // (Note: Due to  the WebP specific encoding function, this vector already
    // contains the EXIF header characters and size information, as well as the
    // possible padding byte. Therefore, simply taking the length of this
    // vector takes their byte count also into account and no further values
    // need to be added)
    update_file_size_information(&mut file, encoded_metadata.len() as i32)?;

    // Finally, set the EXIF flag
    set_exif_flag(path, true)?;

    return Ok(());
}





#[cfg(test)]
mod tests 
{
    use std::fs::copy;
    use std::fs::remove_file;
    use std::path::Path;

    #[test]
    fn
    clear_metadata()
    -> Result<(), std::io::Error>
    {
        // Remove file from previous run and replace it with fresh copy
        if let Err(error) = remove_file("tests/read_sample_no_exif.webp")
        {
            println!("{}", error);
        }
        copy("tests/read_sample.webp", "tests/read_sample_no_exif.webp")?;

        // Clear the metadata
        crate::webp::file::clear_metadata(Path::new("tests/read_sample_no_exif.webp"))?;

        Ok(())
    }
}
