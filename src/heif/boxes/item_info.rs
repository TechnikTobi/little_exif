// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::util::read_be_u16;
use crate::util::read_be_u32;
use crate::util::read_null_terminated_string;

use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;
use crate::heif::boxes::impl_generic_iso_box;

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
    header:                BoxHeader,
    item_id:               u16,
    item_protection_index: u16,
    item_name:             String,
    additional_data:       Vec<u8>,
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

        return Ok(ItemInfoEntryBox {
            header:                header,
            item_id:               item_id,
            item_protection_index: item_protection_index,
            item_name:             item_name,
            additional_data:       additional_data,
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
    header:     BoxHeader,
    item_count: usize,
    items:      Vec<ItemInfoEntryBox>
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

impl_generic_iso_box!(
    ItemInfoEntryBox,
    ItemInfoBox
);