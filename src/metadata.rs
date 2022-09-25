use std::path::Path;

use crate::endian::{Endian, U8conversion};
use crate::exif_tag::{ExifTag, ExifTagGroup};
use crate::png::write_metadata_to_png;

const NEWLINE: u8 = 0x0a;
const SPACE: u8 = 0x20;

const EXIF_header: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];
const TIFF_header_little: [u8; 8] = [0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00];
const TIFF_header_big: [u8; 8] = [0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x08];

const IFD_ENTRY_LENGTH: u32 = 12;
const IFD_END: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

macro_rules! to_u8_vec_macro {
	($type:ty, $value:expr, $endian:expr)
	=>
	{
		<$type as U8conversion<$type>>::to_u8_vec($value, $endian)
	};
}

pub struct
Metadata
{
	data: Vec<ExifTag>,
	endian: Endian 
}

impl
Metadata
{
	pub fn
	new
	()
	-> Metadata
	{
		Metadata { endian: Endian::Little, data: Vec::new() }
	}


	pub fn
	new_from_path
	(
		path: &Path
	)
	-> Metadata
	{
		Metadata { endian: Endian::Little, data: Vec::new() }
	}
	

	pub fn
	get_tag
	(
		&self,
		input_tag: &ExifTag
	)
	-> Option<&ExifTag> 
	{
		for tag in &self.data
		{
			if tag.as_u16() == input_tag.as_u16()
			{
				return Some(tag);
			}
		}
		return None;
	}


	pub fn
	set_tag
	(
		&mut self,
		input_tag: ExifTag,
	)
	{
		self.data.retain(|tag| tag.as_u16() != input_tag.as_u16());
		self.data.push(input_tag);

		// Sort the tags by the IFD they will go into the file later on
		self.data.sort_by(
			|a, b| 
			if b.get_group() == a.get_group() 
			{
				std::cmp::Ordering::Equal
			}
			else
			{
				if b.get_group() < a.get_group()
				{
					std::cmp::Ordering::Greater
				}
				else
				{
					std::cmp::Ordering::Less
				}
			}
		);

		println!("Output after set_tag");
		for value in &self.data
		{
			println!("{:?}", value.get_group());
		}
	}


	pub fn
	write_to_file
	(
		&self,
		path: &Path
	)
	-> Result<(), String>
	{
		if !path.exists()
		{
			return Err("Can't write Metadata - File does not exist!".to_string());
		}

		let file_type = path.extension();
		if file_type.is_none()
		{
			return Err("Can't get extension from given path!".to_string());
		}

		let file_type_str = file_type.unwrap().to_str();
		if file_type_str.is_none()
		{
			return Err("Can't convert file type to string!".to_string());
		}
		
		match file_type_str.unwrap().to_lowercase().as_str()
		{
			"png"	=> write_metadata_to_png(&path, &self.encode_metadata_png()),
			_		=> Err("Unsupported file type!".to_string()),
		}

	}

	/*
	fn
	decode
	(
		encoded_data: &Vec<u8>
	)
	->
	Metadata
	{

		let mut exif_all = Vec::new();
		let mut other_byte: Option<u8> = None;

		for byte in &encoded_data
		{
			if other_byte.is_none()
			{
				other_byte = Some(byte);
				continue;
			}
			


			other_byte = None;
		}
	}
	*/


	fn
	encode_ifd
	(
		&self,																	// The metadata struct, containing the tags
		group: ExifTagGroup,													// The group the specific tags need to belong to (e.g. IFD0, ExifIFD, ...)
		given_offset: u32,														// How much offset already exists
		next_ifd_link: &[u8; 4],												// A link to the next IFD (e.g. IFD1 for IFD0) or 4 bytes of 0x00 to signal "no next IFD"
		subifd_tag: Option<ExifTag>												// An optional ExifTag signaling that a SubIFD will follow
	)
	-> Option<(u32, Vec<u8>)>
	{
		// Start Interop IFD with number of entries
		// If there are none, return None
		let mut ifd_vec: Vec<u8> = Vec::new();
		let mut count_entries = (subifd_tag.is_some() as u16);
		for tag in &self.data
		{
			if tag.is_writable() && tag.get_group() == group
			{
				count_entries += 1;
			}
		}

		if count_entries == 0
		{
			return None;
		}

		// Start by adding the number of entries
		ifd_vec.extend(to_u8_vec_macro!(u16, &count_entries, &self.endian).iter());
		assert_eq!(ifd_vec.len(), 2);

		// Compute first offset value and provide offset area in case its needed
		let mut next_offset: u32 = 0						as u32
		+ given_offset										as u32
		+ ifd_vec.len()										as u32
		+ IFD_ENTRY_LENGTH * count_entries 					as u32
		+ next_ifd_link.len()								as u32;
		let mut ifd_offset_area: Vec<u8> = Vec::new();

		// Write directory entries to the vector
		for tag in &self.data
		{
			// Skip tags that can't be written or don't belong to the group
			if !tag.is_writable() || tag.get_group() != group
			{
				continue;
			}

			if let value = tag.value_as_u8_vec(&self.endian)
			{
				// Add Tag & Data Format /										2 + 2 bytes
				ifd_vec.extend(to_u8_vec_macro!(u16, &tag.as_u16(), &self.endian).iter());
				ifd_vec.extend(to_u8_vec_macro!(u16, &tag.format().as_u16(), &self.endian).iter());

				// Add number of components /									4 bytes
				let number_of_components: u32 = tag.number_of_components();
				ifd_vec.extend(to_u8_vec_macro!(u32, &number_of_components, &self.endian).iter());

				// Optional string padding (i.e. string is shorter than it should be)
				let mut string_padding: Vec<u8> = Vec::new();
				if tag.is_string()
				{
					for _ in 0..(number_of_components - value.len() as u32)
					{
						string_padding.push(0x00);
					}	
				}

				// Add offset or value /										4 bytes
				// Depending on the amount of data, either put it directly into
				// next 4 bytes or write an offset where the data can be found 
				let byte_count: u32 = number_of_components * tag.format().bytes_per_component();
				if byte_count > 4
				{
					ifd_vec.extend(to_u8_vec_macro!(u32, &next_offset, &self.endian).iter());
					ifd_offset_area.extend(value.iter());
					ifd_offset_area.extend(string_padding.iter());

					next_offset += byte_count;
				}
				else
				{
					let pre_length = ifd_vec.len();

					ifd_vec.extend(value.iter());
					ifd_vec.extend(string_padding.iter());

					let post_length = ifd_vec.len();

					// Make sure that this area is indeed *exactly* 4 bytes long
					for _ in 0..(4-(post_length - pre_length) ) {
						ifd_vec.push(0x00);
					}
				}
			}
			else
			{
				println!("Can't unpack value from Tag!");
			}
		}

		// In case we have to write a SubIFD (e.g. ExifIFD) next
		// Do NOT mix this up with link to next IFD (like e.g. IFD1)
		if let Some(tag) = subifd_tag
		{
			// Write the offset tag & data format /								2 + 2 bytes
			ifd_vec.extend(to_u8_vec_macro!(u16, &tag.as_u16(), &self.endian).iter());
			ifd_vec.extend(to_u8_vec_macro!(u16, &tag.format().as_u16(), &self.endian).iter());

			// Add number of components /										4 bytes
			ifd_vec.extend(to_u8_vec_macro!(u32, &tag.number_of_components(), &self.endian).iter());

			// Add the offset /													4 bytes
			// We assume (know) that this is one component which has exactly
			// 4 bytes, thus fitting perfectly into the directory entry
			ifd_vec.extend(to_u8_vec_macro!(u32, &next_offset, &self.endian).iter());
		}

		// Write link and offset data
		ifd_vec.extend(next_ifd_link.iter());
		ifd_vec.extend(ifd_offset_area.iter());

		// Return next_offset as well to where to start with the offset
		// in the subordinate IFDs
		return Some((next_offset, ifd_vec));
	}

	fn
	encode_metadata_general
	(
		&self
	)
	-> Vec<u8>
	{
		// Start construction with TIFF header
		let mut exif_vec: Vec<u8> = Vec::new();
		match self.endian {
			Endian::Little	=> exif_vec.extend(TIFF_header_little.iter()),
			Endian::Big		=> exif_vec.extend(TIFF_header_big.iter())
		}

		// IFD0
		let (offset_post_ifd0, ifd0_data) = self.encode_ifd(
			ExifTagGroup::IFD0,
			8,																	// For the TIFF header
			&[0x00, 0x00, 0x00, 0x00],											// For now no link to IFD1
			Some(ExifTag::ExifOffset(vec![0]))
		).unwrap();
		exif_vec.extend(ifd0_data.iter());

		// ExifIFD
		let exif(offset_post_exififd, exififd_data)ifd_result = self.encode_ifd(
			ExifTagGroup::ExifIFD,
			offset_post_ifd0,													// Don't need +8 as already accounted for in this value due to previous function call
			&[0x00, 0x00, 0x00, 0x00],
			None
		).unwrap();
		exif_vec.extend(exififd_data.iter());

		// Other directories here... (someday)
		
		let mut counter = 0u32;
		for byte in &exif_vec
		{
			if *byte > 0x20
			{
				println!("{:#04x} {:#04x} {}", counter, *byte, *byte as char);
			}
			else
			{
				println!("{:#04x} {:#04x}", counter, *byte);
			}
			
			counter += 1;
		}
		
		return exif_vec;
	}

	// The bytes during encoding need to be encoded themselves:
	// A given byte (e.g. 0x30 for the char '0') has two values in the string of its hex representation ('3' and '0')
	// These two characters need to be encoded themselves (51 for '3', 48 for '0'), resulting in the final encoded
	// version of the EXIF data
	// Independent of endian as this does not affect the ordering of values WITHIN a byte 
	fn encode_byte(byte: &u8) -> [u8; 2] 
	{
		[
			byte / 16 + (if byte / 16 < 10 {'0' as u8} else {'a' as u8 - 10}),
			byte % 16 + (if byte % 16 < 10 {'0' as u8} else {'a' as u8 - 10}) 
		]
	}

	fn
	encode_metadata_png
	(
		&self
	)
	-> Vec<u8>
	{
		// IFD0/PNG specific stuff

		let exif_vec = self.encode_metadata_general();

		// The size of the EXIF data area, consists of
		// - length of EXIF header (follows after ssss)
		// - length of exif_vec
		// - 1 for ssss itself (why not 4? idk)
		let ssss = (
			EXIF_header.len()	as u32 
			+ exif_vec.len()	as u32 
			+ 1					as u32
		).to_string();

		// Construct final vector with the bytes as they will be sent to the encoder
		//                               \n       e     x     i     f     \n
		let mut png_exif: Vec<u8> = vec![NEWLINE, 0x65, 0x78, 0x69, 0x66, NEWLINE];

		// Write ssss
		for _ in 0..(8-ssss.len())
		{
			png_exif.push(SPACE);
		}
		png_exif.extend(ssss.as_bytes().to_vec().iter());
		png_exif.push(NEWLINE);

		// Write EXIF header and previously constructed EXIF data as encoded bytes
		for byte in &EXIF_header
		{
			png_exif.extend(Self::encode_byte(byte).iter());
		}

		for byte in &exif_vec
		{
			png_exif.extend(Self::encode_byte(byte).iter());
		}
		
		// Write end of EXIF data
		png_exif.push(0x30);
		png_exif.push(0x30);
		png_exif.push(NEWLINE);

		return png_exif;
	}
}
