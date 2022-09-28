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