use std::collections::HashMap;
use std::path::Path;

use crate::endian::Endian;
use crate::exif_tag::ExifTag;
use crate::exif_tag_value::ExifTagValue;

pub struct
Metadata
{
	data: HashMap<ExifTag, ExifTagValue>,
	endian: Endian 
}

const SUPPORTED_FILE_TYPES: [&'static str; 1] = [
	"png"
];

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

	/*
	pub fn
	get_tag
	(
		&self,
		tag: &str
	)
	-> Result<ExifTagValue, String>
	{
		
		Ok(())	
	}
	*/
	
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

		// According to the compiler this is currently unstable - using exists() instead...
		/*		
		let file_exists_check = path.try_exists();

		if file_exists_check.is_err()
		{
			return file_exists_check;
		}
		*/

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
		Vec::new()
	}
}
