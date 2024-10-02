// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub(crate) const NEWLINE:                u8      = 0x0a;
pub(crate) const SPACE:                  u8      = 0x20;
pub(crate) const EXIF:                   [u8; 4] = [0x45, 0x78, 0x69, 0x66];
pub(crate) const EXIF_HEADER:            [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];

macro_rules! perform_file_action {
	( 
		$action: expr
	) 
	=>
	{
		let file_action_result = $action;
		if file_action_result.is_err()
		{
			return Err(file_action_result.err().unwrap());
		}
	};
}

macro_rules! io_error {
	($kind:ident, $message:expr)
	=>
	{
		Err(std::io::Error::new(
			std::io::ErrorKind::$kind,
			$message
		))
	};
}

pub(crate) use perform_file_action;
pub(crate) use io_error;