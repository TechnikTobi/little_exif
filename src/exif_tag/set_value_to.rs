// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::endian::Endian;
use crate::u8conversion::*;

use super::ExifTag;
use super::ExifTagFormat;

macro_rules! build_set_function {
	( 
		$( (
			$function_name:ident, 
			$rust_type:ty,
			$format_type:ident
		) ),* 
	) 
	=>
	{
		impl ExifTag
		{
			/// This helps with handling tags that come in a different format
			/// than expected and, one converted, setting the data in the
			/// format that little_exif expects it to be.
			pub(crate) fn
			$(
				$function_name
			)*
			(
				&self,
				data: $($rust_type)*
			)
			-> Result<ExifTag, String>
			{
				match self.format()
				{
					$(ExifTagFormat::$format_type)* => {
						let endian   = Endian::Little;
						let raw_data = data.to_u8_vec(&endian);
						return Self::from_u16_with_data(
							self.as_u16(),
							&$(ExifTagFormat::$format_type)*,
							&raw_data,
							&endian,
							&self.get_group(),
						);
					}
					_ => Err(format!("Not a {:?} compatible tag!", $(ExifTagFormat::$format_type)*))
				}
			}
		}
	}
}

build_set_function![(set_value_to_int16u_vec, Vec<u16>, INT16U)];
build_set_function![(set_value_to_int32u_vec, Vec<u32>, INT32U)];