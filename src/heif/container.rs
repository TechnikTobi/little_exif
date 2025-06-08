// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::heif::box_type::BoxType;
use crate::heif::boxes::item_location::ItemLocationEntry;
use crate::heif::boxes::meta::MetaBox;
use crate::heif::read_next_box;

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

        let mut exif_buffer = vec![0u8; length as usize];
        cursor.read_exact(&mut exif_buffer)?;

        return Ok(exif_buffer);
    }

}
