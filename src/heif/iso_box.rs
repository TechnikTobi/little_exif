// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::util::read_1_bytes;
use crate::util::read_3_bytes;
use crate::util::read_4_bytes;
use crate::util::read_be_u16;
use crate::util::read_be_u32;
use crate::util::read_be_u64;
use crate::util::read_null_terminated_string;

use super::box_type::BoxType;
use super::box_header::BoxHeader;


pub struct 
MetaBox
{
    header:           BoxHeader,
    handler_box:      HandlerBox,
    primary_item_box: Option<IsoBox>,
    data_info_box:    Option<IsoBox>,
    item_loc_box:     Option<IsoBox>,
    item_protect_box: Option<IsoBox>,
    item_info_box:    Option<IsoBox>,
    ipmp_control_box: Option<IsoBox>,
    item_ref_box:     Option<IsoBox>,
    item_data_box:    Option<IsoBox>,
    other_boxes:      Vec<IsoBox>,
}

//  extends FullBox(‘meta’, version = 0, 0) {

// HandlerBox(handler_type) theHandler;
// PrimaryItemBox primary_resource; // optional
// DataInformationBox file_locations; // optional
// ItemLocationBox item_locations; // optional
// ItemProtectionBox protections; // optional
// ItemInfoBox item_infos; // optional
// IPMPControlBox IPMP_control; // optional
// ItemReferenceBox item_refs; // optional
// ItemDataBox item_data; // optional
// Box other_boxes[]; // optional 


pub struct
HandlerBox
{
    header: BoxHeader,
}

// Examples:
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
            + 2 // item_id
            + 2 // item_protection_index
            + item_name.len();
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
GenericIsoBox
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
    -> Result<Box<Self>, std::io::Error>
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


pub struct
ItemInfoBox
{
    header:     BoxHeader,
    item_count: usize,
    items:      Vec<ItemInfoEntryBox>
}

impl
GenericIsoBox
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
    -> Result<Box<Self>, std::io::Error>
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

// - iloc


pub trait
GenericIsoBox
{
    fn
    construct_from_cursor
    <T: Seek + Read>
    (
        cursor: &mut T,
        header:  BoxHeader
    )
    -> Result<Box<Self>, std::io::Error>;
}

pub struct
IsoBox
{
    header:    BoxHeader,
    data:      Vec<u8>,
}

impl
GenericIsoBox
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
    -> Result<Box<Self>, std::io::Error> 
    {
        // Check if this box is the last in the file
        // See also: ISO/IEC 14496-12:2015, § 4.2
        if header.get_box_size() == 0
        {
            let mut buffer = Vec::new();
            cursor.read_to_end(&mut buffer);
            return Ok(Box::new(IsoBox {
                header: header,
                data:   buffer
            }));
        }

        let data_left_to_read = header.get_box_size() - header.get_header_size();

        let mut buffer = vec![0u8; data_left_to_read];
        cursor.read_exact(&mut buffer)?;

        return Ok(Box::new(IsoBox {
            header: header,
            data:   buffer
        }));
    }
}