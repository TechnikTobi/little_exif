// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::general_file_io::io_error;

/// Reads in the next 1 bytes, starting at the current position of the cursor.
/// The function call advances the cursor by 1 bytes.
pub(crate) fn
read_1_bytes
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<[u8; 1], std::io::Error>
{
    // Read in the 1 bytes
    let mut field = [0u8; 1];
    let bytes_read = cursor.read(&mut field)?;

    // Check that indeed 1 bytes were read
    if bytes_read != 1
    {
        return io_error!(Other, "Could not read the next 1 bytes!");
    }

    return Ok(field);
}

/// Reads in the next 2 bytes, starting at the current position of the cursor.
/// The function call advances the cursor by 2 bytes.
pub(crate) fn
read_2_bytes
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<[u8; 2], std::io::Error>
{
    // Read in the 2 bytes
    let mut field = [0u8; 2];
    let bytes_read = cursor.read(&mut field)?;

    // Check that indeed 2 bytes were read
    if bytes_read != 2
    {
        return io_error!(Other, "Could not read the next 2 bytes!");
    }

    return Ok(field);
}

/// Reads in the next 3 bytes, starting at the current position of the cursor.
/// The function call advances the cursor by 3 bytes.
pub(crate) fn
read_3_bytes
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<[u8; 3], std::io::Error>
{
    // Read in the 3 bytes
    let mut field = [0u8; 3];
    let bytes_read = cursor.read(&mut field)?;

    // Check that indeed 3 bytes were read
    if bytes_read != 3
    {
        return io_error!(Other, "Could not read the next 3 bytes!");
    }

    return Ok(field);
}

/// Reads in the next 4 bytes, starting at the current position of the cursor.
/// The function call advances the cursor by 4 bytes.
pub(crate) fn
read_4_bytes
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<[u8; 4], std::io::Error>
{
    // Read in the 4 bytes
    let mut field = [0u8; 4];
    let bytes_read = cursor.read(&mut field)?;

    // Check that indeed 4 bytes were read
    if bytes_read != 4
    {
        return io_error!(Other, "Could not read the next 4 bytes!");
    }

    return Ok(field);
}

/// Reads in the next 8 bytes, starting at the current position of the cursor.
/// The function call advances the cursor by 8 bytes.
pub(crate) fn
read_8_bytes
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<[u8; 8], std::io::Error>
{
    // Read in the 8 bytes
    let mut field = [0u8; 8];
    let bytes_read = cursor.read(&mut field)?;

    // Check that indeed 8 bytes were read
    if bytes_read != 8
    {
        return io_error!(Other, "Could not read the next 8 bytes!");
    }

    return Ok(field);
}

/// Reads in the next 16 bytes, starting at the current position of the cursor.
/// The function call advances the cursor by 16 bytes.
pub(crate) fn
read_16_bytes
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<[u8; 16], std::io::Error>
{
    // Read in the 16 bytes
    let mut field = [0u8; 16];
    let bytes_read = cursor.read(&mut field)?;

    // Check that indeed 16 bytes were read
    if bytes_read != 16
    {
        return io_error!(Other, "Could not read the next 16 bytes!");
    }

    return Ok(field);
}

/// Reads in a u16 in big endian format at the current cursor position
/// The function call advances the cursor by 2 bytes.
pub(crate) fn
read_be_u16
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<u16, std::io::Error>
{
    let bytes = read_2_bytes(cursor)?;
    return Ok(bytes[0] as u16 * 256 + bytes[1] as u16);
}

/// Reads in a u32 in big endian format at the current cursor position
/// The function call advances the cursor by 4 bytes.
pub(crate) fn
read_be_u32
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<u32, std::io::Error>
{
    let     bytes = read_4_bytes(cursor)?;
    let mut value = 0u32;

    for byte in bytes
    {
        value = value * 256 + byte as u32;
    }

    return Ok(value);
}

/// Reads in a u64 in big endian format at the current cursor position
/// The function call advances the cursor by 8 bytes.
pub(crate) fn
read_be_u64
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<u64, std::io::Error>
{
    let     bytes = read_8_bytes(cursor)?;
    let mut value = 0u64;

    for byte in bytes
    {
        value = value * 256 + byte as u64;
    }

    return Ok(value);
}

pub(crate) fn
read_null_terminated_string
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<String, std::io::Error>
{
    let mut string_buffer    = Vec::new();
    let mut character_buffer = read_1_bytes(cursor)?;
    while character_buffer[0] != 0x00
    {
        string_buffer.push(character_buffer[0]);
        character_buffer = read_1_bytes(cursor)?;
    }

    return String::from_utf8(string_buffer).map_err(
        |_e| 
        std::io::Error::new
        (
            std::io::ErrorKind::InvalidData,
            "Could not convert byte data to UTF-8 string!"
        )
    );
}


/// Inserts a slice into a vector at a given offset, shifting elements 
/// starting at the offset towards the end.
/// Returns 0 (zero) if the operation was successful, non-zero if the offset 
/// is larger than the current length of the destination vector. In the latter 
/// case, everything stays untouched.
pub(crate) fn 
insert_multiple_at<T>
(
    vec_dst: &mut Vec<T>,
    offset:  usize,
    vec_src: &mut Vec<T>,
)
-> usize
where T: Copy 
{
    match (vec_dst.len(), vec_src.len()) 
    {
        (_, 0)           => 0,
        (current_len, _) => {

            // If this is true we return at this point as this would cause a
            // "gap" between existing and new vector contents
            if current_len < offset
            {
                return std::cmp::max(1, current_len);
            }

            // Reserve without over-allocation space needed for new elements
            vec_dst.reserve_exact(vec_src.len());

            let mut temp = vec_dst.split_off(offset);
            vec_dst.append(vec_src);
            vec_dst.append(&mut temp);

            return 0;
        },
    }
}

/// Removes a section in the middle of a vector. The element at index `start` 
/// is where the removal starts, up to the element prior to at index `end`
/// The element originally positioned at `end` will survive. 
pub(crate) fn
range_remove<T>
(
    vec:   &mut Vec<T>,
    start: usize,
    end:   usize
)
where T: Copy
{
    // Invalid input, nothing to do here
    if start > end { return; }

    let old_vec_len = vec.len();

    // Simply truncating is sufficient in this case
    if end >= old_vec_len { vec.truncate(start); return; }

    // Otherwise, move the elements starting at end over to the left
    for (dst_offset, src_index) in (end..old_vec_len).enumerate()
    {
        vec[start + dst_offset] = vec[src_index];
    }

    // Resize the vector to cut off any residue from the shifting operations
    let new_vec_len = old_vec_len - (end - start);
    vec.truncate(new_vec_len);
}

#[macro_export]
macro_rules! debug_println 
{
    (
        $($arg:tt)*
    ) 
    => 
    (
        /*
        #[cfg(debug_assertions)] 
        {
            print!("LITTLE EXIF DEBUG: ");
            println!($($arg)*);
        }
        */

        ()
    );
}


/*
/// Inserts a slice into a vector at a given offset, shifting elements 
/// starting at the offset towards the end.
/// Returns 0 (zero) if the operation was successful, non-zero if the offset 
/// is larger than the current length of the destination vector. In the latter 
/// case, everything stays untouched.
pub(crate) fn insert_multiple_at<T>
(
    vec_dst: &mut Vec<T>, 
    offset:  usize, 
    vec_src: &mut [T]
)
-> usize
where T: Copy 
{
    match (vec_dst.len(), vec_src.len()) 
    {
        (_, 0)           => 0,
        (current_len, _) => {

            // Elements that need to be moved to make way for the new ones
            let move_count = current_len - offset;

            // If this is less than 0 we return at this point as this would
            // cause a "gap" between existing and new vector contents
            // (move_count is usize and thus can't be less than 0)
            if current_len < offset
            {
                return std::cmp::max(1, current_len);
            }

            // Reserve without over-allocation space needed for new elements
            vec_dst.reserve_exact(vec_src.len());

            unsafe
            {
                // Pointer to the first location where vec_src elements will
                // be placed. 
                // Called `src` at this stage as previously it has to serve
                // as source for elements that require to be copied to the
                // right to make way 
                let src = vec_dst.as_mut_ptr().offset(offset as isize);

                // Set the new length of the vector after the operation
                vec_dst.set_len(current_len + vec_src.len());

                // Check if there are any elements that require to be moved
                if move_count > 0 
                {
                    let dst = src.offset(vec_src.len() as isize);
                    std::ptr::copy(
                        src,                                                    // Source pointer
                        dst,                                                    // Destination pointer
                        move_count                                              // How many elements to copy
                    );
                }

                // Copy the new elements at the new "free" locations
                // The previous source pointer `src` becomes the destination of
                // the new elements to be inserted
                // In contrast to the previous copy we can here be sure that
                // the source and destination don't overlap
                std::ptr::copy_nonoverlapping(
                    vec_src.as_mut_ptr(),                                       // Source pointer
                    src,                                                        // Destination pointer
                    vec_src.len()                                               // How many elements to copy (here: ALL)
                );
            }

            return 0;
        },
    }
}
*/