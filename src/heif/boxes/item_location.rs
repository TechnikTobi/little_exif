// Copyright © 2025-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::debug_println;
use crate::io_error_plain;
use crate::endian::Endian;
use crate::general_file_io::io_error;
use crate::u8conversion::U8conversion;
use crate::u8conversion::to_u8_vec_macro;
use crate::util::read_be_u16;
use crate::util::read_be_u32;
use crate::util::read_be_u64;

use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;

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



#[derive(Debug, PartialEq)]
pub(crate) enum
ItemConstructionMethod
{
    FILE = 0,
    IDAT = 1,
    ITEM = 2,
}



#[allow(dead_code)]
#[derive(Debug)]
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
#[derive(Debug)]
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
                _ => return io_error!(Other, format!("Invalid index_size: {}!", index_size))
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
            _ => return io_error!(Other, format!("Invalid offset_size: {}!", offset_size))
        };

        let extent_length = match length_size
        {
            0 => 0,
            4 => read_be_u32(cursor)? as u64,
            8 => read_be_u64(cursor)?,
            _ => return io_error!(Other, format!("Invalid length_size: {}!", length_size))
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
            _     => return io_error!(Other, "Invalid version for ItemLocationEntry decode!".to_string())
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
            _ => return io_error!(Other, "Invalid base_offset_size!".to_string())
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

        debug_println!("{:?}", entry);

        return Ok(entry);
    }

    pub(crate) fn
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

    /*
    pub(crate) fn
    get_size
    (
        &self,
        parent: &ItemLocationBox
    )
    -> usize
    {
        let mut size = 0usize;

        // item_id
        if parent.get_header().get_version() == 2
        {
            size += 4;
        }
        else
        {
            size += 2;
        }

        // reserved_and_construction_method
        if 
            parent.get_header().get_version() == 1
            ||
            parent.get_header().get_version() == 2
        {
            size += 2;
        }

        // data_reference_index
        size += 2;

        // base_offset
        size += parent.base_offset_size as usize;

        // extent_count
        size += 2;

        // extents
        for extent in &self.extents
        {
            size += extent.get_size(
                parent.offset_size, 
                parent.length_size, 
                parent.index_size
            );
        }

        return size;
    }
    */
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
        let (offset_size, length_size, base_offset_size) =
            (
                (temp >> 12 & 0x0f) as u8, 
                (temp >> 8  & 0x0f) as u8, 
                (temp >> 4  & 0x0f) as u8
            );
        let index_size = match header.get_version()
        {
            1 | 2 => temp as u8 & 0x0f,
            _     => 0,
        };

        let item_count = match header.get_version()
        {
            0 | 1 => read_be_u16(cursor)? as u32,
            2     => read_be_u32(cursor)?,
            _     => return io_error!(Other, "Invalid version for ItemLocationBox decode!".to_string())
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

    pub(crate) fn
    get_item_location_entry
    (
        &self,
        item_id: u16
    )
    -> Result<&ItemLocationEntry, std::io::Error>
    {
        self.items.iter()
            .find(|item| item.item_id == item_id as u32)
            .ok_or(
                io_error_plain!(
                    Other, 
                    format!("ItemLocationEntry with item_id {} not found!", item_id)
                )
            )
    }

    // Returns the ID of the new entry and by how many bytes this box got longer
    pub(crate) fn
    create_new_item_location_entry
    (
        &mut self,
        data_start:  u64,
        data_length: u64
    )
    -> (u32, u64)
    {
        // Determine largest iloc ID so far
        let old_largest_id = self.items
            .iter()
            .map(|x| x.item_id)
            .max()
            .unwrap_or(0);

        self.items.push(ItemLocationEntry 
            {
                item_id:                          old_largest_id + 1, 
                reserved_and_construction_method: 0, 
                data_reference_index:             0, 
                base_offset:                      0,
                extent_count:                     1, 
                extents:                          vec![
                    ItemLocationEntryExtentEntry 
                    {
                        extent_index:  Some(0),
                        extent_offset: data_start,
                        extent_length: data_length,
                    }
                ]
            }
        );

        self.item_count += 1;

        // Due to the addition of a new item, the size in the header needs to 
        // be adjusted as well
        // TODO: make this more efficient by only computing how much memory is
        // needed, not by actually serializing (and thus, allocating memory)
        let old_box_size = self.header.get_box_size();
        let new_box_size = self.serialize().len() as u64;
        self.header.set_box_size(new_box_size);

        return (
            self.items.last_mut().expect("No items present after insertion").item_id,
            new_box_size - old_box_size
        );
    }

    pub(crate) fn
    add_to_extents
    (
        &mut self,
        value: i64
    )
    {
        for item in &mut self.items
        {
            if item.get_construction_method() == ItemConstructionMethod::IDAT
            {
                // In this case the offset information is relative to the
                // position of an idat box -> not affected by change in length
                // of another box
                continue;
            }

            if item.get_construction_method() == ItemConstructionMethod::ITEM
            {
                // Offset is relative to another item's extent
                // Also nothing to do here (for now...)
                continue;
            }

            if item.data_reference_index != 0
            {
                // A value other than 0 implies that this extent refers to
                // another file, not this one, so we can also skip this
                // See ISO/IEC 14496-12:2015 § 8.11.3.1, p. 78
                continue;
            }

            // For now, just add the value to the extent offsets. 
            // This may be problematic in case the offset points to a location
            // before the iloc or iinf boxes, so changing their length won't
            // affect that offset value
            for extent in &mut item.extents
            {
                extent.extent_offset = (extent.extent_offset as i64 + value) as u64;
            }
        }
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



impl
GenericIsoBox
for
ItemLocationBox
{
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();

        // let (offset_size,        length_size,              base_offset_size) =
        //     ((temp >> 12) as u8, (temp >> 8 & 0x0f) as u8, (temp >> 4 & 0x0f) as u8);

        #[allow(clippy::double_parens)]
        let temp = 0u16 
            + ((self.offset_size      as u16) << 12)
            + ((self.length_size      as u16) <<  8)
            + ((self.base_offset_size as u16) <<  4)
            + ((self.index_size       as u16) <<  0)
            ;

        serialized.extend(to_u8_vec_macro!(u16, &temp, &Endian::Big).iter());

        match self.header.get_version()
        {
            0 | 1 => serialized.extend(to_u8_vec_macro!(u16, &(self.item_count as u16), &Endian::Big).iter()),
            2     => serialized.extend(to_u8_vec_macro!(u32, & self.item_count,         &Endian::Big).iter()),
            _     => panic!("Invalid version for ItemLocationBox serialize!")
        };

        for item in &self.items
        {
            match self.header.get_version()
            {
                0 | 1 => serialized.extend(to_u8_vec_macro!(u16, &(item.item_id as u16), &Endian::Big).iter()),
                2     => serialized.extend(to_u8_vec_macro!(u32, & item.item_id,         &Endian::Big).iter()),
                _     => panic!("Invalid version for ItemLocationBox serialize!")
            };

            if (self.header.get_version() == 1) || (self.header.get_version() == 2)
            {
                serialized.extend(to_u8_vec_macro!(u16, &item.reserved_and_construction_method, &Endian::Big).iter());
            }
            
            serialized.extend(to_u8_vec_macro!(u16, &item.data_reference_index, &Endian::Big).iter());
            match self.base_offset_size
            {
                0 => (),
                4 => serialized.extend(to_u8_vec_macro!(u32, &(item.base_offset as u32), &Endian::Big).iter()),
                8 => serialized.extend(to_u8_vec_macro!(u64, & item.base_offset,         &Endian::Big).iter()),
                _ => panic!("Invalid base_offset_size!")
            };

            serialized.extend(to_u8_vec_macro!(u16, &item.extent_count, &Endian::Big).iter());

            for extent in &item.extents
            {
                if 
                    (self.header.get_version() == 1 || self.header.get_version() == 2)
                    &&
                    self.index_size > 0
                {
                    match self.index_size
                    {
                        4 => {
                            let idx: u32 = extent.extent_index.expect("Extent index missing") as u32;
                            serialized.extend(to_u8_vec_macro!(u32, &idx, &Endian::Big).iter());
                        },
                        8 => {
                            let idx: u64 = extent.extent_index.expect("Extent index missing");
                            serialized.extend(to_u8_vec_macro!(u64, &idx, &Endian::Big).iter());
                        },
                        _ => panic!("Invalid index_size!")
                    }
                }

                match self.offset_size
                {
                    0 => (),
                    4 => serialized.extend(to_u8_vec_macro!(u32, &(extent.extent_offset as u32), &Endian::Big).iter()),
                    8 => serialized.extend(to_u8_vec_macro!(u64, & extent.extent_offset,         &Endian::Big).iter()),
                    _ => panic!("Invalid offset_size!")
                };

                match self.length_size
                {
                    0 => (),
                    4 => serialized.extend(to_u8_vec_macro!(u32, &(extent.extent_length as u32), &Endian::Big).iter()),
                    8 => serialized.extend(to_u8_vec_macro!(u64, & extent.extent_length,         &Endian::Big).iter()),
                    _ => panic!("Invalid length_size!")
                };
            }
        }

        return serialized;
    }


    fn as_any         (&    self) -> &    dyn std::any::Any {      self        }
    fn as_any_mut     (&mut self) -> &mut dyn std::any::Any {      self        }
    fn get_header     (&    self) -> &        BoxHeader     { &    self.header }
    fn get_header_mut (&mut self) -> &mut     BoxHeader     { &mut self.header }
}
