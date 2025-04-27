// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub mod get;
pub mod set;

use core::panic;
use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::vec;

use log::warn;

use crate::endian::*;
use crate::exif_tag::decode::decode_tag_with_format_exceptions;
use crate::exif_tag::ExifTag;
use crate::exif_tag::TagType;
use crate::exif_tag_format::ExifTagFormat;
use crate::general_file_io::io_error;
use crate::metadata::Metadata;
use crate::u8conversion::from_u8_vec_macro;
use crate::u8conversion::to_u8_vec_macro;
use crate::u8conversion::U8conversion;

/// Useful constants for dealing with IFDs: The length of a single IFD entry is
/// equal to 12 bytes, as the entry consists of the tags hex value (2 byte), 
/// the format (2 byte), the number of components (4 byte) and the value/offset
/// section (4 byte).
/// The four zeros tell us that this is the last IFD in its sequence and there
/// is no link to another IFD
const IFD_ENTRY_LENGTH: u32     = 12;
const IFD_END_NO_LINK:  [u8; 4] = [0x00, 0x00, 0x00, 0x00];

/// The different types of Image File Directories (IFD). A generic IFD is one
/// without further specialization, like e.g. IFD0. The generic IFDs start
/// with IFD0, which is located via the offset at the start of the TIFF data. 
/// The next IFD (in this case: IFD1) is then located via the link offset at
/// the end of IFD0. 
/// Other IFDs, like e.g. the ExifIFD, are linked via offset tags (in case of 
/// the ExifIFD offset: 0x8769) that are located in the respective generic IFD 
/// (most of them in IFD0).
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd)]
#[allow(non_snake_case, non_camel_case_types)]
pub enum
ExifTagGroup
{
	GENERIC,
	EXIF,
	INTEROP,
	// MAKERNOTES, // TODO: Decide what to do with maker notes stuff...
	GPS,
}

/// The value of `belongs_to_generic_ifd_nr` tells us what generic IFD this
/// specific IFD belongs to, e.g. `0` would indicate that it belongs (or is)
/// IFD0. 
#[derive(Clone, Debug)]
pub struct
ImageFileDirectory
{
	tags:                      Vec<ExifTag>,
	ifd_type:                  ExifTagGroup,
	belongs_to_generic_ifd_nr: u32,
}

impl
ImageFileDirectory
{

	pub fn
	new_with_tags
	(
		tags:  Vec<ExifTag>,
		group: ExifTagGroup,
		nr:    u32
	)
	-> Self
	{
		ImageFileDirectory { tags: tags, ifd_type: group, belongs_to_generic_ifd_nr: nr }
	}

	/// Sorts the tags according to their hex value
	/// See TIFF 6.0 Specification: "The entries in an IFD must be sorted in 
	/// ascending order by Tag." (page 15/121)
	pub(crate) fn
	sort_tags
	(
		&mut self
	)
	{
		self.tags.sort_by(
			|a, b|
			a.as_u16().cmp(&b.as_u16())
		);
	}

	/// If everything goes Ok and there is enough data to unpack, this returns
	/// the offset to the next generic IFD that needs to be processed.
	pub(crate) fn
	decode_ifd
	(
		data_cursor:         &mut Cursor<&Vec<u8>>,
		data_begin_position:      u64,                                          // Stays the same for all calls to this function while decoding
		endian:              &    Endian,
		group:               &    ExifTagGroup,
		generic_ifd_nr:           u32,                                          // Reuse value for recursive calls; only gets incremented by caller
		insert_into:         &mut Vec<ImageFileDirectory>,                      // Stays the same for all calls to this function while decoding
	)
	-> Result<Option<u32>, std::io::Error>
	{
		////////////////////////////////////////////////////////////////////////
		// PREPARATION 

		// Backup the entry position where this IFD started
		let data_cursor_entry_position = data_cursor.position();

		// Check if there is enough data to decode an IFD
		if (data_cursor.get_ref().len() as i64 - data_cursor_entry_position as i64) < 6i64
		{
			return Ok(None);
		}

		// The first two bytes give us the number of entries in this IFD
		let mut number_of_entries_buffer = vec![0u8; 2];
		data_cursor.read_exact(&mut number_of_entries_buffer)?;
		let number_of_entries = from_u8_vec_macro!(u16, &number_of_entries_buffer.to_vec(), endian);

		// Check that there is enough data to unpack
		let required = 0
			+ 2
			+ IFD_ENTRY_LENGTH as usize * number_of_entries as usize 
			+ IFD_END_NO_LINK.len();
		let available = (0
			+ data_cursor.get_ref().len() as i64 
			- data_cursor_entry_position  as i64) as usize;

		if required > available
		{
			return io_error!(Other, format!("Not enough data to decode IFD! Required: {} Available: {}", required, available));
		}

		// Temporarily storing specific tags that have been decoded
		// This has to do with data offset tags that are interconnected with
		// other tags.
		// For example, for decoding the StripOffsets we also need the 
		// StripByteCounts to know how many bytes each strip has
		let mut strip_tags:     (Option<ExifTag>, Option<ExifTag>) = (None, None);
		let mut thumbnail_info: (Option<ExifTag>, Option<ExifTag>) = (None, None);
		// Others following here in the future...

		////////////////////////////////////////////////////////////////////////
		// TAG-DECODING

		// Storing all tags while decoding
		let mut tags = Vec::new();

		// loop through the entries - assumes that the value stored in
		// `number_of_entries` is correct
		for _ in 0..number_of_entries
		{
			// Read the entry into a buffer
			let mut entry_buffer = vec![0u8; IFD_ENTRY_LENGTH as usize];
			data_cursor.read_exact(&mut entry_buffer)?;

			// Decode the first 8 bytes with the tag, format and component number
			let hex_tag              = from_u8_vec_macro!(u16, &entry_buffer[0..2].to_vec(), endian);
			let hex_format           = from_u8_vec_macro!(u16, &entry_buffer[2..4].to_vec(), endian);
			let hex_component_number = from_u8_vec_macro!(u32, &entry_buffer[4..8].to_vec(), endian);

			// Decode the format
			// TODO: What to do in case these two differ but the given format
			// can be casted into the expected one, e.g. R64U to R64S?
			let format;
			if let Some(decoded_format) = ExifTagFormat::from_u16(hex_format)
			{
				format = decoded_format;
			}
			else
			{
				return io_error!(Other, format!("Illegal format value: {}", hex_format));
			}

			// Calculating the number of required bytes to determine if next
			// 4 bytes are data or an offset to data
			// Note: It is expected that the format here is "correct" in the
			// sense that it tells us whether or not an offset is used for the
			// data even if the given format in the image file is not the
			// right/default one for the currently processed tag according to 
			// the exif specification. 
			let byte_count = format.bytes_per_component() * hex_component_number;

			let raw_data;
			if byte_count > 4
			{
				// Compute the offset
				let hex_offset = from_u8_vec_macro!(u32, &entry_buffer[8..12].to_vec(), endian);

				// Backup current position & go to offset position
				let backup_position = data_cursor.position();
				data_cursor.set_position(data_begin_position);
				data_cursor.seek(std::io::SeekFrom::Current(hex_offset as i64))?;

				// Read the raw data
				let mut raw_data_buffer = vec![0u8; byte_count as usize];
				data_cursor.read_exact(&mut raw_data_buffer)?;
				raw_data = raw_data_buffer.to_vec();
			
				// Rewind the cursor to the start of the next entry
				data_cursor.set_position(backup_position);
			}
			else
			{
				// The 4 bytes are the actual data
				// Note: This may actually be *less* than 4 bytes! 
				raw_data = entry_buffer[8..(8+byte_count as usize)].to_vec();
			}

			// Try to get the tag via its hex value
			let tag_result = ExifTag::from_u16(hex_tag, group);

			// Start of by checking if this is an unknown tag
			if tag_result.is_err()
			{
				// Note: `from_u16_with_data` can NOT be called initially due
				// to some possible conversion of data needed, e.g. INT16U to
				// INT32U, which is not accounted for yet at this stage
				tags.push(ExifTag::from_u16_with_data(
					hex_tag, 
					&format, 
					&raw_data, 
					&endian, 
					group
				).unwrap());
				continue;
			}

			// We can now safely unwrap the result as it can't be an error
			let mut tag = tag_result.unwrap();

			// If this is an IFD offset tag, perform a recursive call
			if let TagType::IFD_OFFSET(subifd_group) = tag.get_tag_type()
			{
				// Compute the offset to the SubIFD and save the current position
				let offset          = from_u8_vec_macro!(u32, &raw_data, endian) as usize;
				let backup_position = data_cursor.position();

				// Go to the SubIFD offset and decode that
				data_cursor.set_position(data_begin_position);
				data_cursor.seek(std::io::SeekFrom::Current(offset as i64))?;

				let subifd_decode_result = Self::decode_ifd(
					data_cursor,
					data_begin_position,
					endian,
					&subifd_group,
					generic_ifd_nr,
					insert_into,
				);

				// Check that this actually worked
				if let Ok(_subifd_result) = subifd_decode_result
				{
					// Assert result, restore old cursor position & continue

					// Disabled assert as of issue #31
					// The idea behind this assert was that, as we are decoding
					// a SubIFD, there shouldn't be a link after the last entry
					// to another IFD and those 4 bytes are expected to be zero
					// and we get a Ok(None) from the recursive call back.
					
					// assert_eq!(subifd_result, None);

					// However, it is possible that those 4 bytes don't exist
					// at all and they are part of some offset data, possibly
					// even from another IFD! 
					// So, for now we just assume that `subifd_result` is not
					// of relevance until evidence suggests otherwise.
					
					data_cursor.set_position(backup_position);
					continue;
				}
				else
				{
					return io_error!(Other, format!("Could not decode SubIFD {:?}:\n  {}", subifd_group, subifd_decode_result.err().unwrap()));
				}
			}

			// At this point we check if the format is actually what we expect
			// it to be and convert it if possible
			tag = decode_tag_with_format_exceptions(
				&tag,
				format,
				&raw_data,
				endian,
				hex_tag,
				group
			)?;

			// Now we have at least confirmed that the format is ok (or has
			// been corrected). Next, we need to differ between the two other
			// tag types:
			if let TagType::DATA_OFFSET(_) = tag.get_tag_type()
			{
				match tag
				{
					ExifTag::StripOffsets(_, _) => {
						strip_tags.0 = Some(tag);
					},
					ExifTag::StripByteCounts(_) => {
						strip_tags.1 = Some(tag);
					},
					ExifTag::ThumbnailOffset(_, _) => {
						thumbnail_info.0 = Some(tag);
					},
					ExifTag::ThumbnailLength(_) => {
						thumbnail_info.1 = Some(tag);
					},
					_ => ()
				}

				// do NOT push these tags to the tags vector yet!
			}
			else // TagType::VALUE
			{
				// Simply push this tag onto the vector
				tags.push(tag);
			}

		} // end of for-loop

		////////////////////////////////////////////////////////////////////////
		// POST TAG-DECODING

		// At this stage we have decoded the tags themselves. 
		// However, the data offset tags need further processing (i.e. their 
		// data needs to be read as well)
		if strip_tags.0.is_some() && strip_tags.1.is_some()
		{
			// 0 -> offsets
			// 1 -> byte counts
			if let 
				(
					TagType::DATA_OFFSET(offsets),
					TagType::DATA_OFFSET(byte_counts)
				)
				= 
				(
					strip_tags.0.unwrap().get_tag_type(),
					strip_tags.1.unwrap().get_tag_type()
				)
			{
				let backup_position = data_cursor.position();

				let mut strip_data = Vec::new();

				// Gather the data from the offsets
				for (offset, byte_count) in offsets.iter().zip(byte_counts.iter())
				{
					data_cursor.set_position(data_begin_position);
					data_cursor.seek(std::io::SeekFrom::Current(*offset as i64))?;

					let mut data_buffer = vec![0u8; *byte_count as usize];
					data_cursor.read_exact(&mut data_buffer)?;
					strip_data.push(data_buffer);
				}

				// Push StripOffset tag to tags vector
				tags.push(ExifTag::StripOffsets(Vec::new(), strip_data));

				// Push StripByteCounts tag to tags vector
				tags.push(ExifTag::StripByteCounts(byte_counts));

				// Restore backup position
				data_cursor.set_position(backup_position);
			}
		}

		if thumbnail_info.0.is_some() && thumbnail_info.1.is_some()
		{
			// 0 -> offset
			// 1 -> length
			if let
				(
					TagType::DATA_OFFSET(offset),
					TagType::DATA_OFFSET(length)
				)
				=
				(
					thumbnail_info.0.unwrap().get_tag_type(),
					thumbnail_info.1.unwrap().get_tag_type()
				)
			{
				let backup_position = data_cursor.position();

				if offset.len() == 1 && length.len() == 1
				{
					let mut thumbnail_data = vec![0u8; length[0] as usize];

					// Gather the data at the offset
					data_cursor.set_position(data_begin_position);
					data_cursor.seek(std::io::SeekFrom::Current(offset[0] as i64))?;
					data_cursor.read_exact(&mut thumbnail_data)?;

					// Push ThumbnailOffset tag to tags vector
					tags.push(ExifTag::ThumbnailOffset(Vec::new(), thumbnail_data));

					// Also push ThumbnailLength tag to tags vector
					tags.push(ExifTag::ThumbnailLength(length));
				}
				else
				{
					warn!("Can't decode thumbnail! The ThumbnailOffset and ThumbnailLength tags are expected to contain exactly 1 INT32U value. However, they have {} and {} values.", offset.len(), length.len());
				}

				// Restore backup position
				data_cursor.set_position(backup_position);
			}
		}

		// Other offset tags here in the future...

		// At this point we are done with decoding the tags of this IFD and its
		// associated SubIFDs! 

		// Put the current IFD into the given, referenced vector
		insert_into.push(ImageFileDirectory { 
			tags: tags, 
			ifd_type: *group, 
			belongs_to_generic_ifd_nr: generic_ifd_nr
		});

		// Read in the link to the next IFD and check if its zero
		let mut next_ifd_link_buffer = vec![0u8; 4];
		if data_cursor.read_exact(&mut next_ifd_link_buffer).is_err()
		{
			// Covers the case that this IFD is stored at the very end of the
			// file and its a SubIFD that has no link at all
			return Ok(None);
		}

		let link_is_zero = next_ifd_link_buffer.iter()
			.zip(IFD_END_NO_LINK.iter())
			.filter(|&(read, constant)| read == constant)
			.count() == IFD_END_NO_LINK.len();

		if link_is_zero
		{
			return Ok(None);
		}
		return Ok(Some(from_u8_vec_macro!(u32, &next_ifd_link_buffer, endian)));
	}



	/// Recursively encodes IFDs
	/// Returns
	/// - an index position where the 4 bytes for the link to the next IFD are located
	/// - the offset of the encoded IFD, to be used for linking to this IFD
	pub(crate) fn
	encode_ifd
	(
		&self,
		data:                       &Metadata,
		ifds_with_offset_info_only: &mut Vec<ImageFileDirectory>,
		encode_vec:                 &mut Vec<u8>,
		current_offset:             &mut u32
	)
	-> Result<(u64, Vec<u8>), std::io::Error>
	{

		// Store all relevant tags (IFD tags + offset tags) in a temporary 
		// location and sort them there
		let all_relevant_tags = self.tags.iter().chain(ifds_with_offset_info_only
			.iter()
			.filter(|ifd| 
				ifd.get_generic_ifd_nr() == self.get_generic_ifd_nr() &&
				ifd.get_ifd_type()       == self.get_ifd_type()
			)
			.next().unwrap().get_tags()
			.iter()).cloned().collect::<Vec<ExifTag>>();

		// Start writing this IFD by adding the number of entries
		let count_entries = all_relevant_tags.iter().filter(
			|tag| tag.is_writable() || 
			if let TagType::IFD_OFFSET(_)  = tag.get_tag_type() { true } else { false } ||
			if let TagType::DATA_OFFSET(_) = tag.get_tag_type() { true } else { false }
		).count() as u16;

		encode_vec.extend(to_u8_vec_macro!(u16, &count_entries, &data.get_endian()).iter());

		// Remember the current offset as this is needed to address this IFD
		// and link to it from other IFDs
		let ifd_offset     = current_offset.clone();
		let ifd_offset_vec = to_u8_vec_macro!(u32, &ifd_offset, &data.get_endian());

		// Advance offset address to the point after the entries and provide
		// offset area vector
		*current_offset += 0
			+ 2                                                                 // length of entry count section
			+ IFD_ENTRY_LENGTH * count_entries as u32
			+ IFD_END_NO_LINK.len()            as u32
		;
		let mut ifd_offset_area: Vec<u8>;

		// Ensure that offset data is aligned properly
		let alignment_count = (4 - *current_offset % 4) % 4;
		*current_offset += alignment_count;
		ifd_offset_area = vec![0u8; alignment_count as usize];
		

		// Write directory entries to the vector
		for tag in &all_relevant_tags
		{
			// Skip tags that can't be written
			if !tag.is_writable()
			{
				// But don't skip tags that describe offsets to IFDs or Data!
				if let TagType::IFD_OFFSET(_) = tag.get_tag_type() {}
				else if let TagType::DATA_OFFSET(_) = tag.get_tag_type() {}
				else { continue; }
			}

			// Need to differentiate at this stage as we have to access e.g. the 
			// StripOffsets that are stored in a local vec
			let value = match tag.get_tag_type()
			{
				TagType::VALUE => {
					tag.value_as_u8_vec(&data.get_endian())
				},

				TagType::DATA_OFFSET(_) => {
					match tag
					{
						ExifTag::StripOffsets(_, strip_data) => {
							let mut value = Vec::new();
							for strip in strip_data
							{
								// Store the current offset where the strip is
								// pushed, push the strip and account for its length
								// in the offset variable
								value.extend(
									to_u8_vec_macro!(u32, &current_offset.clone(), &data.get_endian())
								);
								ifd_offset_area.extend(strip);
								*current_offset += strip.len() as u32;
							}
							value
						},
		
						ExifTag::ThumbnailOffset(_, thumbnail_data) => {
							let value = to_u8_vec_macro!(u32, &current_offset.clone(), &data.get_endian());
							ifd_offset_area.extend(thumbnail_data);
							*current_offset += thumbnail_data.len() as u32;
							value
						},

						_ => tag.value_as_u8_vec(&data.get_endian()),
					}
				}

				TagType::IFD_OFFSET(_) => {

					if let Some(group) = Self::get_ifd_type_for_offset_tag(tag)
					{
						// Find that IFD in the parent struct and encode that
						if let Ok((_, subifd_offset)) = data.get_ifds()
							.iter()
							.filter(|ifd| 
								ifd.get_generic_ifd_nr() == self.get_generic_ifd_nr() &&
								ifd.get_ifd_type()       == group
							)
							.next().unwrap().encode_ifd(
								data, 
								ifds_with_offset_info_only, 
								&mut ifd_offset_area, 
								current_offset
							)
						{
							subifd_offset
						}
						else
						{
							panic!("Could not find IFD in parent struct!");
						}
					}
					else
					{
						panic!("Could not determine type of SubIFD!");
					}
				}
			};

			// Re-align 
			let alignment_count = (4 - *current_offset % 4) % 4;
			*current_offset += alignment_count;
			ifd_offset_area.extend(vec![0u8; alignment_count as usize]);
			
			
			// Add Tag & Data Format /                                          2 + 2 bytes
			encode_vec.extend(to_u8_vec_macro!(u16, &tag.as_u16(),          &data.get_endian()).iter());
			encode_vec.extend(to_u8_vec_macro!(u16, &tag.format().as_u16(), &data.get_endian()).iter());

			// Add number of components /                                       4 bytes
			let number_of_components: u32 = tag.number_of_components();
			encode_vec.extend(to_u8_vec_macro!(u32, &number_of_components, &data.get_endian()).iter());

			// Optional string padding (i.e. string is shorter than it should be)
			let mut string_padding: Vec<u8> = Vec::new();
			if tag.is_string()
			{
				for _ in 0..(number_of_components - value.len() as u32)
				{
					string_padding.push(0x00);
				}
			}

			// Add offset or value /                                            4 bytes
			// Depending on the amount of data, either put it directly into
			// next 4 bytes or write an offset where the data can be found 
			let byte_count: u32 = number_of_components * tag.format().bytes_per_component();
			if byte_count > 4
			{
				encode_vec.extend(to_u8_vec_macro!(u32, current_offset, &data.get_endian()).iter());
				ifd_offset_area.extend(value.iter());
				ifd_offset_area.extend(string_padding.iter());

				*current_offset += byte_count;

				// Re-align 
				let alignment_count = (4 - *current_offset % 4) % 4;
				*current_offset += alignment_count;
				ifd_offset_area.extend(vec![0u8; alignment_count as usize]);
			}
			else
			{
				let pre_length = encode_vec.len();

				encode_vec.extend(value.iter());
				encode_vec.extend(string_padding.iter());

				let post_length = encode_vec.len();

				// Make sure that this area is indeed *exactly* 4 bytes long
				for _ in 0..(4-(post_length - pre_length) ) {
					encode_vec.push(0x00);
				}
			}
		}

		// Write link and offset data
		encode_vec.extend(IFD_END_NO_LINK.iter());
		encode_vec.extend(ifd_offset_area.iter());

		return Ok(((ifd_offset + 2 + IFD_ENTRY_LENGTH * count_entries as u32) as u64, ifd_offset_vec));
	}
}
