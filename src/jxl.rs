// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;

use crate::endian::Endian;
use crate::metadata::Metadata;
use crate::u8conversion::*;
use crate::general_file_io::*;
use crate::util::insert_multiple_at;
use crate::util::range_remove;

pub(crate) const JXL_SIGNATURE:          [u8; 2]  = [0xff, 0x0a];
pub(crate) const ISO_BMFF_JXL_SIGNATURE: [u8; 12] = [
    0x00, 0x00, 0x00, 0x0c,
    0x4a, 0x58, 0x4c, 0x20,
    0x0d, 0x0a, 0x87, 0x0a
];

pub(crate) const FTYP_BOX: [u8; 20] = [
    0x00, 0x00, 0x00, 0x14, // length of this box
    0x66, 0x74, 0x79, 0x70, // "ftyp"
    0x6a, 0x78, 0x6c, 0x20, // "jxl "
    0x00, 0x00, 0x00, 0x00, // minor version
    0x6a, 0x78, 0x6c, 0x20  // "jxl " - yes, again
];

pub(crate) const ISO_BMFF_EXIF_MINOR_VERSION: [u8; 4] = [0x00, 0x00, 0x00, 0x06];

pub(crate) const BROB_BOX: [u8; 4] = [0x62, 0x72, 0x6f, 0x62];

#[non_exhaustive]
struct IsoBmffBoxType;

impl IsoBmffBoxType {
    pub const EXIF: [u8; 4] = [0x45, 0x78, 0x69, 0x66]; // "Exif"
    pub const FTYP: [u8; 4] = [0x66, 0x74, 0x79, 0x70]; // "ftyp"
    pub const JXL:  [u8; 4] = [0x4a, 0x58, 0x4c, 0x20]; // "JXL "
    pub const JXLC: [u8; 4] = [0x6a, 0x78, 0x6c, 0x63]; // "jxlc"
}

/// Checks if the given file buffer vector starts with the necessary bytes that
/// indicate a JXL file in an ISO BMFF container
/// These containers are divided into boxes, each consisting of
/// - 4 bytes that give the box size n
/// - 4 bytes that give the box type (e.g. "jxlc" for a JXL codestream)
/// - n-8 bytes of data
/// 
/// These 12 bytes are for checking the first box that is the same for all such
/// stored JXL images
fn
starts_with_iso_bmff_signature
(
    file_buffer: &[u8],
)
-> bool
{
    return file_buffer[0..12].iter()
        .zip(ISO_BMFF_JXL_SIGNATURE.iter())
        .filter(|&(read, constant)| read == constant)
        .count() == ISO_BMFF_JXL_SIGNATURE.len();
}

/// There are two types of JXL image files: One are simply a JXL codestream,
/// which start with the `JXL_SIGNATURE` bytes 0xFF0A. These can *not* store
/// any metadata.
/// The other type is contained within a ISO BMFF container and are able to 
/// include EXIF metadata. 
/// If this function returns true, the image needs to be converted first before
/// it is able to hold any metadata
fn
starts_with_jxl_signature
(
    file_buffer: &[u8],
)
-> bool
{
    return file_buffer[0..2].iter()
        .zip(JXL_SIGNATURE.iter())
        .filter(|&(read, constant)| read == constant)
        .count() == JXL_SIGNATURE.len();
}

fn
check_signature
(
    file_buffer: &[u8],
)
-> Result<(), std::io::Error>
{
    if starts_with_jxl_signature(file_buffer)
    {
        return io_error!(Other, "Simple JXL codestream file - No metadata!");
    }

    if !starts_with_iso_bmff_signature(file_buffer)
    {
        return io_error!(Other, "This isn't ISO BMFF JXL data!");
    }

    return Ok(());
}

fn
file_check_signature
(
    path: &Path
)
-> Result<File, std::io::Error>
{
    let mut file = open_write_file(path)?;

    let mut signature_buffer = [0u8; 12];
    let bytes_read = file.read(&mut signature_buffer)?;

    if bytes_read != 12
    {
        return io_error!(InvalidData, "Can't open JXL file - Can't read signature!");
    }

    check_signature(&signature_buffer)?;

    return Ok(file);
}



pub(crate) fn
clear_metadata
(
    file_buffer: &mut Vec<u8>
)
-> Result<(), std::io::Error>
{
    check_signature(file_buffer)?;

    let mut position = 0;

    loop
    {
        if position >= file_buffer.len() { return Ok(()); }

        // Get the first 4 bytes at the current cursor position to determine
        // the length of the current box 
        let length_buffer = file_buffer[position..position+4].to_vec();
        let length        = from_u8_vec_macro!(u32, &length_buffer, &Endian::Big) as usize;

        // Next, read the box type
        let type_buffer = file_buffer[position+4..position+8].to_vec();

        if box_contains_exif(
            &mut Cursor::new(
                file_buffer[position+8..position+12].to_vec()
            ), 
            [type_buffer[0], type_buffer[1], type_buffer[2], type_buffer[3]]
        )?
        {
            range_remove(file_buffer, position, position+length);
        }
        else
        {
            // Not an EXIF box so skip it
            position += length;
        }
    }
}

pub(crate) fn
file_clear_metadata
(
    path: &Path
)
-> Result<(), std::io::Error>
{
    let mut file = file_check_signature(path)?;

    let mut length_buffer = [0u8; 4];
    let mut type_buffer   = [0u8; 4];

    loop
    {
        let position        = file.stream_position()?;
        let old_file_length = file.metadata().unwrap().len();
        if position >= old_file_length { return Ok(()); }

        file.read_exact(&mut length_buffer)?;
        file.read_exact(&mut type_buffer)?;

        let length = from_u8_vec_macro!(u32, &length_buffer, &Endian::Big) as usize;

        if box_contains_exif(&mut file, type_buffer)?
        {
            // Seek past the EXIF box ...
            file.seek(SeekFrom::Current((length-8) as i64))?;


            // ... copy everything from here onwards into a buffer ...
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            // ... seek back to the start of the EXIF box ...
            file.seek(std::io::SeekFrom::Start(position))?;

            // ... overwrite everything from here onward ...
            file.write_all(&buffer)?;
            file.seek(std::io::SeekFrom::Start(position))?;

            // ... and finally update the file size - otherwise there will be
            // duplicate bytes at the end!
            file.set_len(old_file_length - length as u64)?;
        }
        else
        {
            // Not an EXIF box so skip it
            assert_eq!(position+8, file.stream_position()?);
            file.seek(SeekFrom::Current((length-8) as i64))?;
        }
    }
}


fn
check_brob_type_for_exif
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<bool, std::io::Error>
{
    // Check if the next for 4 bytes say 'Exif'
    let mut brob_type = [0u8; 4];
    cursor.read_exact(&mut brob_type)?;

    // Seek back to position prior to brob type

    cursor.seek(SeekFrom::Current(-4))?;

    return Ok(brob_type == EXIF);
}



fn
box_contains_exif
<T: Seek + Read>
(
    cursor:      &mut T,
    type_buffer:  [u8; 4],
)
-> Result<bool, std::io::Error>
{
    if type_buffer == EXIF
    {
        return Ok(true);
    }
    if type_buffer == BROB_BOX && check_brob_type_for_exif(cursor)?
    {
        return Ok(true);
    }

    return Ok(false);
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

    return generic_read_metadata(&mut cursor);
}

pub(crate) fn
file_read_metadata
(
    path: &Path
)
-> Result<Vec<u8>, std::io::Error>
{
    let mut file = open_read_file(path)?;

    // Read first 12 bytes and check that we have a ISO BMFF file
    let mut first_12_bytes = [0u8; 12];
    let     bytes_read     = file.read(&mut first_12_bytes)?;

    if bytes_read != 12
    {
        return io_error!(InvalidData, "Can't open JXL file - Can't read & check ISO BMFF signature!");
    }

    check_signature(&first_12_bytes)?;

    return generic_read_metadata(&mut file);
}

fn
generic_read_metadata
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<Vec<u8>, std::io::Error>
{
    loop
    {
        // Get the first 4 bytes at the current cursor position to determine
        // the length of the current box (and account for the 8 bytes of length
        // and box type)
        let mut length_buffer = [0u8; 4];
        cursor.read_exact(&mut length_buffer)?;
        let length = from_u8_vec_macro!(u32, &length_buffer, &Endian::Big) - 8;

        // Next, read the box type
        let mut type_buffer = [0u8; 4];
        cursor.read_exact(&mut type_buffer)?;

        match type_buffer
        {
            EXIF => {
                // Skip the next 4 bytes (which contain the minor version???)
                cursor.seek(SeekFrom::Current(4))?;

                // `length-4` because of the previous relative seek operation
                let mut exif_buffer = vec![0u8; (length-4) as usize];
                cursor.read_exact(&mut exif_buffer)?;

                return Ok(exif_buffer);
            },

            BROB_BOX => { // -> Brotli encoded data

                let position = cursor.stream_position()? as usize;

                if check_brob_type_for_exif(cursor)?
                {
                    // Skip the next 4 bytes (which contain the minor version???)
                    cursor.seek(SeekFrom::Current(4))?;

                    let mut compressed_exif_buffer = vec![
                        0u8; 
                        (length-4) as usize
                    ];
                    cursor.read_exact(&mut compressed_exif_buffer)?;
                    
                    let mut decompressed_exif_buffer = Vec::new();

                    match brotli::BrotliDecompress(
                        &mut Cursor::new(compressed_exif_buffer), 
                        &mut decompressed_exif_buffer
                    ) 
                    {
                        Ok(_)  => (),
                        Err(e) => return Err(e)
                    };

                    // Ignore the next 4 bytes (I guess for the same reason 
                    // as above - some sort of minor version?)
                    return Ok(decompressed_exif_buffer[4..].to_vec());
                }
                else 
                {
                    cursor.seek(SeekFrom::Start(position as u64 + length as u64))?;
                }
            }

            _ => {
                // Not an EXIF box so skip it
                cursor.seek(SeekFrom::Current(length as i64))?;
            }
        }
    }
}

fn
encode_metadata_jxl
(
    exif_vec: &[u8],
)
-> Vec<u8>
{
    let exif_box_length = 0                        // Length has to include
        + 4                                        // - the length field
        + IsoBmffBoxType::EXIF.len()        as u32 // - the box type 
        + ISO_BMFF_EXIF_MINOR_VERSION.len() as u32 // - the minor version
        + EXIF_HEADER.len()                 as u32 // - the exif header
        + exif_vec.len()                    as u32 // - the exif data
    ;
    
    let mut jxl_exif = Vec::new();
    jxl_exif.extend(to_u8_vec_macro!(u32, &exif_box_length, &Endian::Big));
    jxl_exif.extend(IsoBmffBoxType::EXIF);
    jxl_exif.extend(ISO_BMFF_EXIF_MINOR_VERSION);
    jxl_exif.extend(EXIF_HEADER.iter());
    jxl_exif.extend(exif_vec.iter());

    return jxl_exif;
}

fn
find_insert_position
(
    file_buffer: &Vec<u8>
)
-> Result<usize, std::io::Error>
{
    let mut cursor = Cursor::new(file_buffer);

    loop
    {
        // Get the first 4 bytes at the current cursor position to determine
        // the length of the current box (and account for the 8 bytes of length
        // and box type)
        let mut length_buffer = [0u8; 4];
        cursor.read_exact(&mut length_buffer)?;
        let length = from_u8_vec_macro!(u32, &length_buffer, &Endian::Big) - 8;

        // Next, read the box type
        let mut type_buffer = [0u8; 4];
        cursor.read_exact(&mut type_buffer)?;

        match type_buffer
        {
            IsoBmffBoxType::JXL |
            IsoBmffBoxType::FTYP => {
                // Place exif box after these boxes
                cursor.seek(SeekFrom::Current(length as i64))?;
            }
            _ => {
                return Ok(cursor.position() as usize - 8);
            }
        }
    }
}

pub(crate) fn 
write_metadata
(
    file_buffer: &mut Vec<u8>,
    metadata:    &Metadata
)
-> Result<(), std::io::Error> 
{
    if starts_with_jxl_signature(file_buffer)
    {
        // Need to modify the file_buffer first so that it is a ISO BMFF 
        let mut new_file_buffer = Vec::new();

        // Start of the new file
        new_file_buffer.extend(ISO_BMFF_JXL_SIGNATURE);
        new_file_buffer.extend(FTYP_BOX);

        // JXL codestream box
        // - length of box (including 4 bytes of length & type fields each)
        // - type field
        // - data
        let jxlc_box_length = file_buffer.len() as u32 + 8;
        new_file_buffer.extend(to_u8_vec_macro!(u32, &jxlc_box_length, &Endian::Big));
        new_file_buffer.extend(IsoBmffBoxType::JXLC);
        new_file_buffer.append(file_buffer);

        // Replace file buffer
        *file_buffer = new_file_buffer;
    }

    // Remove old metadata
    clear_metadata(file_buffer)?;
    
    // Insert new metadata
    let mut encoded_metadata = encode_metadata_jxl(&metadata.encode()?);
    let     insert_position  = find_insert_position(file_buffer)?;
    insert_multiple_at(file_buffer, insert_position, &mut encoded_metadata);

    return Ok(());
}

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
    // The called function handles the removal of old metadata and the JXL
    // specific encoding, so we pass only the generally encoded metadata here
    write_metadata(&mut file_buffer, metadata)?;

    // Seek back to start & write the file
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&file_buffer)?;

    return Ok(());
}