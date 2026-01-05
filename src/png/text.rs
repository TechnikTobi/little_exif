// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use miniz_oxide::deflate::compress_to_vec_zlib;
use miniz_oxide::inflate::decompress_to_vec_zlib;

use crate::general_file_io::io_error;

/// This gets the keyword of a $TEXT chunk.
/// Fortunately, this is the same for tEXt, zTXt and iTXt, as they all
/// start with a keyword that is followed by a NUL separator.
pub(crate) fn get_keyword_from_text_chunk(chunk_data: &[u8]) -> String {
    let mut keyword_buffer = Vec::new();
    for character in chunk_data {
        if *character == 0x00 {
            break;
        }
        keyword_buffer.push(*character);
    }
    return String::from_utf8(keyword_buffer).unwrap();
}

pub(crate) fn get_data_from_text_chunk(
    chunk_name: &str,
    chunk_data: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    // The keyword length is required in all cases for determining the start
    // of the actual data
    let keyword_length = get_keyword_from_text_chunk(chunk_data).len();

    match chunk_name {
        "tEXt" => {
            // Find start of data
            // For this we take the keyword length and add 1 for the NUL byte
            let data_start = keyword_length + 1;

            return Ok(chunk_data[data_start..].to_vec());
        }

        "zTXt" => {
            // Find start of data
            // For this we take the keyword length and add 2 for the NUL byte
            // and the byte representing the compression method
            let data_start = keyword_length + 2;

            // Check compression method
            if chunk_data[keyword_length + 1] != 0x00 {
                return io_error!(Other, "Unknown compression method for zTXt!");
            }

            // Decode zlib data
            if let Ok(decompressed_data) = decompress_to_vec_zlib(&chunk_data[data_start..]) {
                return Ok(decompressed_data);
            } else {
                return io_error!(Other, "Could not inflate compressed chunk data!");
            }
        }

        "iTXt" => {
            // We need more than just the keyword length
            let (compression_flag, compression_method, _keyword, language_tag, translated_keyword) =
                get_info_about_iTXt_chunk(chunk_data);

            let data_start = keyword_length // keyword
				+ 3                         // NUL, compression flag & method
				+ language_tag.len()        // language tag
				+ 1                         // NUL
				+ translated_keyword.len()  // translated keyword
				+ 1; // NUL

            if compression_flag == 0x00 {
                // No compression, simply return the data
                return Ok(chunk_data[data_start..].to_vec());
            }

            if compression_method != 0x00 {
                return io_error!(Other, "Unknown compression method for iTXt!");
            }

            // Decode zlib data
            if let Ok(decompressed_data) = decompress_to_vec_zlib(&chunk_data[data_start..]) {
                return Ok(decompressed_data);
            } else {
                return io_error!(Other, "Could not inflate compressed chunk data!");
            }
        }

        _ => {
            return io_error!(Other, "Unknown text chunk!");
        }
    }
}

pub(crate) fn construct_similar_with_new_data(
    chunk_name: &str,
    old_chunk_data: &[u8],
    new_data: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    // Note: data is just the text after the keyword an so on, while *chunk*
    // data describes the entire data field that includes the keyword, the
    // compression information and so on

    // The keyword and NUL will be needed in every cases:
    let keyword = get_keyword_from_text_chunk(old_chunk_data);
    let mut new_chunk_data = keyword.bytes().map(|byte| byte as u8).collect::<Vec<u8>>();
    new_chunk_data.push(0x00);

    match chunk_name {
        "tEXt" => {
            new_chunk_data.extend(new_data);
        }

        "zTXt" => {
            // Check compression method
            if old_chunk_data[keyword.len() + 1] != 0x00 {
                return io_error!(Other, "Unknown compression method for zTXt!");
            }

            new_chunk_data.extend(compress_to_vec_zlib(&new_data, 8).iter());
        }

        "iTXt" => {
            // We need more than just the keyword
            let (compression_flag, compression_method, _keyword, language_tag, translated_keyword) =
                get_info_about_iTXt_chunk(old_chunk_data);

            // Push compression information
            new_chunk_data.push(compression_flag);
            new_chunk_data.push(compression_method);

            // Add the language tag and translated keyword
            new_chunk_data.extend(language_tag.bytes().map(|byte| byte as u8));
            new_chunk_data.push(0x00);
            new_chunk_data.extend(translated_keyword.bytes().map(|byte| byte as u8));
            new_chunk_data.push(0x00);

            // Check compression
            if compression_flag == 0x00 {
                // No compression, simply add the new data
                new_chunk_data.extend(new_data);
            } else {
                if compression_method != 0x00 {
                    return io_error!(Other, "Unknown compression method for iTXt!");
                }
                new_chunk_data.extend(compress_to_vec_zlib(&new_data, 8).iter());
            }
        }

        _ => {
            return io_error!(Other, "Unknown text chunk!");
        }
    }

    return Ok(new_chunk_data);
}

/// The iTXt chunk is in its structure more complex than e.g. tEXt. The data
/// section consists of (from the specifications, see paragraph 11.3.3.4 of
/// https://www.w3.org/TR/png ):
/// - Keyword              1-79 bytes (character string)
/// - Null separator       1 byte (null character)
/// - Compression flag     1 byte
/// - Compression method   1 byte
/// - Language tag         0 or more bytes (character string)
/// - Null separator       1 byte (null character)
/// - Translated keyword   0 or more bytes
/// - Null separator       1 byte (null character)
/// - Text                 0 or more bytes
#[allow(non_snake_case)]
fn get_info_about_iTXt_chunk(
    chunk_data: &[u8],
) -> (
    u8,     // compression flag
    u8,     // compression method
    String, // keyword
    String, // language tag
    String, // translated keyword
) {
    // Tells us where we currently are in the chunk data
    let mut chunk_counter = 0;

    // Buffers for the different attributes
    let mut keyword_buffer = Vec::new();

    loop {
        if chunk_data[chunk_counter] != 0x00 {
            keyword_buffer.push(chunk_data[chunk_counter]);
            chunk_counter += 1;
        } else {
            break;
        }
    }

    let _null_separator_1 = chunk_data[chunk_counter + 0];
    let compression_flag = chunk_data[chunk_counter + 1];
    let compression_method = chunk_data[chunk_counter + 2];

    chunk_counter += 3;
    let mut language_tag_buffer = Vec::new();

    loop {
        if chunk_data[chunk_counter] != 0x00 {
            language_tag_buffer.push(chunk_data[chunk_counter]);
            chunk_counter += 1;
        } else {
            break;
        }
    }

    chunk_counter += 1;
    let mut translated_keyword_buffer = Vec::new();

    loop {
        if chunk_data[chunk_counter] != 0x00 {
            translated_keyword_buffer.push(chunk_data[chunk_counter]);
            chunk_counter += 1;
        } else {
            break;
        }
    }

    return (
        compression_flag,
        compression_method,
        String::from_utf8(keyword_buffer).unwrap_or_default(),
        String::from_utf8(language_tag_buffer).unwrap_or_default(),
        String::from_utf8(translated_keyword_buffer).unwrap_or_default(),
    );
}
