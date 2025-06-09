// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

use crate::general_file_io::EXIF_HEADER;
use crate::heif::box_type::BoxType;
use crate::heif::boxes::item_location::ItemLocationEntry;
use crate::heif::boxes::meta::MetaBox;
use crate::heif::read_next_box;

use crate::metadata::Metadata;
use crate::util::read_be_u32;

use super::boxes::GenericIsoBox;
use super::boxes::item_info::ItemInfoBox;
use super::boxes::item_location::ItemLocationBox;

pub struct
HeifContainer
{
    boxes: Vec<Box<dyn GenericIsoBox>>
}

impl
HeifContainer
{
    pub(super) fn
    construct_from_cursor_unboxed
    <T: Seek + Read>
    (
        cursor: &mut T,
    )
    -> Result<Self, std::io::Error>
    {
        let mut boxes = Vec::new();

        loop 
        {
            if let Ok(next_box) = read_next_box(cursor)
            {
                boxes.push(next_box);
            }
            else
            {
                break;
            }
        }

        return Ok(Self { boxes })
    }

    fn
    get_meta_box
    (
        &self
    )
    -> &MetaBox
    {
        return match self.boxes.iter()
            .find(|b| b.get_header().get_box_type() == BoxType::meta)
            .unwrap()
            .as_any()
            .downcast_ref::<MetaBox>() {
                Some(unboxed) => unboxed,
                None          => panic!("Can't unbox ItemInfoBox!")
            };
    }

    fn
    get_meta_box_mut
    (
        &mut self
    )
    -> &mut MetaBox
    {
        return match self.boxes.iter_mut()
            .find(|b| b.get_header().get_box_type() == BoxType::meta)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<MetaBox>() {
                Some(unboxed) => unboxed,
                None          => panic!("Can't unbox ItemInfoBox!")
            };
    }

    fn
    get_item_info_box
    (
        &self
    )
    -> &ItemInfoBox
    {
        return match self.get_meta_box().other_boxes.iter()
            .find(|b| b.get_header().get_box_type() == BoxType::iinf)
            .unwrap()
            .as_any()
            .downcast_ref::<ItemInfoBox>() {
                Some(unboxed) => unboxed,
                None          => panic!("Can't unbox ItemInfoBox!")
            };
    }

    fn
    get_item_id_exif_data
    (
        &self
    )
    -> u16
    {
        return self.get_item_info_box().get_exif_item_id();
    }

    fn
    get_item_location_box
    (
        &self
    )
    -> &ItemLocationBox
    {
        return match self.get_meta_box().other_boxes.iter()
            .find(|b| b.get_header().get_box_type() == BoxType::iloc)
            .unwrap()
            .as_any()
            .downcast_ref::<ItemLocationBox>() {
                Some(unboxed) => unboxed,
                None          => panic!("Can't unbox ItemLocationBox!")
            };
    }

    fn
    get_item_location_box_mut
    (
        &mut self
    )
    -> &mut ItemLocationBox
    {
        return match self.get_meta_box_mut().other_boxes.iter_mut()
            .find(|b| b.get_header().get_box_type() == BoxType::iloc)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<ItemLocationBox>() {
                Some(unboxed) => unboxed,
                None          => panic!("Can't unbox ItemLocationBox!")
            };
    }

    fn
    get_exif_item_location_entry
    (
        &self,
        exif_item_id: u16,
    )
    -> &ItemLocationEntry
    {
        return self.get_item_location_box().items.iter()
            .find(|item| item.item_id == exif_item_id as u32)
            .unwrap();
    }

    fn
    get_exif_data_pos_and_len
    (
        &self,
        exif_item_id: u16,
    )
    -> (u64, u64)
    {
        let exif_item    = self.get_exif_item_location_entry(exif_item_id);
        let exif_extents = &exif_item.extents;

        if exif_extents.len() != 1
        {
            panic!("Expected exactly one EXIF extent info entry! Please create a new ticket at https://github.com/TechnikTobi/little_exif with an example image file");
        }

        match exif_item.get_construction_method()
        {
            super::boxes::item_location::ItemConstructionMethod::FILE => {

                // Unwrap is ok here as we have previously established that 
                // this first element must exist via if exif_extents.len() != 1
                return (
                    exif_extents.first().unwrap().extent_offset,
                    exif_extents.first().unwrap().extent_length
                );
            },

            super::boxes::item_location::ItemConstructionMethod::IDAT => {
                panic!("HEIF: item constr. method 'IDAT' currently not supported. Please create a new ticket at https://github.com/TechnikTobi/little_exif with an example image file");
            },

            super::boxes::item_location::ItemConstructionMethod::ITEM => {
                panic!("HEIF: item constr. method 'ITEM' currently not supported. Please create a new ticket at https://github.com/TechnikTobi/little_exif with an example image file");
            },
        }
    }

    pub(super) fn
    get_exif_data
    <T: Seek + Read>
    (
        &self,
        cursor: &mut T,
    )
    -> Result<Vec<u8>, std::io::Error>
    {
        // Locate exif data
        let exif_item_id    = self.get_item_id_exif_data();
        let (start, length) = self.get_exif_data_pos_and_len(exif_item_id);

        // Reset cursor to start of exif data
        cursor.seek(std::io::SeekFrom::Start(start))?;

        // Read in the first 4 bytes, which gives the offset to the start
        // of the TIFF header and seek to that
        let exif_tiff_header_offset = read_be_u32(cursor)? as usize;

        cursor.seek_relative(exif_tiff_header_offset as i64)?;

        // Read in the remaining bytes
        let mut exif_buffer = vec![0u8; 
            length as usize 
            - 4                       // the 4 bytes that store the offset
            - exif_tiff_header_offset // the actual offset
        ];
        cursor.read_exact(&mut exif_buffer)?;

        // Stick a EXIF_HEADER in the front
        let mut full_exif_data = EXIF_HEADER.to_vec();
        full_exif_data.append(&mut exif_buffer);

        return Ok(full_exif_data);
    }

    /// Constructs a new version of the exif data area of the HEIF file
    /// the i64 tells us the delta in bytes. If negative, the new area is
    /// shorter than the old one, positive if longer
    fn
    construct_new_exif_data_area
    <T: Seek + Read>
    (
        &self,
        cursor:   &mut T,
        metadata: &Metadata
    )
    -> Result<(Vec<u8>, i64), std::io::Error>
    {
        // Locate old exif data
        let exif_item_id    = self.get_item_id_exif_data();
        let (start, length) = self.get_exif_data_pos_and_len(exif_item_id);

        // Reset cursor to start of exif data
        cursor.seek(std::io::SeekFrom::Start(start))?;

        // Read in all of this area
        let mut exif_buffer = vec![0u8; length as usize];
        cursor.read_exact(&mut exif_buffer)?;

        // Decode the first 4 bytes, which tells us where to cut off the old 
        // data and replace with the new one
        let mut local_cursor            = Cursor::new(exif_buffer[0..4].to_vec());
        let     exif_tiff_header_offset = read_be_u32(&mut local_cursor)?;

        // Cut off data, starting at the old TIFF header and replace with new
        let mut new_exif_buffer = exif_buffer[0..exif_tiff_header_offset as usize + 4].to_vec();
        new_exif_buffer.append(&mut metadata.encode()?);

        let delta = new_exif_buffer.len() as i64 - length as i64;

        return Ok((
            new_exif_buffer,
            delta 
        ));
    }

    pub(super) fn
    generic_write_metadata
    <T: Seek + Read + Write>
    (
        &mut self,
        cursor:   &mut T,
        metadata: &Metadata
    )
    -> Result<(), std::io::Error>
    {
        // Find out where old exif is located, needed to determine which iloc
        // entries need to be updated
        let id       = self.get_item_id_exif_data();
        let (pos, _) = self.get_exif_data_pos_and_len(id);

        // Construct new exif data area
        let (new_exif_area, delta) = self.construct_new_exif_data_area(
            cursor, 
            metadata
        )?;

        // Update the location data in the iloc box
        for item in self.get_item_location_box_mut().items.iter_mut()
        {
            for extent in item.extents.iter_mut()
            {
                if extent.extent_offset < pos
                {
                    continue;
                }
                if extent.extent_offset == pos
                {
                    // Special case where we have the extent of the exif area
                    // needs update in length, not offset
                    extent.extent_length = (extent.extent_length as i64 + delta) as u64;
                    continue;
                }
                if extent.extent_offset > pos
                {
                    extent.extent_offset = (extent.extent_offset as i64 + delta) as u64;
                }
            }
        }

        

        return Ok(());
    }



}
