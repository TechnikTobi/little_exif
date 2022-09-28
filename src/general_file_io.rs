use std::io::{Error, ErrorKind};

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

pub(crate) use io_error;