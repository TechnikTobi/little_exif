use std::collections::HashMap;

use crate::endian::Endian;
use crate::exif_tag::ExifTag;
use crate::exif_tag_value::ExifTagValue;

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
		Metadata { data: HashMap::new() }
	}

	pub fn
	new_from_path
	(
		path: &String
	)
	-> Metadata
	{
		Metadata { data: HashMap::new() }
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
		&self,
		tag: ExifTag,
		value: ExifTagValue
	)
	-> Result<(), String>
	{
		Ok(())		
	}
}
