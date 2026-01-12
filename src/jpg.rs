// Copyright © 2022-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use crate::endian::Endian;
use crate::metadata::Metadata;
use crate::u8conversion::*;
use crate::general_file_io::*;

pub(crate) const JPG_SIGNATURE: [u8; 2] = [0xff, 0xd8];

const JPG_MARKER_PREFIX: u8  = 0xff;
const JPG_APP1_MARKER:   u16 = 0xffe1;



fn
encode_metadata_jpg
(
    exif_vec: &[u8],
)
-> Vec<u8>
{
    // vector storing the data that will be returned
    let mut jpg_exif: Vec<u8> = Vec::new();

    // Compute the length of the exif data (includes the two bytes of the
    // actual length field)
    let length = 2u16 + (EXIF_HEADER.len() as u16) + (exif_vec.len() as u16);

    // Start with the APP1 marker and the length of the data
    // Then copy the previously encoded EXIF data 
    jpg_exif.extend(to_u8_vec_macro!(u16, &JPG_APP1_MARKER, &Endian::Big));
    jpg_exif.extend(to_u8_vec_macro!(u16, &length, &Endian::Big));
    jpg_exif.extend(EXIF_HEADER.iter());
    jpg_exif.extend(exif_vec.iter());

    return jpg_exif;
}



fn
check_signature
(
    file_buffer: &[u8],
)
-> Result<(), std::io::Error>
{
    if !file_buffer.starts_with(&JPG_SIGNATURE)
    {
        return io_error!(InvalidData, "Can't open JPG file - Wrong signature!");
    }

    // Signature is valid - can proceed using as JPG file
    return Ok(());
}

fn
file_check_signature
(
    path: &Path
)
-> Result<File, std::io::Error>
{
    let mut file = open_read_file(path)?;
    
    // Read & check the signature
    let mut signature_buffer = [0u8; 2];
    let bytes_read = file.read(&mut signature_buffer)?;

    if bytes_read != 2
    {
        return io_error!(InvalidData, "Can't open JPG file - Can't read signature!");
    }

    check_signature(&signature_buffer)?;

    // Signature is valid - can proceed using the file as JPG file
    return Ok(file);
}


pub(crate) fn
clear_metadata
(
    file_buffer: &mut Vec<u8>,
)
-> Result<(), std::io::Error>
{
    return clear_segment(file_buffer, 0xe1);
}


pub(crate) fn
clear_segment
(
    file_buffer:    &mut Vec<u8>,
    segment_marker: u8,
)
-> Result<(), std::io::Error>
{
    check_signature(file_buffer)?;

    // Setup of variables necessary for going through the file
    let mut byte_buffer = [0u8; 1];                                             // A buffer for reading in a byte of data from the file
    let mut previous_byte_was_marker_prefix = false;                            // A boolean for remembering if the previous byte was a marker prefix (0xFF)
    let mut cursor = Cursor::new(file_buffer);

    // Skip 0xFFD8 at the start
    cursor.seek(SeekFrom::Current(2))?;

    loop
    {
        // Read next byte into buffer
        if let Err(e) = cursor.read_exact(&mut byte_buffer)
        {
            if e.kind() == std::io::ErrorKind::UnexpectedEof
            {
                // Reached end of file without encountering EOI marker 0xd9
                // See issue #93 for examples where this happens
                return Ok(());
            }
            else
            {
                return Err(e);
            }
        }

        if previous_byte_was_marker_prefix
        {
            // Check if this is the end of the file. In that case, the length
            // data can't be read and we need to return prematurely. 
            if byte_buffer[0] == 0xd9                                           // EOI marker
            {
                // No more data to read in
                return Ok(());
            }

            // Read in the length of the segment
            // (which follows immediately after the marker)
            let mut length_buffer = [0u8; 2];
            cursor.read_exact(&mut length_buffer)?;

            // Decode the length to determine how much more data there is
            let length = from_u8_vec_res_macro!(u16, &length_buffer, &Endian::Big)?;
            let remaining_length = (length - 2) as usize;

            if byte_buffer[0] == segment_marker                                 // Given marker, e.g. for APP1
            {
                // Backup current position, account for the 4 bytes already read
                let backup_position = cursor.position() - 4;

                // Skip the segment
                cursor.seek(SeekFrom::Current(remaining_length as i64))?;

                // Copy data from there onwards into a buffer
                let mut temp_buffer = Vec::new();
                cursor.read_to_end(&mut temp_buffer)?;

                // Overwrite segment
                cursor.set_position(backup_position);
                cursor.write_all(&temp_buffer)?;

                // Cut off right-most bytes that are now duplicates due 
                // to the previous shift-to-left operation
                let cutoff_index = backup_position as usize + temp_buffer.len();
                cursor.get_mut().truncate(cutoff_index);

                // Seek to start of next segment
                cursor.set_position(backup_position);
            }
            else if byte_buffer[0] == 0xda
            {
                // See `generic_read_metadata`
                cursor.seek(SeekFrom::Current(remaining_length as i64))?;

                if let Err(e) = skip_ecs(&mut cursor)
                {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof
                    {
                        // Again, same as the check above, we have reached end
                        // of file without encountering EOI marker 0xd9
                        // See issue #93 for examples where this happens
                        return Ok(());
                    }
                    else
                    {
                        return Err(e);
                    }
                }
            }
            else
            {
                // Skip this segment
                cursor.seek(SeekFrom::Current(remaining_length as i64))?;
            }

            previous_byte_was_marker_prefix = false;
        }
        else
        {
            previous_byte_was_marker_prefix = byte_buffer[0] == JPG_MARKER_PREFIX;
        }
    }
}

pub(crate) fn
file_clear_segment
(
    path:           &Path,
    segment_marker: u8,
)
-> Result<(), std::io::Error>
{
    // Load the entire file into memory instead of reading one byte at a time
    // to improve the overall speed
    // Thanks to Xuf3r for this improvement!
    let mut file_buffer: Vec<u8> = std::fs::read(path)?;

    // Clear the metadata in the APP1 segment from the file buffer
    clear_segment(&mut file_buffer, segment_marker)?;
    
    // Write the file
    // Possible to optimize further by returning the purged bytestream itself?
    let mut file = std::fs::OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(&file_buffer)?;

    return Ok(());
}

pub(crate) fn
file_clear_metadata
(
    path: &Path
)
-> Result<(), std::io::Error>
{
    return file_clear_segment(path, 0xe1);
}


/// Provides the JPEG specific encoding result as vector of bytes to be used
/// by the user (e.g. in combination with another library)
pub(crate) fn
as_u8_vec
(
    general_encoded_metadata: &[u8],
)
-> Vec<u8>
{
    encode_metadata_jpg(general_encoded_metadata)
}



pub(crate) fn
write_metadata
(
    file_buffer: &mut Vec<u8>,
    metadata:    &Metadata
)
-> Result<(), std::io::Error>
{
    // Remove old metadata
    clear_metadata(file_buffer)?;

    // Encode the data specifically for JPG
    let mut encoded_metadata = encode_metadata_jpg(&metadata.encode()?);

    // Insert the metadata right after the signature
    crate::util::insert_multiple_at(file_buffer, 2, &mut encoded_metadata);

    return Ok(());
}

/// Writes the given generally encoded metadata to the JP(E)G image file at 
/// the specified path. 
/// Note that any previously stored metadata under the APP1 marker gets removed
/// first before writing the "new" metadata. 
pub(crate) fn
file_write_metadata
(
    path:     &Path,
    metadata: &Metadata
)
-> Result<(), std::io::Error>
{
    // Load the entire file into memory instead of performing multiple read, 
    // seek and write operations
    let mut file = open_write_file(path)?;
    let mut file_buffer: Vec<u8> = Vec::new();
    file.read_to_end(&mut file_buffer)?;

    // Writes the metadata to the file_buffer vec
    // The called function handles the removal of old metadata and the JPG
    // specific encoding, so we pass only the generally encoded metadata here
    write_metadata(&mut file_buffer, metadata)?;

    // Seek back to start & write the file
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&file_buffer)?;

    return Ok(());
}

pub(crate) fn
read_metadata
(
    file_buffer: &Vec<u8>
)
-> Result<Vec<u8>, std::io::Error>
{
    check_signature(file_buffer)?;

    let mut cursor = Cursor::new(file_buffer);

    // Skip signature
    cursor.set_position(2);

    return generic_read_metadata(&mut cursor);
}

pub(crate) fn
file_read_metadata
(
    path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
    // Use a buffered reader to speed up operations, see issue #21
    let mut buffered_file = BufReader::new(file_check_signature(path)?);
    return generic_read_metadata(&mut buffered_file);
}

/// Skips the entropy-coded segment (ECS) that is followed by a start of scan
/// segment (SOS) and positions the cursor at the start of the next segment,
/// i.e. a 0xFF byte that is followed by a marker that is NOT 0xD0-0xD7 or 0x00.
/// Assumes that the given cursor is positioned at the start of the ECS
fn 
skip_ecs
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<(), std::io::Error>
{
    let mut byte_buffer = [0u8; 1];                                             // A buffer for reading in a byte of data from the file
    let mut previous_byte_was_marker_prefix = false;                            // A boolean for remembering if the previous byte was a marker prefix (0xFF)

    loop
    {
        // Read next byte into buffer
        cursor.read_exact(&mut byte_buffer)?;

        if previous_byte_was_marker_prefix
        {
            match byte_buffer[0]
            {
                0xd0 |
                0xd1 |
                0xd2 |
                0xd3 |
                0xd4 |
                0xd5 |
                0xd6 |
                0xd7 |
                0x00 => {
                    // Do nothing
                },

                _ => {
                    // Position back to where the 0xFF byte is located
                    cursor.seek(SeekFrom::Current(-2))?;
                    return Ok(()); 
                },
            }

            previous_byte_was_marker_prefix = false;
        }
        else
        {
            previous_byte_was_marker_prefix = byte_buffer[0] == JPG_MARKER_PREFIX;
        }
    }
}


fn
generic_read_metadata
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<Vec<u8>, std::io::Error>
{
    // Setup of variables necessary for going through the data
    let mut byte_buffer = [0u8; 1];                                             // A buffer for reading in a byte of data from the file
    let mut previous_byte_was_marker_prefix = false;                            // A boolean for remembering if the previous byte was a marker prefix (0xFF)

    loop
    {
        // Read next byte into buffer
        cursor.read_exact(&mut byte_buffer)?;

        if previous_byte_was_marker_prefix
        {
            // Check if this is the end of the file. In that case, the length
            // data can't be read and we need to return prematurely. 
            // This is why this case can't be included in the match afterwards.
            if byte_buffer[0] == 0xd9                                           // EOI marker
            {
                // No more data to read in
                return io_error!(Other, "No EXIF data found!");
            }

            // Read in the length of the segment
            // (which follows immediately after the marker)
            let mut length_buffer = [0u8; 2];
            cursor.read_exact(&mut length_buffer)?;

            // Decode the length to determine how much more data there is
            let length = from_u8_vec_res_macro!(u16, &length_buffer, &Endian::Big)?;
            if length < 2
            {
                return io_error!(InvalidData, "Mangled JPG data encountered!");
            }

            let remaining_length = (length - 2) as usize;

            match byte_buffer[0]
            {
                0xe1 => {                                                       // APP1 marker
                    // Read in & return the remaining data
                    let mut app1_buffer = vec![0u8; remaining_length];
                    cursor.read_exact(&mut app1_buffer)?;

                    return Ok(app1_buffer);
                },

                0xda => {                                                       // SOS marker
                    // The start of scan (SOS) segment is followed by a blob of
                    // image data, the entropy-coded segment (ECS), which has no
                    // information regarding its length (as it may easily be 
                    // bigger than the max segment length of 64kb)

                    // So, we have to scan byte-for-byte at this point until
                    // a marker prefix comes up that is NOT
                    // - followed by a restart marker (D0 - D7) or 
                    // - a data FF (followed by 00)

                    // So, start by skipping the SOS segment
                    cursor.seek(SeekFrom::Current(remaining_length as i64))?;

                    // And skip the ECS
                    skip_ecs(cursor)?;
                }

                _ => {                                                          // Every other marker
                    // Skip this segment
                    cursor.seek(SeekFrom::Current(remaining_length as i64))?;
                },
            }

            previous_byte_was_marker_prefix = false;
        }
        else
        {
            previous_byte_was_marker_prefix = byte_buffer[0] == JPG_MARKER_PREFIX;
        }
    }
}