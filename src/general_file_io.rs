// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub(crate) const NEWLINE: u8 = 0x0a;
pub(crate) const SPACE: u8 = 0x20;
pub(crate) const EXIF: [u8; 4] = [0x45, 0x78, 0x69, 0x66];
pub(crate) const EXIF_HEADER: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];
pub(crate) const LITTLE_ENDIAN_INFO: [u8; 4] = [0x49, 0x49, 0x2a, 0x00];
pub(crate) const BIG_ENDIAN_INFO: [u8; 4] = [0x4d, 0x4d, 0x00, 0x2a];

macro_rules! perform_file_action {
    (
		$action: expr
	) => {
        let file_action_result = $action;
        if file_action_result.is_err() {
            return Err(file_action_result.err().unwrap());
        }
    };
}

macro_rules! io_error {
    ($kind:ident, $message:expr) => {
        Err(std::io::Error::new(std::io::ErrorKind::$kind, $message))
    };
}

use std::fs::{File, OpenOptions};
use std::path::Path;

pub(crate) fn open_read_file(path: &Path) -> Result<File, std::io::Error> {
    if !path.exists() {
        return io_error!(NotFound, "Can't open file - File does not exist!");
    }

    OpenOptions::new().read(true).write(false).open(path)
}

pub(crate) fn open_write_file(path: &Path) -> Result<File, std::io::Error> {
    if !path.exists() {
        return io_error!(NotFound, "Can't open file - File does not exist!");
    }

    OpenOptions::new().read(true).write(true).open(path)
}

pub(crate) use {io_error, perform_file_action};
