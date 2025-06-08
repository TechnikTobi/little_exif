// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

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

    pub(super) fn
    generic_write_metadata
    <T: Seek + Write>
    (
        cursor:   &mut T,
        metadata: &Metadata
    )
    -> Result<(), std::io::Error>
    {
        // Encode new metadata
        let encoded_metadata = metadata.encode()?;

        // Determine delta in byte length
        println!("{:?}", encoded_metadata);
        
        todo!();

        // Does *not* call generic_clear_metadata, as the entire tiff data gets
        // overwritten anyways
        cursor.write_all(&metadata.encode()?)?;

        return Ok(());
    }



}
