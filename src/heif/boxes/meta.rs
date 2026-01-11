// Copyright © 2025-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;

use crate::endian::Endian;
use crate::general_file_io::io_error;
use crate::u8conversion::U8conversion;
use crate::u8conversion::to_u8_vec_macro;
use crate::util::read_be_u32;

use crate::heif::box_type::BoxType;
use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;

use crate::heif::boxes::item_info::ItemInfoBox;
use crate::heif::boxes::item_location::ItemLocationBox;
use crate::heif::boxes::item_reference::ItemReferenceBox;

use super::read_box_based_on_header;

#[allow(dead_code)]
pub struct 
MetaBox
{
    header:                 BoxHeader,
    handler_box:            HandlerBox,
    // primary_item_box:    Option<IsoBox>,              // pitm
    // data_info_box:       Option<IsoBox>,              // dinf
    // item_loc_box:        Option<IsoBox>,              // iloc
    // item_protect_box:    Option<IsoBox>,              // ipro
    // item_info_box:       Option<IsoBox>,              // iinf
    // ipmp_control_box:    Option<IsoBox>,              // ipmc
    // item_ref_box:        Option<IsoBox>,              // iref
    // item_data_box:       Option<IsoBox>,              // idat
    pub(crate) other_boxes: Vec<Box<dyn GenericIsoBox>>,
}

impl
MetaBox
{
    pub(crate) fn
    get_item_info_box
    (
        &self
    )
    -> Result<&ItemInfoBox, std::io::Error>
    {
        match self.other_boxes.iter().find(|b| b.get_header().get_box_type() == BoxType::iinf) {
            Some(b) => match b.as_any().downcast_ref::<ItemInfoBox>() {
                Some(unboxed) => Ok(unboxed),
                None => io_error!(
                    InvalidData,
                    "Found iinf box but could not downcast to ItemInfoBox"
                ),
            },
            None => io_error!(
                NotFound,
                "No iinf box found in MetaBox"
            ),
        }
    }

    pub(crate) fn
    get_item_location_box
    (
        &self
    )
    -> Result<&ItemLocationBox, std::io::Error>
    {
        match self.other_boxes.iter().find(|b| b.get_header().get_box_type() == BoxType::iloc) {
            Some(b) => match b.as_any().downcast_ref::<ItemLocationBox>() {
                Some(unboxed) => Ok(unboxed),
                None => io_error!(
                    InvalidData,
                    "Found iloc box but could not downcast to ItemLocationBox"
                ),
            },
            None => io_error!(
                NotFound,
                "No iloc box found in MetaBox"
            ),
        }
    }

    pub(crate) fn
    get_item_location_box_mut
    (
        &mut self
    )
    -> Result<&mut ItemLocationBox, std::io::Error>
    {
        match self.other_boxes.iter_mut().find(|b| b.get_header().get_box_type() == BoxType::iloc) {
            Some(b) => match b.as_any_mut().downcast_mut::<ItemLocationBox>() {
                Some(unboxed) => Ok(unboxed),
                None => io_error!(
                    InvalidData,
                    "Found iloc box but could not downcast to ItemLocationBox"
                ),
            },
            None => io_error!(
                NotFound,
                "No iloc box found in MetaBox"
            ),
        }
    }

    pub(crate) fn
    get_item_reference_box
    (
        &self
    )
    -> Option<&ItemReferenceBox>
    {
        if let Some(found_box) = self.other_boxes.iter()
            .find(|b| b.get_header().get_box_type() == BoxType::iref)
        {
            return found_box.as_any().downcast_ref::<ItemReferenceBox>();
        }
        return None;
    }

    pub(crate) fn
    create_new_item_reference_box_if_none_exists_yet
    (
        &mut self
    )
    {
        if self.get_item_reference_box().is_some()
        {
            return;
        }

        let new_iref_box = ItemReferenceBox::new();

        let index = self.other_boxes
            .iter()
            .position(|x| x.get_header().get_box_type() == BoxType::iinf)
            .expect("Could not find iinf box to insert iref before");
         
        self.other_boxes.insert(index, Box::new(new_iref_box));
    }
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
        if header.get_box_size() < header.get_header_size()
        {
            return io_error!(
                InvalidData,
                format!(
                    "MetaBox has invalid size: box size {} is smaller than header size {}",
                    header.get_box_size(),
                    header.get_header_size()
                )
            );
        }

        if header.get_box_size() < header.get_header_size() {
            return io_error!(
                InvalidData,
                format!(
                    "MetaBox has invalid size: box size {} is too small",
                    header.get_box_size()
                )
            );
        }
        // Read in the remaining bytes for this box
        let     remaining_bytes = header.get_box_size() - header.get_header_size();
        let mut meta_box_bytes: Vec<u8> = Vec::new();
        meta_box_bytes.try_reserve_exact(remaining_bytes as usize)?;
        cursor.take(remaining_bytes as u64).read_to_end(&mut meta_box_bytes)?;

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
            header,
            handler_box,
            other_boxes,
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

        // Check that there is enough data left to read the box name
        if header.get_box_size() < header.get_header_size() + 4 + 4 + 12 
        {
            return io_error!(
                InvalidData,
                format!(
                    "HandlerBox has invalid size: box size {} is too small to contain mandatory name field",
                    header.get_box_size()
                )
            );
        }

        // Check that the remaining data is not unreasonably large 
        // This threshold is somewhat arbitrary
        if header.get_box_size() > (u32::MAX/16) as u64
        {
            return io_error!(
                Unsupported,
                format!(
                    "HandlerBox size {} exceeds maximum supported size ({})",
                    header.get_box_size(),
                    (u32::MAX/16)
                )
            );
        }

        let number_of_bytes_that_form_the_name = header.get_box_size() as u64
            - header.get_header_size() as u64 // header
            - 4                               // pre_defined
            - 4                               // handler_type
            - 12                              // reserved
            ;

        let mut name: Vec<u8> = Vec::new();

        // This may cause an out of memory error, but won't panic like vec![]
        name.try_reserve_exact(number_of_bytes_that_form_the_name as usize)?;

        // Can't use read_exact here because the name buffer we read into is
        // still size 0 (only has reserved capacity!)
        cursor
            .take(number_of_bytes_that_form_the_name as u64)
            .read_to_end(&mut name)?;

        return Ok(HandlerBox { 
            header, 
            pre_defined,
            handler_type,
            reserved,
            name, 
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


    fn as_any         (&    self) -> &    dyn std::any::Any {      self        }
    fn as_any_mut     (&mut self) -> &mut dyn std::any::Any {      self        }
    fn get_header     (&    self) -> &        BoxHeader     { &    self.header }
    fn get_header_mut (&mut self) -> &mut     BoxHeader     { &mut self.header }
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

    fn as_any         (&    self) -> &    dyn std::any::Any {      self        }
    fn as_any_mut     (&mut self) -> &mut dyn std::any::Any {      self        }
    fn get_header     (&    self) -> &        BoxHeader     { &    self.header }
    fn get_header_mut (&mut self) -> &mut     BoxHeader     { &mut self.header }
}