// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Cursor;
use std::io::Read;
use std::io::Seek;

use crate::general_file_io::io_error;
use crate::general_file_io::EXIF_HEADER;
use crate::heif::box_type::BoxType;
use crate::heif::boxes::iso::IsoBox;
use crate::heif::boxes::item_info::ItemInfoEntryBox;
use crate::heif::boxes::item_location::ItemConstructionMethod;
use crate::heif::boxes::item_location::ItemLocationEntry;
use crate::heif::boxes::item_location::ItemLocationEntryExtentEntry;
use crate::heif::boxes::meta::MetaBox;
use crate::heif::read_next_box;

use crate::metadata::Metadata;
use crate::util::insert_multiple_at;
use crate::util::range_remove;
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
    get_item_info_box_mut
    (
        &mut self
    )
    -> &mut ItemInfoBox
    {
        return match self.get_meta_box_mut().other_boxes.iter_mut()
            .find(|b| b.get_header().get_box_type() == BoxType::iinf)
            .unwrap()
            .as_any_mut()
            .downcast_mut::<ItemInfoBox>() {
                Some(unboxed) => unboxed,
                None          => panic!("Can't unbox ItemInfoBox!")
            };
    }

    fn
    get_item_id_exif_data
    (
        &self
    )
    -> Result<u16, std::io::Error>
    {
        if let Some(item) = self.get_item_info_box().get_exif_item()
        {
            return Ok(item.item_id);
        }

        return io_error!(Other, "No EXIF item found!");
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
                    exif_extents.first().unwrap().extent_offset + exif_item.base_offset,
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
        let exif_item_id    = self.get_item_id_exif_data()?;
        let (start, length) = self.get_exif_data_pos_and_len(exif_item_id);

        // Reset cursor to start of exif data
        cursor.seek(std::io::SeekFrom::Start(start))?;

        // Read in the first 4 bytes, which gives the offset to the start
        // of the TIFF header and seek to that
        let exif_tiff_header_offset = read_be_u32(cursor)? as usize;

        cursor.seek(std::io::SeekFrom::Current(exif_tiff_header_offset as i64))?;

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
        metadata: &Metadata,
    )
    -> Result<(Vec<u8>, i64), std::io::Error>
    {
        let mut new_exif_buffer;
        let delta;
        // Locate old exif data
        if let Ok(exif_item_id)    = self.get_item_id_exif_data() {
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
            new_exif_buffer = exif_buffer[0..exif_tiff_header_offset as usize + 4].to_vec();

            if !metadata.get_ifds().is_empty()
            {
                new_exif_buffer.append(&mut metadata.encode()?);
            }
            delta = new_exif_buffer.len() as i64 - length as i64;
        } else {
            // Create a new exif header, starting with an empty TIFF header.
            new_exif_buffer = 0_u32.to_be_bytes().to_vec();

            if !metadata.get_ifds().is_empty()
            {
                new_exif_buffer.append(&mut metadata.encode()?);
            }
            delta = new_exif_buffer.len() as i64;
        }

        return Ok((
            new_exif_buffer,
            delta 
        ));
    }

    pub(super) fn
    generic_write_metadata
    (
        &mut self,
        file_buffer: &mut Vec<u8>,
        metadata:    &    Metadata
    )
    -> Result<(), std::io::Error>
    {
        // Find out where old exif is located, needed to determine which iloc
        // entries need to be updated
        let id                           = self.get_item_id_exif_data();
        let (old_exif_pos, old_exif_len) = id.as_ref()
            .map(|id| self.get_exif_data_pos_and_len(*id))
            .unwrap_or((0, 0));

        let mut cursor = Cursor::new(file_buffer);

        // Construct new exif data area
        let (mut new_exif_area, delta) = self.construct_new_exif_data_area(
            &mut cursor, 
            metadata
        )?;

        // Update the location data in the iloc box, or insert the new box
        if id.is_ok()
        {
            for item in self.get_item_location_box_mut().items.iter_mut()
            {
                // First, check if any extent of this item has the same offset as
                // the old exif data area. In that case, there must be only one
                // extent - other cases can't be handled right now
                if item.extents.iter()
                    .any(|extent| {
                        item.base_offset + extent.extent_offset == old_exif_pos
                    })
                {
                    if item.extents.len() != 1
                    {
                        panic!("Expect to have exactly one extent info for EXIF!");
                    }

                    // In case of the EXIF extent information we need to update
                    // the length information, not the offset!
                    let new_ext_len = (
                        item.extents.first().unwrap().extent_length as i64
                        + delta
                    ) as u64;
                    item.extents.first_mut().unwrap().extent_length = new_ext_len;

                    continue;
                }

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

                if item.base_offset > delta.unsigned_abs()
                {
                    // Potentially modify the entire base offset 
                    // however, we can only do that if all complete offsets
                    // point to an area after the exif data area
                    // So we need to check that first:
                    if item.extents.iter()
                        .all(|extent| {
                            item.base_offset + extent.extent_offset >= old_exif_pos
                        })
                    {
                        item.base_offset = (item.base_offset as i64 + delta) as u64;
                        continue;
                    }
                }

                // At this point we have no option left but to modify all 
                // individual extent offsets
                for extent in item.extents.iter_mut()
                {
                    let complete_offset = item.base_offset + extent.extent_offset;

                    if complete_offset > old_exif_pos
                    {
                        extent.extent_offset = (extent.extent_offset as i64 + delta) as u64;
                    }
                }
            }
        }
        else
        {
            // In this case, there is no existing metadata. We need to create item location
            // and metadata entries, and append the new exif box into the data. Due to the
            // layout of the container format, this also requires updating sizes and offsets
            // that in some cases are dependent on the new entries we are creating.
            let old_largest_id = self.get_item_location_box()
                .items
                .iter()
                .map(|x| x.item_id)
                .max()
                .unwrap_or(0);

            // Update location index with the new entry, and fix its metadata
            let iloc = self.get_item_location_box_mut();
            if iloc.base_offset_size == 0
            {
                iloc.base_offset_size = 4;
            }
            iloc.items.push(ItemLocationEntry {
                item_id: old_largest_id + 1,
                reserved_and_construction_method: 0,
                data_reference_index: 0,
                // this is dependent on the size of the entries we are in the
                // process of creating; this will have to be computed later
                base_offset: 0,
                extent_count: 1,
                extents: vec![ItemLocationEntryExtentEntry {
                    extent_index: None,
                    extent_offset: 0,
                    extent_length: delta.unsigned_abs(),
                }]
            });
            iloc.item_count += 1;
            let new_box_size = iloc.serialize().len();
            iloc.get_header_mut().set_box_size(new_box_size);

            // Add the new item info entries, and fix up the iinf metadata
            let iinf = self.get_item_info_box_mut();
            iinf.item_count += 1;
            iinf.items.push(ItemInfoEntryBox::new_exif_info_entry_box((old_largest_id + 1) as u16));
            let new_box_size = iinf.serialize().len();
            iinf.get_header_mut().set_box_size(new_box_size);

            // Fix up the size of the meta box, since the iloc and iinf boxes are inside it
            let new_box_size = self.get_meta_box().serialize().len();
            self.get_meta_box_mut().get_header_mut().set_box_size(new_box_size);

            // Append the new exif area to the mdat box
            let mdat = match self.boxes.iter_mut()
                .find(|b| b.get_header().get_box_type() == BoxType::mdat)
                .unwrap()
                .as_any_mut()
                .downcast_mut::<IsoBox>() {
                    Some(unboxed) => unboxed,
                    None          => panic!("Can't unbox mdat IsoBox!")
                };
            mdat.append_data(&mut new_exif_area);

            // Now that the new data is inserted, calculate the new offsets and correct them in iloc
            self.fix_iloc_offsets();
        }

        // Now we clear the vec and write the boxes to it
        // Keep track of how many bytes were written so we know when to 
        // replace old exif data with new
        cursor.get_mut().clear();

        let mut written_bytes    = 0usize;
        let mut new_exif_written  = false;
        let     end_of_old_exif  = (old_exif_pos + old_exif_len) as usize;

        for iso_box in &mut self.boxes
        {
            let mut serialized = iso_box.serialize();

            // If this box encompasses the exif data area, update its size and
            // serialize it again
            // TODO: As this is not the cleanest approach (e.g. what if the
            // exif area is not in this top level box but some nested box? 
            // -> requires update of size fields of all boxes "downward") some
            // other solution needs to be found for this
            // In the meantime, this should work for the majority of HEIFs
            if 
                written_bytes + serialized.len() >= end_of_old_exif 
                && 
                !new_exif_written
                &&
                id.is_ok()
            {
                let new_size = (iso_box.get_header().get_box_size() as i64 + delta) as usize;
                iso_box.get_header_mut().set_box_size(new_size);
                serialized = iso_box.serialize();

                // Write the serialized box with the OLD exif data
                cursor.get_mut().extend(&serialized);

                // Remove old exif data
                range_remove(
                    cursor.get_mut(), 
                    old_exif_pos as usize, 
                    (old_exif_pos + old_exif_len) as usize
                );

                // Insert new exif data
                insert_multiple_at(
                    cursor.get_mut(),
                    old_exif_pos as usize, 
                    &mut new_exif_area
                );

                new_exif_written = true;
            }
            else
            {
                // Just extend with the serialized box contents
                cursor.get_mut().extend(&serialized);
            }

            written_bytes = written_bytes + serialized.len();
        }

        return Ok(());
    }

    /// Recalculates the offsets inside the iloc box based on the
    /// size of each box and extent lengths.
    fn
    fix_iloc_offsets
    (
        &mut self
    )
    {
        let mut mdat_data_start: u64= 0;
        for bx in self.boxes.iter()
        {
            if bx.get_header().get_box_type() == BoxType::mdat
            {
                // This offset should include the mdat header - we want the
                // position of the start of the data section.
                mdat_data_start += bx.get_header().get_header_size() as u64;
                break;
            }
            mdat_data_start += bx.get_header().get_box_size() as u64;
        }

        let mut base_offset = mdat_data_start;
        for ile in self.get_item_location_box_mut().items.iter_mut()
        {
            ile.base_offset = base_offset;
            let mut extent_offset = 0;
            for ext in ile.extents.iter_mut()
            {
                ext.extent_offset = extent_offset;
                extent_offset += ext.extent_length;
            }
            base_offset += extent_offset;
        }
    }

    pub(super) fn 
    generic_clear_metadata
    (
        &mut self,
        file_buffer: &mut Vec<u8>,
    )
    -> Result<(), std::io::Error>
    {
        // Instead of truly clearing the metadata, just write an empty 
        // exif data area
        // Based on what the macOS shortcut is doing, only keeps the tags
        // 0x0112: Orientation
        // 0x011A: XResolution
        // 0x011B: YResolution
        // 0x0128: ResolutionUnit

        // Note: It is up for debate whether keeping this information is wanted
        // or not/this should write a truly empty exif area

        // Create cursor
        let mut cursor = Cursor::new(file_buffer);

        // Read original metadata
        let orig_metadata = Metadata::general_decoding_wrapper(
            self.get_exif_data(&mut cursor)
        )?;

        // Construct new metadata that only contains the above tags
        let mut new_metadata = Metadata::new();

        // 0x0112: Orientation
        if let Some(tag) = orig_metadata.get_tag_by_hex(0x0112, None).next()
        {
            new_metadata.set_tag(tag.clone());
        }

        // 0x011A: XResolution
        if let Some(tag) = orig_metadata.get_tag_by_hex(0x011A, None).next()
        {
            new_metadata.set_tag(tag.clone());
        }

        // 0x011A: YResolution
        if let Some(tag) = orig_metadata.get_tag_by_hex(0x011B, None).next()
        {
            new_metadata.set_tag(tag.clone());
        }

        // 0x0128: ResolutionUnit
        if let Some(tag) = orig_metadata.get_tag_by_hex(0x0128, None).next()
        {
            new_metadata.set_tag(tag.clone());
        }

        return self.generic_write_metadata(cursor.get_mut(), &new_metadata);
    }
}