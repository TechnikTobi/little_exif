// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::debug_println;

use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;

#[allow(dead_code)]
#[derive(Clone)]
pub struct
IsoBox
{
    header: BoxHeader,
    data:   Vec<u8>,
}

impl
IsoBox
{
    fn
    construct_from_cursor_unboxed
    <T: Seek + Read>
    (
        cursor: &mut T,
        header:  BoxHeader
    )
    -> Result<IsoBox, std::io::Error> 
    {
        debug_println!("Constructing generic ISO box for type {:?}", header.get_box_type());

        // Check if this box is the last in the file
        // See also: ISO/IEC 14496-12:2015, § 4.2
        if header.get_box_size() == 0
        {
            let mut buffer = Vec::new();
            cursor.read_to_end(&mut buffer)?;
            return Ok(IsoBox {
                header: header,
                data:   buffer
            });
        }

        let data_left_to_read = header.get_box_size() - header.get_header_size();

        let mut buffer = vec![0u8; data_left_to_read];
        cursor.read_exact(&mut buffer)?;

        return Ok(IsoBox {
            header: header,
            data:   buffer
        });
    }
}

impl
ParsableIsoBox
for
IsoBox
{
    fn
    construct_from_cursor
    <T: Seek + Read>
    (
        cursor: &mut T,
        header:  BoxHeader
    )
    -> Result<Box<dyn GenericIsoBox>, std::io::Error> 
    {
        return Ok(Box::new(IsoBox::construct_from_cursor_unboxed(
            cursor, 
            header
        )?));
    }
}

impl
GenericIsoBox
for
IsoBox
{
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();
        serialized.extend(&self.data);
        return serialized;
    }

    fn as_any         (&    self) -> &    dyn std::any::Any {      self        }
    fn as_any_mut     (&mut self) -> &mut dyn std::any::Any {      self        }
    fn get_header     (&    self) -> &        BoxHeader     { &    self.header }
    fn get_header_mut (&mut self) -> &mut     BoxHeader     { &mut self.header }
}
