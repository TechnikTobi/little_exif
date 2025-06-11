// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::debug_println;

use crate::endian::Endian;
use crate::u8conversion::U8conversion;
use crate::u8conversion::to_u8_vec_macro;
use crate::util::read_be_u16;
use crate::util::read_be_u32;
use crate::util::read_null_terminated_string;

use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;

// - infe
// 00000015:   size of 0x15 bytes (including the 0x04 bytes of the size field itself) 
// 696E6665:   byte representation of `infe` 
// 02:         version 2
// 000001:     24 bits of flags
// 0019:       item ID (16 bits)
// 0000:       item protection index (16 bits)
// 6876633100: item name, a null terminated string, here: "hvc1"
// theoretically, after this point there would be two other strings, the
// content_type and the optional content_encoding, however, the practical
// examples did *not* have any of this

#[allow(dead_code)]
pub struct
ItemInfoEntryBox
{
    pub(self)  header:                BoxHeader,
    pub(crate) item_id:               u16,
    pub(crate) item_protection_index: u16,
    pub(crate) item_name:             String,
    pub(crate) additional_data:       Vec<u8>,
}

// - iinf
// 00000603:   size of 0x603 bytes (including the 0x04 bytes of the size field itself)
// 69696E66:   byte representation of `iinf` 
// 00000000:   version (here: 0) and 24 bits of flags
// 0041:       number of item info entries, here 0x41 = 65
// 0000001569: start of first info entry

#[allow(dead_code)]
pub struct
ItemInfoBox
{
    pub(self)  header:     BoxHeader,
    pub(crate) item_count: usize,
    pub(crate) items:      Vec<ItemInfoEntryBox>
}

impl
ItemInfoEntryBox
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
        let item_id               = read_be_u16(cursor)?;
        let item_protection_index = read_be_u16(cursor)?;
        let item_name             = read_null_terminated_string(cursor)?;

        // Determine how much data is left for this entry
        let data_read_so_far = header.get_header_size() 
            + 2                    // item_id
            + 2                    // item_protection_index
            + item_name.len() + 1; // string len + null terminator
        let data_left_to_read = header.get_box_size() - data_read_so_far;

        let mut additional_data = vec![0u8; data_left_to_read];
        cursor.read_exact(&mut additional_data)?;

        debug_println!("ID: {}, Name: {}", item_id, item_name);

        return Ok(ItemInfoEntryBox {
            header,
            item_id,
            item_protection_index,
            item_name,
            additional_data,
        });
    }
}

impl
ParsableIsoBox
for
ItemInfoEntryBox
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
        return Ok(Box::new(ItemInfoEntryBox::construct_from_cursor_unboxed(
            cursor, 
            header
        )?));
    }
}

impl
ItemInfoBox
{
    pub fn
    get_exif_item
    (
        &self
    )
    -> &ItemInfoEntryBox
    {
        return self.items.iter()
            .find(|item| item.item_name == "Exif")
            .unwrap();
    }
}

impl
ParsableIsoBox
for
ItemInfoBox
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
        let item_count;

        // See: ISO/IEC 14496-12:2015, § 8.11.6.2
        if header.get_version() == 0
        {
            item_count = read_be_u16(cursor)? as usize;
        }
        else
        {
            item_count = read_be_u32(cursor)? as usize;
        }

        let mut items = Vec::new();
        for _ in 0..item_count
        {
            let header = BoxHeader::read_box_header(cursor)?;
            items.push(ItemInfoEntryBox::construct_from_cursor_unboxed(
                cursor, 
                header
            )?);
        }

        return Ok(Box::new(ItemInfoBox { 
            header:     header,
            item_count: item_count, 
            items:      items 
        }));
    }
}

impl
GenericIsoBox
for
ItemInfoEntryBox
{
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();
        
        serialized.extend(to_u8_vec_macro!(u16, &self.item_id,               &Endian::Big).iter());
        serialized.extend(to_u8_vec_macro!(u16, &self.item_protection_index, &Endian::Big).iter());
        serialized.extend(self.item_name.bytes());
        serialized.push(0x00); // null terminator for item name string
        serialized.extend(&self.additional_data);

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
ItemInfoBox
{
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();

        if self.header.get_version() == 0
        {
            serialized.extend(to_u8_vec_macro!(u16, &(self.item_count as u16), &Endian::Big).iter());
        }
        else
        {
            serialized.extend(to_u8_vec_macro!(u32, &(self.item_count as u32), &Endian::Big).iter());
        }
        
        for item in &self.items
        {
            serialized.extend(item.serialize());
        }

        return serialized;
    }


    fn as_any         (&    self) -> &    dyn std::any::Any {      self        }
    fn as_any_mut     (&mut self) -> &mut dyn std::any::Any {      self        }
    fn get_header     (&    self) -> &        BoxHeader     { &    self.header }
    fn get_header_mut (&mut self) -> &mut     BoxHeader     { &mut self.header }
}