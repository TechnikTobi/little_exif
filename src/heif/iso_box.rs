// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;

use crate::general_file_io::io_error;
use crate::heif::iso_box;
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
    // primary_item_box: Option<IsoBox>, // pitm
    // data_info_box:    Option<IsoBox>, // dinf
    // item_loc_box:     Option<IsoBox>, // iloc
    // item_protect_box: Option<IsoBox>, // ipro
    // item_info_box:    Option<IsoBox>, // iinf
    // ipmp_control_box: Option<IsoBox>, // ipmc
    // item_ref_box:     Option<IsoBox>, // iref
    // item_data_box:    Option<IsoBox>, // idat
    other_boxes:      Vec<IsoBox>,
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
            // let sub_box    = IsoBox::construct_from_cursor_unboxed(&mut local_cursor, sub_header);

            let boxed_sub_box = IsoBox::construct_from_cursor(&mut local_cursor, sub_header)?;
            let sub_box = match boxed_sub_box.as_any().downcast_ref::<IsoBox>() {
                Some(iso_box) => iso_box,
                None          => panic!("&a isn't a B!")
            };

            other_boxes.push(sub_box.clone());
        }

        return Ok(Box::new(MetaBox { 
            header:           header,
            handler_box:      handler_box,
            // primary_item_box: None,
            // data_info_box:    None,
            // item_loc_box:     None,
            // item_protect_box: None,
            // item_info_box:    None,
            // ipmp_control_box: None,
            // item_ref_box:     None,
            // item_data_box:    None,
            other_boxes:      other_boxes,
        }));
    }
}

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
ParsableIsoBox
for
HandlerBox
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
        return Ok(Box::new(HandlerBox::construct_from_cursor_unboxed(
            cursor, 
            header
        )?));
    }
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

// - iloc
pub struct
ItemLocationBox
{
    header:           BoxHeader,
    offset_size:      u8,  // actually u4
    length_size:      u8,  // actually u4
    base_offset_size: u8,  // actually u4
    reserved:         u8,  // actually u4, 
                           // if version == 1 || 2 this is called index_size
    item_count:       u32, // only if version == 2, if version < 2 this is u16
    items:            Vec<ItemLocationEntry>
}

pub struct
ItemLocationEntry
{
    item_id:                          u32, // only if version == 2, 
                                           // if version < 2 this is u16
    reserved_and_construction_method: u16, // first 12 bits are reserved, the
                                           // other 4 are construction method:
                                           // - 0: file
                                           // - 1: idat
                                           // - 2: item
                                           // only present if version == 1 || 2
    data_reference_index:             u16,
    base_offset:                      u64, // actual size depends on value of
                                           // base_offset_size * 8
    extent_count:                     u16, // must be equal or greater 1
    extents:                          Vec<ItemLocationEntryExtentEntry>,
}

pub struct
ItemLocationEntryExtentEntry
{
    extent_index:  Option<u64>, // only if (version == 1 || 2) && index_size>0
                                // actual size depends on index_size  * 8
    extent_offset: u64,         // actual size depends on offset_size * 8
    extent_length: u64,         // actual size depends on length_size * 8
}

/*
0001: item_id
0000: reserved and construction method
0000: data ref index
// as base offset size is zero, no bytes for base offset
0001: extent count
// as index size is also zero, no bytes for extent index
00004841: extent offset
0000052D: extent length
*/

impl 
ItemLocationEntryExtentEntry
{
    fn
    read_from_cursor
    <T: Seek + Read>
    (
        cursor:     &mut T,
        header:     &BoxHeader,
        offset_size: u8,
        length_size: u8,
        index_size:  u8,
    )
    -> Result<Self, std::io::Error>
    {
        let extent_index = if 
            (header.get_version() == 1 || header.get_version() == 2)
            &&
            index_size > 0
        {
            match index_size
            {
                4 => Some(read_be_u32(cursor)? as u64),
                8 => Some(read_be_u64(cursor)?),
                _ => panic!("Invalid index_size!")
            }
        }
        else
        {
            None
        };

        let extent_offset = match offset_size
        {
            0 => 0,
            4 => read_be_u32(cursor)? as u64,
            8 => read_be_u64(cursor)?,
            _ => panic!("Invalid offset_size!")
        };

        let extent_length = match length_size
        {
            0 => 0,
            4 => read_be_u32(cursor)? as u64,
            8 => read_be_u64(cursor)?,
            _ => panic!("Invalid length_size!")
        };

        return Ok(Self{extent_index, extent_offset, extent_length});
    }
}

impl
ItemLocationEntry
{
    fn
    read_from_cursor
    <T: Seek + Read>
    (
        cursor:           &mut T,
        header:           &BoxHeader,
        offset_size:       u8,
        length_size:       u8,
        base_offset_size:  u8,
        index_size:        u8,
    )
    -> Result<Self, std::io::Error>
    {
        let item_id = match header.get_version()
        {
            0 | 1 => read_be_u16(cursor)? as u32,
            2     => read_be_u32(cursor)?,
            _     => panic!("Invalid version for ItemLocationEntry decode!")
        };

        let reserved_and_construction_method = if 
        (header.get_version() == 1) || (header.get_version() == 2)
        { read_be_u16(cursor)? } else { 0 };

        let data_reference_index = read_be_u16(cursor)?;
        let base_offset = match base_offset_size
        {
            0 => 0,
            4 => read_be_u32(cursor)? as u64,
            8 => read_be_u64(cursor)?,
            _ => panic!("Invalid base_offset_size!")
        };

        let extent_count = read_be_u16(cursor)?;

        let mut extents = Vec::new();

        for _ in 0..extent_count
        {
            extents.push(ItemLocationEntryExtentEntry::read_from_cursor(
                cursor, 
                header, 
                offset_size, 
                length_size, 
                index_size
            )?);
        }

        return Ok(Self { 
            item_id, 
            reserved_and_construction_method, 
            data_reference_index, 
            base_offset, 
            extent_count, 
            extents
        });
    }
}


pub trait 
GenericIsoBox 
{
    fn as_any(&self) -> &dyn std::any::Any;
}

impl GenericIsoBox for MetaBox          { fn as_any(&self) -> &dyn std::any::Any {self} }
impl GenericIsoBox for HandlerBox       { fn as_any(&self) -> &dyn std::any::Any {self} }
impl GenericIsoBox for ItemInfoBox      { fn as_any(&self) -> &dyn std::any::Any {self} }
impl GenericIsoBox for ItemInfoEntryBox { fn as_any(&self) -> &dyn std::any::Any {self} }
impl GenericIsoBox for IsoBox           { fn as_any(&self) -> &dyn std::any::Any {self} }

pub trait
ParsableIsoBox: GenericIsoBox
{
    fn
    construct_from_cursor
    <T: Seek + Read>
    (
        cursor: &mut T,
        header:  BoxHeader
    )
    -> Result<Box<dyn GenericIsoBox>, std::io::Error>;
}

#[derive(Clone)]
pub struct
IsoBox
{
    header:    BoxHeader,
    data:      Vec<u8>,
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
        println!("Constructing generic ISO box for type {:?}", header.get_box_type());

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


pub(super) fn
read_box_based_on_header
<T: Seek + Read>
(
    cursor: &mut T,
    header:  BoxHeader
)
-> Result<Box<dyn GenericIsoBox>, std::io::Error>
{
    return match header.get_box_type()
    {
        BoxType::meta => MetaBox::    construct_from_cursor(cursor, header),
        BoxType::iinf => ItemInfoBox::construct_from_cursor(cursor, header),
        _             => IsoBox::     construct_from_cursor(cursor, header)
    };
}

pub(super) fn
read_next_box
<T: Seek + Read>
(
    cursor: &mut T,
)
-> Result<Box<dyn GenericIsoBox>, std::io::Error>
{
    let header = BoxHeader::read_box_header(cursor)?;

    println!("{:?}", header);

    return read_box_based_on_header(cursor, header);
}