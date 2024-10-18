// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::vec;

use crate::endian::*;
use crate::exif_tag::ExifTag;
use crate::exif_tag::TagType;
use crate::exif_tag_format::ExifTagFormat;
use crate::exif_tag_format::INT16U;
use crate::general_file_io::io_error;
use crate::tiff;
use crate::tiffdata::Tiffdata;
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
	GPS,
}

/*

LEGACY VERSION

#[allow(non_camel_case_types)]
#[derive(Debug, Eq, PartialEq, PartialOrd, Hash, Clone, Copy)]
pub enum
ExifTagGroup
{
	IFD0,
		ExifIFD,
			InteropIFD,
			MakerNotesIFD,
		GPSIFD,
	IFD1,
	Other,
}

*/


/// The value of `belongs_to_generic_ifd_nr` tells us what generic IFD this
/// specific IFD belongs to, e.g. `0` would indicate that it belongs (or is)
/// IFD0. 
#[derive(Clone, Debug)]
pub struct
ImageFileDirectory
{
	pub tags:                  Vec<ExifTag>,
	ifd_type:                  ExifTagGroup,
	belongs_to_generic_ifd_nr: u32,
}

impl
ImageFileDirectory
{
	pub fn
	get_generic_ifd_nr
	(
		&self
	)
	-> u32
	{
		return self.belongs_to_generic_ifd_nr;
	}

	pub fn
	get_ifd_type
	(
		&self
	)
	-> ExifTagGroup
	{
		return self.ifd_type;
	}

	pub fn
	get_offset_tag_for_parent_ifd
	(
		&self
	)
	-> Option<(ExifTagGroup, ExifTag)>
	{
		match self.ifd_type
		{
			ExifTagGroup::GENERIC  => None,
			ExifTagGroup::EXIF     => Some((ExifTagGroup::GENERIC, ExifTag::ExifOffset(Vec::new()))),
			ExifTagGroup::GPS      => Some((ExifTagGroup::GENERIC, ExifTag::GPSInfo(   Vec::new()))),
			ExifTagGroup::INTEROP  => panic!("INTEROP NOT YET SUPPORT - PLEASE CONTACT THE LITTLE_EXIF DEVELOPER!"),
		}
	}

	pub fn
	get_ifd_type_for_offset_tag
	(
		tag: &ExifTag
	)
	-> Option<ExifTagGroup>
	{
		match tag
		{
			ExifTag::ExifOffset(_) => Some(ExifTagGroup::EXIF),
			ExifTag::GPSInfo(_)    => Some(ExifTagGroup::GPS),
			_ => None
		}
	}

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

	/// Add a tag to this IFD. Sorts all its tags after insert.
	pub fn
	add_tag
	(
		&mut self,
		tag: ExifTag
	)
	{
		self.tags.push(tag);
		self.sort_tags();
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
		if (0
			+ 2
			+ IFD_ENTRY_LENGTH as usize * number_of_entries as usize 
			+ IFD_END_NO_LINK.len()
		) > (
			data_cursor.get_ref().len() as i64 - data_cursor_entry_position as i64
		) as usize
		{
			return io_error!(Other, "Not enough data to decode IFD!");
		}

		// Temporarily storing specific tags that have been decoded
		// This has to do with data offset tags that are interconnected with
		// other tags.
		// For example, for decoding the StripOffsets we also need the 
		// StripByteCounts to know how many bytes each strip has
		let mut strip_tags: (Option<ExifTag>, Option<ExifTag>) = (None, None);
		// Others following here in the future...

		////////////////////////////////////////////////////////////////////////
		// TAG-DECODING

		// Storing all tags while decoding
		let mut tags = Vec::new();

		// loop through the entries - assumes that the value stored in
		// `number_of_entries` is correct
		for i in 0..number_of_entries
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
				data_cursor.seek_relative(hex_offset as i64)?;

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
				data_cursor.seek_relative(offset as i64);

				let subifd_decode_result = Self::decode_ifd(
					data_cursor,
					data_begin_position,
					endian,
					&subifd_group,
					generic_ifd_nr,
					insert_into,
				);

				// Check that this actually worked
				if let Ok(subifd_result) = subifd_decode_result
				{
					// Assert result, restore old cursor position & continue
					assert_eq!(subifd_result, None);
					data_cursor.set_position(backup_position);
					continue;
				}
				else
				{
					return io_error!(Other, format!("Could not decode SubIFD:\n  {}", subifd_decode_result.err().unwrap()));
				}
			}

			// At this point we check if the format is actually what we expect
			// it to be and convert it if possible
			if tag.format().as_u16() != format.as_u16()
			{
				// The expected format and the given format in the file
				// do *not* match. Check special cases (INT16U -> INT32U)
				// If no special cases match, return an error
				if 
					tag.format() == ExifTagFormat::INT32U &&
					format       == ExifTagFormat::INT16U
				{
					let int16u_data = <INT16U as U8conversion<INT16U>>::from_u8_vec(&raw_data, endian);
					let int32u_data = int16u_data.into_iter().map(|x| x as u32).collect::<Vec<u32>>();

					tag = tag.set_value_to_int32u_vec(int32u_data).unwrap();
				}
				// Other special cases
				else
				{
					return io_error!(Other, format!("Illegal format for known tag! Tag: {:?} Expected: {:?} Got: {:?}", tag, tag.format(), format));
				}
			}
			else
			{
				// Format is as expected; set the data by replacing the tag
				tag = ExifTag::from_u16_with_data(
					hex_tag, 
					&format, 
					&raw_data, 
					&endian, 
					group
				).unwrap();
			}

			// Now we have at least confirmed that the format is ok (or has
			// been corrected). Next, we need to differ between the two other
			// tag types:
			if let TagType::DATA_OFFSET(_) = tag.get_tag_type()
			{
				match tag
				{
					ExifTag::StripOffsets(_, _) => {
						strip_tags.0 = Some(tag)
					},
					ExifTag::StripByteCounts(_, _) => {
						strip_tags.1 = Some(tag)
					}
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
					data_cursor.seek_relative(*offset as i64)?;

					let mut data_buffer = vec![0u8; *byte_count as usize];
					data_cursor.read_exact(&mut data_buffer)?;
					strip_data.push(data_buffer);
				}

				// Push StipOffset tag to tags vector
				tags.push(ExifTag::StripOffsets(Vec::new(), strip_data));

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
		data_cursor.read_exact(&mut next_ifd_link_buffer)?;

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

	pub(crate) fn
	encode_ifd
	(
		&self,
		tiffdata:                   &Tiffdata,
		ifds_with_offset_info_only: &mut Vec<ImageFileDirectory>,
		encode_vec:                 &mut Vec<u8>,
		current_offset:             &mut u32
	)
	-> Result<u32, std::io::Error>
	{
		// Get the offset information for this IFD's SubIFDs
		let ifd_with_offset_info_only = ifds_with_offset_info_only
			.iter()
			.filter(|ifd| 
				ifd.get_generic_ifd_nr() == self.get_generic_ifd_nr() &&
				ifd.get_ifd_type()       == self.get_ifd_type()
			)
			.next().unwrap();

		// Check if this IFD links to a SubIFD. If so, encode that one first
		for offset_tag in &ifd_with_offset_info_only.tags.clone()
		{
			if let Some(group) = Self::get_ifd_type_for_offset_tag(offset_tag)
			{
				// Find that IFD in the parent struct and encode that
				if let Ok(subifd_offset) = tiffdata.get_ifds()
					.iter()
					.filter(|ifd| 
						ifd.get_generic_ifd_nr() == self.get_generic_ifd_nr() &&
						ifd.get_ifd_type()       == group
					)
					.next().unwrap().encode_ifd(
						tiffdata, 
						ifds_with_offset_info_only, 
						encode_vec, 
						current_offset
					)
				{
					// Update the offset tag for later
					&ifds_with_offset_info_only
						.iter_mut()
						.filter(|ifd| 
							ifd.get_generic_ifd_nr() == self.get_generic_ifd_nr() &&
							ifd.get_ifd_type()       == self.get_ifd_type()
						)
						.next().unwrap().tags
						.iter_mut()
						.filter(|tag| tag.as_u16() == offset_tag.as_u16())
						.for_each(|tag| { *tag = ExifTag::from_u16_with_data(
							tag.as_u16(), 
							&tag.format(), 
							&to_u8_vec_macro!(u32, &subifd_offset, &tiffdata.get_endian()), 
							&tiffdata.get_endian(), 
							&tag.get_group()
						).unwrap()});
				}
			}
		}

		

		// SubIFDs are done; Now we need to handle data areas that are 
		// described by data offset tags, such as StripOffsets
		// As we can't modify the tags directly, store their relevant data
		// that results from these write operations in new vectors
		let mut new_StripOffsets = Vec::new();
		// let mut new_TODO ...

		for tag in &self.tags
		{
			if let TagType::DATA_OFFSET(_) = tag.get_tag_type()
			{
				match tag
				{
					ExifTag::StripOffsets(_, strip_data) => {
						for strip in strip_data
						{
							// Store the current offset where the strip is
							// pushed, push the strip and account for its length
							// in the offset variable
							new_StripOffsets.extend(
								to_u8_vec_macro!(u32, &current_offset.clone(), &tiffdata.get_endian())
							);
							encode_vec.extend(strip);
							*current_offset += strip.len() as u32;
						}
					},
					// TODO: What other tags to put in here?!
					_ => ()
				}
			}
		}

		// Now we can finally start by writing this IFD!
		// Start by adding the number of entries
		let count_entries = self.tags.iter().filter(
			|tag| tag.is_writable() || 
			if let TagType::IFD_OFFSET(_) = tag.get_tag_type() { true } else { false } 
		).count() as u16;
		encode_vec.extend(to_u8_vec_macro!(u16, &count_entries, &tiffdata.get_endian()).iter());

		// Advance offset address to the point after the entries and provide
		// offset area vector
		*current_offset += 0
			+ 2                                                                 // length of entry count section
			+ IFD_ENTRY_LENGTH * count_entries as u32
			+ IFD_END_NO_LINK.len()            as u32
		;
		let mut ifd_offset_area: Vec<u8> = Vec::new();

		// Write directory entries to the vector
		for tag in &self.tags
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
			let value = if let ExifTag::StripOffsets(_, _) = tag
			{
				&new_StripOffsets
			}
			else
			{
				&tag.value_as_u8_vec(&tiffdata.get_endian())
			};
			
			// Add Tag & Data Format /                                          2 + 2 bytes
			encode_vec.extend(to_u8_vec_macro!(u16, &tag.as_u16(),          &tiffdata.get_endian()).iter());
			encode_vec.extend(to_u8_vec_macro!(u16, &tag.format().as_u16(), &tiffdata.get_endian()).iter());

			// Add number of components /                                       4 bytes
			let number_of_components: u32 = tag.number_of_components();
			encode_vec.extend(to_u8_vec_macro!(u32, &number_of_components, &tiffdata.get_endian()).iter());

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
				encode_vec.extend(to_u8_vec_macro!(u32, current_offset, &tiffdata.get_endian()).iter());
				ifd_offset_area.extend(value.iter());
				ifd_offset_area.extend(string_padding.iter());

				*current_offset += byte_count;
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

		
		todo!()
	}
}