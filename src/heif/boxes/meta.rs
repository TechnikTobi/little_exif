// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;

use crate::endian::Endian;
use crate::u8conversion::U8conversion;
use crate::u8conversion::to_u8_vec_macro;
use crate::util::read_be_u32;

use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;

use super::read_box_based_on_header;

#[allow(dead_code)]
pub struct 
MetaBox
{
    header:           BoxHeader,
    handler_box:      HandlerBox,
    // primary_item_box: Option<IsoBox>, // pitm
    // data_info_box:    Option<IsoBox>, // dinf
    // item_loc_box:     Option<IsoBox>, // iloc
    // item_protect_box: Option<IsoBox>, // ipro
    // item_info_box:    Option<IsoBox>, // iinf
    // ipmp_control_box: Option<IsoBox>, // ipmc
    // item_ref_box:     Option<IsoBox>, // iref
    // item_data_box:    Option<IsoBox>, // idat
    pub(crate) other_boxes:      Vec<Box<dyn GenericIsoBox>>,
}

impl
ParsableIsoBox
for
MetaBox
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
        // Read in the remaining bytes for this box
        let     remaining_bytes = header.get_box_size() - header.get_header_size();
        let mut meta_box_bytes  = vec![0u8; remaining_bytes];
        cursor.read_exact(&mut meta_box_bytes)?;

        // Construct local cursor for these bytes
        let mut local_cursor = Cursor::new(meta_box_bytes);

        // Read in the mandatory handler box
        let handler_box_header = BoxHeader::read_box_header(&mut local_cursor)?;
        let handler_box        = HandlerBox::construct_from_cursor_unboxed(
            &mut local_cursor, 
            handler_box_header
        )?;

        // Read in other boxes
        let mut other_boxes = Vec::new();
        while local_cursor.position() < remaining_bytes as u64
        {
            let sub_header = BoxHeader::read_box_header(&mut local_cursor)?;
            let sub_box    = read_box_based_on_header(
                &mut local_cursor, 
                sub_header
            )?;

            other_boxes.push(sub_box);
        }

        return Ok(Box::new(MetaBox { 
            header:           header,
            handler_box:      handler_box,
            other_boxes:      other_boxes,
        }));
    }
}

#[allow(dead_code)]
pub struct
HandlerBox
{
    header:       BoxHeader,
    pre_defined:  u32,
    handler_type: u32,
    reserved:     [u32; 3],
    name:         Vec<u8> // UTF-8 string, don't bother decoding
}

impl
HandlerBox
{
    fn
    construct_from_cursor_unboxed
    <T: Seek + Read>
    (
        cursor: &mut T,
        header:  BoxHeader
    )
    -> Result<Self, std::io::Error>
    {
        let pre_defined  = read_be_u32(cursor)?;
        let handler_type = read_be_u32(cursor)?;
        let reserved     = [
            read_be_u32(cursor)?,
            read_be_u32(cursor)?,
            read_be_u32(cursor)?
        ];

        let number_of_bytes_that_form_the_name = header.get_box_size() 
            - header.get_header_size() // header
            - 4                        // pre_defined
            - 4                        // handler_type
            - 12                       // reserved
            ;

        let mut name_buffer = vec![0u8; number_of_bytes_that_form_the_name];
        cursor.read_exact(&mut name_buffer)?;

        return Ok(HandlerBox { 
            header:       header, 
            pre_defined:  pre_defined, 
            handler_type: handler_type, 
            reserved:     reserved, 
            name:         name_buffer 
        });
    }
}

impl
GenericIsoBox
for
MetaBox
{
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();
        serialized.extend(self.handler_box.serialize());
        for sub_box in &self.other_boxes
        {
            serialized.extend(sub_box.serialize());
        }

        return serialized;
    }


    fn as_any     (&    self) -> &    dyn std::any::Any {  self       }
    fn as_any_mut (&mut self) -> &mut dyn std::any::Any {  self       }
    fn get_header (&    self) -> &        BoxHeader     { &self.header}
}

impl
GenericIsoBox
for
HandlerBox
{
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();

        serialized.extend(to_u8_vec_macro!(u32, &self.pre_defined,  &Endian::Big).iter());
        serialized.extend(to_u8_vec_macro!(u32, &self.handler_type, &Endian::Big).iter());
        for value in &self.reserved
        {
            serialized.extend(to_u8_vec_macro!(u32, &value, &Endian::Big).iter());
        }
        serialized.extend(&self.name);

        return serialized;
    }

    fn as_any     (&    self) -> &    dyn std::any::Any {  self       }
    fn as_any_mut (&mut self) -> &mut dyn std::any::Any {  self       }
    fn get_header (&    self) -> &        BoxHeader     { &self.header}
}