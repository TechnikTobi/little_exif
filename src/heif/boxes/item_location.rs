// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::util::read_be_u16;
use crate::util::read_be_u32;
use crate::util::read_be_u64;

use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;
use crate::heif::boxes::impl_generic_iso_box;

#[allow(dead_code)]
pub(crate) struct
ItemLocationBox
{
    pub(self)  header:           BoxHeader,

    pub(crate) offset_size:      u8,  // actually u4
    pub(crate) length_size:      u8,  // actually u4
    pub(crate) base_offset_size: u8,  // actually u4

    pub(crate) index_size:       u8,  
        // actually u4, 
        // only available if version == 1 || 2, otherwise these 4 bytes are
        // handled as `reserved`

    pub(crate) item_count:       u32, 
        // only if version == 2, if version < 2 this is u16

    pub(crate) items:            Vec<ItemLocationEntry>
}



#[derive(Debug)]
pub(crate) enum
ItemConstructionMethod
{
    FILE,
    IDAT,
    ITEM,
}



#[allow(dead_code)]
pub(crate) struct
ItemLocationEntry
{
    pub(crate) item_id:                          u32, 
        // only if version == 2,  if version < 2 this is u16

    pub(crate) reserved_and_construction_method: u16, 
        // first 12 bits are reserved, the other 4 are construction method:
        // - 0: file
        // - 1: idat
        // - 2: item
        // only present if version == 1 || 2

    pub(crate) data_reference_index:             u16,
    pub(crate) base_offset:                      u64, 
        // actual size depends on value of base_offset_size * 8

    pub(crate) extent_count:                     u16, 
        // must be equal or greater 1

    pub(crate) extents:                          Vec<ItemLocationEntryExtentEntry>,
}



#[allow(dead_code)]
pub(crate) struct
ItemLocationEntryExtentEntry
{
    pub(crate) extent_index:  Option<u64>, 
        // only if (version == 1 || 2) && index_size>0
        // actual size depends on index_size  * 8

    pub(crate) extent_offset: u64,
        // actual size depends on offset_size * 8

    pub(crate) extent_length: u64,
        // actual size depends on length_size * 8
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

        let entry = Self { 
            item_id, 
            reserved_and_construction_method, 
            data_reference_index, 
            base_offset, 
            extent_count, 
            extents
        };

        // println!("Item: {} Location: {:x}, Length: {:x}, Construction: {:?}", 
        //     item_id, 
        //     entry.extents.first().unwrap().extent_offset, 
        //     entry.extents.first().unwrap().extent_length, 
        //     entry.get_construction_method()
        // );

        return Ok(entry);
    }

    pub(super) fn
    get_construction_method
    (
        &self
    )
    -> ItemConstructionMethod
    {
        return match self.reserved_and_construction_method as u8 & 0x0f
        {
            0 => ItemConstructionMethod::FILE,
            1 => ItemConstructionMethod::IDAT,
            2 => ItemConstructionMethod::ITEM,
            _ => panic!("Unknown item construction method!")
        };
    }
}

impl
ItemLocationBox
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
        let temp = read_be_u16(cursor)?;
        let (offset_size,        length_size,              base_offset_size) =
            ((temp >> 12) as u8, (temp >> 8 & 0x0f) as u8, (temp >> 4 & 0x0f) as u8);
        let index_size = match header.get_version()
        {
            1 | 2 => temp as u8 & 0x0f,
            _     => 0,
        };

        let item_count = match header.get_version()
        {
            0 | 1 => read_be_u16(cursor)? as u32,
            2     => read_be_u32(cursor)?,
            _     => panic!("Invalid version for ItemLocationBox decode!")
        };

        let mut items = Vec::new();
        for _ in 0..item_count
        {
            items.push(ItemLocationEntry::read_from_cursor(
                cursor, 
                &header, 
                offset_size, 
                length_size, 
                base_offset_size, 
                index_size
            )?);
        }

        return Ok(ItemLocationBox {
            header,
            offset_size,
            length_size,
            base_offset_size,
            index_size,
            item_count,
            items
        });
    }
}

impl
ParsableIsoBox
for
ItemLocationBox
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
        return Ok(Box::new(ItemLocationBox::construct_from_cursor_unboxed(
            cursor, 
            header
        )?));
    }
}

impl_generic_iso_box!(
    ItemLocationBox
);