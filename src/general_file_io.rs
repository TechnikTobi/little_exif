pub const NEWLINE: u8 = 0x0a;
pub const SPACE: u8 = 0x20;
pub const EXIF_HEADER: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];

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