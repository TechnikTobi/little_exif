use std::path::Path;

use crate::endian::{Endian, U8conversion};
use crate::exif_tag::ExifTag;

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
		Metadata { endian: Endian::Big, data: Vec::new() }
	}


	pub fn
	new_from_path
	(
		path: &Path
	)
	-> Metadata
	{
		Metadata { endian: Endian::Big, data: Vec::new() }
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
	}


	pub fn
	write_to_file
	(
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
			"png"	=> Ok(()),
			_		=> Err("Unsupported file type!".to_string()),
		}

	}

	fn
	encode
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
		
		// Number of IFD entries
		// Note: Currently everything will be written into IFD0
		//       as I don't yet understand when to use other IFDs
		exif_vec.extend(to_u8_vec_macro!(u16, &(self.data.len() as u16), &self.endian).iter());

		assert!(exif_vec.len() == 10);

		// Compute what the first offset value will be in case we need that
		// Also provide vec for actual data stored in offset area
		let mut next_offset: u32 = 0			as u32
		+ exif_vec.len()						as u32
		+ IFD_ENTRY_LENGTH * (self.data.len() 	as u32) 
		+ IFD_END.len() 						as u32;
		let mut exif_offset_area: Vec<u8> = Vec::new();

		// Write IFD entries
		for tag in &self.data
		{

			// Skip tags that can't be written
			if !tag.is_writable()
			{
				continue;
			}

			if let value = tag.value_as_u8_vec(&self.endian)
			{

				// Add Tag & Data Format /		2 + 2 bytes
				exif_vec.extend(to_u8_vec_macro!(u16, &tag.as_u16(), &self.endian).iter());
				exif_vec.extend(to_u8_vec_macro!(u16, &tag.format().as_u16(), &self.endian).iter());

				// Add number of components /	4 bytes
				let number_of_components: u32 = tag.number_of_components();
				let byte_count: u32 = number_of_components * tag.format().bytes_per_component();
				exif_vec.extend(to_u8_vec_macro!(u32, &number_of_components, &self.endian).iter());

				// Optional string padding (i.e. string is shorter than it should be)
				let mut string_padding: Vec<u8> = Vec::new();
				if tag.is_string()
				{
					for _ in 0..(number_of_components - value.len() as u32)
					{
						string_padding.push(0x00);
					}	
				}

				// Add offset or value /		4 bytes
				// Depending on the amount of data, either put it directly into the
				// next 4 bytes or write an offset where the data can be found 
				if byte_count > 4
				{
					exif_vec.extend(to_u8_vec_macro!(u32, &next_offset, &self.endian).iter());
					exif_offset_area.extend(value.iter());
					exif_offset_area.extend(string_padding.iter());

					next_offset += byte_count;
				}
				else
				{
					let pre_length = exif_vec.len();

					exif_vec.extend(value.iter());
					exif_vec.extend(string_padding.iter());

					let post_length = exif_vec.len();

					// Make sure that this area is indeed *exactly* 4 bytes long
					for _ in 0..(4-(post_length - pre_length) ) {
						exif_vec.push(0x00);
					}
				}
			}
			else
			{
				println!("Can't unpack value from ExifTag!");
			}
		}

		// Write end and offset data
		exif_vec.extend(IFD_END.iter());
		exif_vec.extend(exif_offset_area.iter());

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
		//                               \n       e     x     i     f
		let mut exif_all: Vec<u8> = vec![NEWLINE, 0x65, 0x78, 0x69, 0x66, NEWLINE];

		// Write ssss
		for _ in 0..(8-ssss.len())
		{
			exif_all.push(SPACE);
		}
		exif_all.extend(ssss.as_bytes().to_vec().iter());
		exif_all.push(NEWLINE);

		// Write EXIF header and previously constructed EXIF data
		for byte in &EXIF_header
		{
			exif_all.extend(Self::encode_byte(byte).iter());
		}

		for byte in &exif_vec
		{
			exif_all.extend(Self::encode_byte(byte).iter());
		}
		
		// Write end of EXIF data
		exif_all.push(0x00);
		exif_all.push(0x00);
		exif_all.push(NEWLINE);

		return exif_all;
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
}
