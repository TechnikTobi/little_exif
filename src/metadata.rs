use std::collections::HashMap;
use std::path::Path;

use crate::endian::{Endian, U8conversion};
use crate::exif_tag::ExifTag;
use crate::exif_tag_value::ExifTagValue;

const NEWLINE: u8 = 0x0a;
const SPACE: u8 = 0x20;

const EXIF_header: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];
const TIFF_header_little: [u8; 8] = [0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00];
const TIFF_header_big: [u8; 8] = [0x4d, 0x4d, 0x00, 0x2a, 0x00, 0x00, 0x00, 0x08];

const IFD_ENTRY_LENGTH: u32 = 12;
const IFD_END: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

const SUPPORTED_FILE_TYPES: [&'static str; 1] = [
	"png"
];

pub struct
Metadata
{
	data: HashMap<ExifTag, ExifTagValue>,
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
		Metadata { endian: Endian::Big, data: HashMap::new() }
	}


	pub fn
	new_from_path
	(
		path: &Path
	)
	-> Metadata
	{
		Metadata { endian: Endian::Big, data: HashMap::new() }
	}
	

	pub fn
	get_tag
	(
		&self,
		tag: ExifTag
	)
	-> Option<&ExifTagValue> 
	{
		self.data.get(&tag)
	}


	pub fn
	set_tag
	(
		&mut self,
		tag: ExifTag,
		value: ExifTagValue
	)
	-> Result<(), String>
	{
		if !tag.is_writable() {
			return Err("This tag can't be set (it is not writable)".to_string());
		}

		if !tag.accepts(&value) {
			return Err("Tag not compatible with value".to_string());
		}

		self.data.insert(tag, value);

		return Ok(());
	}


	pub fn
	write_to_file
	(
		path: &Path
	)
	-> Result<(), String>
	{
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
		
		if !SUPPORTED_FILE_TYPES.contains(&file_type_str.unwrap().to_lowercase().as_str())
		{
			return Err("Unsupported file type!".to_string());
		}

		if !path.exists()
		{
			return Err("Can't write Metadata - File does not exist!".to_string());
		}

		return Ok(());
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
		exif_vec.extend((self.data.len() as u16).to_u8_vec(&self.endian).iter());

		assert!(exif_vec.len() == 10);

		// Compute what the first offset value will be in case we need that
		// Also provide vec for actual data stored in offset area
		let mut next_offset: u32 = 0			as u32
		+ exif_vec.len()						as u32
		+ IFD_ENTRY_LENGTH * (self.data.len() 	as u32) 
		+ IFD_END.len() 						as u32;
		let mut exif_offset_area: Vec<u8> = Vec::new();

		// Write IFD entries
		for (tag, value) in &self.data
		{

			assert!(tag.accepts(value));

			// Add Tag & Data Format /		2 + 2 bytes
			exif_vec.extend(tag.as_u16().to_u8_vec(&self.endian).iter());
			exif_vec.extend(tag.format().to_u8_vec(&self.endian).iter());

			// Add number of components /	4 bytes
			let number_of_components: u32 = (date.data.len() as u32) / tag.bytes_per_component();
			exif_vec.extend(number_of_components.to_u8_vec(&self.endian).iter());

			// Add offset or value /		4 bytes
			// Depending on the amount of data, either put it directly into the
			// next 4 bytes or write an offset where the data can be found 
			if date.data.len() > 4
			{
				exif_vec.extend(next_offset.to_u8_vec(&self.endian).iter());
				exif_offset_area.extend(date.data.iter());

				next_offset += date.data.len() as u32;
			}
			else
			{
				exif_vec.extend(date.data.iter());
			}
		}

		// Write end and offset data
		exif_vec.extend(IFD_END.iter());
		exif_vec.extend(exif_offset_area.iter());


		// The size of the EXIF data area, consists of
		// - length of EXIF header (follows after ssss)
		// - length of exif_vec
		// - 1 for ssss itself (why not 4? idk)
		let ssss = (EXIF_header.len() as u32 + exif_vec.len() as u32 + 1)
		.to_string();

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

		// Write EXIF header
		// (See next for loop comment for explanation)
		for byte in &EXIF_header
		{
			exif_all.push(byte / 16 + (if byte / 16 < 10 {'0' as u8} else {'a' as u8 - 10}));
			exif_all.push(byte % 16 + (if byte % 16 < 10 {'0' as u8} else {'a' as u8 - 10}));
		}

		// Every byte currently in exif_vec needs to be divided into its two hex digits
		// These two hex digits are then treated as ASCII characters
		// The value of these characters (e.g. 0x30 for '0') are then pushed to exif_all
		// Example: 48 (=0x30) in exif_vec results in the two consecutive values 51 and 48 in exif_all
		for byte in &exif_vec
		{
			exif_all.push(byte / 16 + (if byte / 16 < 10 {'0' as u8} else {'a' as u8 - 10}));
			exif_all.push(byte % 16 + (if byte % 16 < 10 {'0' as u8} else {'a' as u8 - 10}));
		}
		
		// Write end of EXIF data
		exif_all.push(0x30);
		exif_all.push(0x30);
		exif_all.push(NEWLINE);

		return exif_all;
	}
}
