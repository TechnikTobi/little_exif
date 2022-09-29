use std::path::Path;
use std::collections::VecDeque;

use crate::endian::{Endian, U8conversion};
use crate::exif_tag::{ExifTag, ExifTagGroup};
use crate::general_file_io::*;

use crate::jpg;
use crate::png;

const NEWLINE: u8 = 0x0a;
const SPACE: u8 = 0x20;

const EXIF_HEADER: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];
const TIFF_HEADER_LITTLE: [u8; 8] = [0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00];
const TIFF_HEADER_BIG: [u8; 8] = [0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x08];

const JPG_APP1_MARKER: [u8; 2] = [0xff, 0xe1];

const IFD_ENTRY_LENGTH: u32 = 12;
const IFD_END: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

macro_rules! to_u8_vec_macro {
	($type:ty, $value:expr, $endian:expr)
	=>
	{
		<$type as U8conversion>::to_u8_vec($value, $endian)
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
		/*
		for value in &Self::decode_metadata_png(path)
		{
			println!("{}", value);
		}
		*/
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
			if a.get_group() == b.get_group() 
			{
				// Same group, but unknown should go last 
				if a.is_unknown() == b.is_unknown()
				{
					std::cmp::Ordering::Equal
				}
				else if !a.is_unknown() && b.is_unknown()
				{
					std::cmp::Ordering::Less
				}
				else
				{
					std::cmp::Ordering::Greater
				}
				
			}
			else
			{
				if a.get_group() < b.get_group() // e.g. IFD0 < ExifIFD
				{
					std::cmp::Ordering::Less
				}
				else
				{
					std::cmp::Ordering::Greater
				}
			}
		);

		println!("Output after set_tag");
		for value in &self.data
		{
			println!("{:?} {}", value.get_group(), value.is_unknown());
		}
	}

	pub fn
	write_to_file
	(
		&self,
		path: &Path
	)
	-> Result<(), std::io::Error>
	{
		if !path.exists()
		{
			return io_error!(Other, "Can't write Metadata - File does not exist!");
		}

		let file_type = path.extension();
		if file_type.is_none()
		{
			return io_error!(Other, "Can't get extension from given path!");
		}

		let file_type_str = file_type.unwrap().to_str();
		if file_type_str.is_none()
		{
			return io_error!(Other, "Can't convert file type to string!");
		}
		
		match file_type_str.unwrap().to_lowercase().as_str()
		{
			"jpg"	=> jpg::write_metadata(&path, &self.encode_metadata_jpg()),
			"jpeg"	=> jpg::write_metadata(&path, &self.encode_metadata_jpg()),
			"png"	=> png::write_metadata(&path, &self.encode_metadata_png()),
			_		=> io_error!(Unsupported, "Unsupported file type!"),
		}

	}


	pub fn
	decode_metadata_png
	(
		encoded_data: &Vec<u8>
	)
	-> Vec<u8>
	{

		let mut exif_all: VecDeque<u8> = VecDeque::new();
		let mut other_byte: Option<u8> = None;

		// This performs the reverse operation to encode_byte:
		// Two successing bytes represent the ASCII values of the digits of 
		// a hex value, e.g. 0x31, 0x32 represent '1' and '2', so the resulting
		// hex value is 0x12, which gets pushed onto exif_all
		for byte in encoded_data
		{
			// Ignore newline characters
			if *byte == '\n' as u8
			{
				continue;
			}

			if other_byte.is_none()
			{
				other_byte = Some(*byte);
				continue;
			}

			let value_string = "".to_owned()
				+ &(other_byte.unwrap() as char).to_string()
				+ &(*byte as char).to_string();
			if let Ok(value) = u8::from_str_radix(value_string.trim(), 16)
			{
				exif_all.push_back(value);
			}
			
			other_byte = None;
		}

		// Now remove the first element until the exif header is found
		// Store the popped elements to get the size information
		let mut exif_header_found = false;
		let mut pop_storage: Vec<u8> = Vec::new();

		while !exif_header_found
		{
			let mut counter = 0;
			for header_value in &EXIF_HEADER
			{
				if *header_value != exif_all[counter]
				{
					break;
				}
				counter += 1;
			}

			exif_header_found = counter == EXIF_HEADER.len();

			if exif_header_found
			{
				break;
			}
			pop_storage.push(exif_all.pop_front().unwrap());
		}

		// The exif header has been found
		// -> exif_all now starts with the exif header information
		// -> pop_storage has in its last 4 elements the size information
		//    that will now get extracted
		// Consider this part optional as it might be removed in the future and
		// isn't strictly necessary and just for validating the data we get
		assert!(pop_storage.len() > 0);

		// Using the encode_byte function re-encode the bytes regarding the size
		// information and construct its value using decimal based shifting
		// Example: 153 = 0
		// + 5*10*10^(2*0) + 3*1*10^(2*0) 
		// + 0*10*10^(2*1) + 1*1*10^(2*1)
		let mut given_exif_len = 0u64;
		for i in 0..std::cmp::min(4, pop_storage.len())
		{
			let re_encoded_byte = Self::encode_byte(&pop_storage[pop_storage.len() -1 -i]);
			let tens_place = u64::from_str_radix(&(re_encoded_byte[0] as char).to_string(), 10).unwrap();
			let ones_place = u64::from_str_radix(&(re_encoded_byte[1] as char).to_string(), 10).unwrap();
			given_exif_len = given_exif_len + tens_place * 10 * 10_u64.pow((2 * i).try_into().unwrap());
			given_exif_len = given_exif_len + ones_place *  1 * 10_u64.pow((2 * i).try_into().unwrap());
		}

		assert!(given_exif_len == exif_all.len().try_into().unwrap());
		// End optional part

		return Vec::from(exif_all);
	}

	pub fn
	decode_metadata_general
	(
		encoded_data: &Vec<u8>
	)
	-> Result<Vec<u8>, std::io::Error>
	{

		// Ensure that we have enough data
		if encoded_data.len() < (EXIF_HEADER.len() + TIFF_HEADER_BIG.len() + 2 + IFD_END.len())
		{
			return io_error!(Other, "Not enough data for encoding!");
		}

		// Validate EXIF header
		for i in 0..EXIF_HEADER.len()
		{
			if encoded_data[i] != EXIF_HEADER[i]
			{
				return io_error!(Other, "Could not validate EXIF header!");
			}
		}

		// Determine endian
		let endian;
		if encoded_data[6] == 0x49 && encoded_data[7] == 0x49
		{
			endian = Endian::Little;
		}
		else if encoded_data[6] == 0x4d && encoded_data[7] == 0x4d
		{
			endian = Endian::Big;
		}
		else
		{
			return io_error!(Other, "Illegal endian information!");
		}



		Ok(Vec::new())
	}
	

	fn
	decode_ifd
	(
		encoded_data: &Vec<u8>,
		group: ExifTagGroup,
	)
	-> Result<Vec<ExifTag>, std::io::Error>
	{
		Ok(Vec::new())
	}

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
		let mut count_entries = subifd_tag.is_some() as u16;
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

			let value = tag.value_as_u8_vec(&self.endian);
			
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
			Endian::Little	=> exif_vec.extend(TIFF_HEADER_LITTLE.iter()),
			Endian::Big		=> exif_vec.extend(TIFF_HEADER_BIG.iter())
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
		let (offset_post_exififd, exififd_data) = self.encode_ifd(
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

	pub fn
	encode_metadata_png
	(
		&self
	)
	-> Vec<u8>
	{
		// Get the general version of the encoded metadata
		let exif_vec = self.encode_metadata_general();

		// The size of the EXIF data area, consists of
		// - length of EXIF header (follows after ssss)
		// - length of exif_vec
		// - 1 for ssss itself (why not 4? idk)
		let ssss = (
			EXIF_HEADER.len()	as u32 
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
		for byte in &EXIF_HEADER
		{
			png_exif.extend(Self::encode_byte(byte).iter());
		}

		for byte in &exif_vec
		{
			png_exif.extend(Self::encode_byte(byte).iter());
		}
		
		// Write end of EXIF data - 2* 0x30 results in the String "00" for 0x00
		png_exif.push(0x30);
		png_exif.push(0x30);
		png_exif.push(NEWLINE);

		return png_exif;
	}

	fn
	encode_metadata_jpg
	(
		&self
	)
	-> Vec<u8>
	{
		// Get the general version of the encoded metadata
		let exif_vec = self.encode_metadata_general();

		// vector storing the data that will be returned
		let mut jpg_exif: Vec<u8> = Vec::new();

		// Compute the length of the exif data (includes the two bytes of the
		// actual length field)
		let length = 2u16 + (exif_vec.len() as u16);

		// Start with the APP1 marker and the length of the data
		// Then copy the previously encoded EXIF data 
		jpg_exif.extend(JPG_APP1_MARKER.iter());
		jpg_exif.extend(to_u8_vec_macro!(u16, &length, &Endian::Big));
		jpg_exif.extend(exif_vec.iter());

		return jpg_exif;
	}
}
