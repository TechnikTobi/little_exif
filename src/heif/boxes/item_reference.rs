// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::endian::Endian;
use crate::u8conversion::U8conversion;
use crate::u8conversion::to_u8_vec_macro;
use crate::util::read_be_u16;
use crate::util::read_be_u32;

use crate::heif::box_header::BoxHeader;
use crate::heif::boxes::GenericIsoBox;
use crate::heif::boxes::ParsableIsoBox;

#[allow(non_snake_case)]
#[derive(Debug)]
pub struct
SingleItemTypeReferenceBox
{
    pub(self)  header:          BoxHeader,
    pub(self)  is_large:        bool,
    pub(crate) from_item_ID:    u32,
    pub(crate) reference_count: u16,
    pub(crate) to_item_ID:      Vec<u32>,
}

#[derive(Debug)]
pub struct
ItemReferenceBox
{
    pub(self)  header:     BoxHeader,
    pub(crate) references: Vec<SingleItemTypeReferenceBox>,
}

impl
SingleItemTypeReferenceBox
{
    #[allow(non_snake_case)]
    fn
    construct_from_cursor_unboxed
    <T: Seek + Read>
    (
        cursor:      &mut T,
        iref_header: &BoxHeader,
    )
    -> Result<Self, std::io::Error>
    {
        let     header     = BoxHeader::read_box_header(cursor)?;
        let mut to_item_ID = Vec::new();

        // Depending on the version stored in the header of the iref box,
        // the references are either 'normal' (version == 0) or "large" 
        // (version == 1), see ISO/IEC 14496-12:2015 § 8.11.12.2
        let is_large = if iref_header.get_version() == 0
        {
            false
        }
        else if iref_header.get_version() == 1
        {
            true
        }
        else
        {
            panic!("Expected either version == 0 or version == 1 for iref box! Please create a new ticket at https://github.com/TechnikTobi/little_exif with an example image file");
        };

        let from_item_ID = if is_large 
            { 
                read_be_u32(cursor)? 
            } 
            else 
            { 
                read_be_u16(cursor)? as u32 
            };

        let reference_count = read_be_u16(cursor)?;

        for _ in 0..reference_count
        {
            to_item_ID.push(
                if is_large 
                {
                    read_be_u32(cursor)?
                }
                else
                {
                    read_be_u16(cursor)? as u32
                }
            );
        }

        return Ok(SingleItemTypeReferenceBox
            {
                header,
                is_large,
                from_item_ID,
                reference_count,
                to_item_ID
            }
        );
    }
}

impl
ItemReferenceBox
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
        let mut bytes_read = 0;

        let mut references = Vec::new();

        while bytes_read < header.get_box_size() - header.get_header_size()
        {
            let next_reference = SingleItemTypeReferenceBox::construct_from_cursor_unboxed(
                cursor, 
                &header
            )?;

            bytes_read += next_reference.get_header().get_box_size();

            references.push(next_reference);
        }

        return Ok(ItemReferenceBox { header, references });
    }
}

impl
ParsableIsoBox
for
ItemReferenceBox
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
        return Ok(Box::new(ItemReferenceBox::construct_from_cursor_unboxed(
            cursor, 
            header
        )?));
    }
}



impl
GenericIsoBox
for
SingleItemTypeReferenceBox
{
    #[allow(non_snake_case)]
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();

        // from_item_ID
        if self.is_large
        {
            serialized.extend(to_u8_vec_macro!(u32, &self.from_item_ID,          &Endian::Big).iter());
        }
        else
        {
            serialized.extend(to_u8_vec_macro!(u16, &(self.from_item_ID as u16), &Endian::Big).iter());
        }

        // reference_count
        serialized.extend(to_u8_vec_macro!(u16, &self.reference_count, &Endian::Big).iter());

        // to_item_ID
        for to_item_ID_entry in &self.to_item_ID
        {
            if self.is_large
            {
                serialized.extend(to_u8_vec_macro!(u32, to_item_ID_entry,            &Endian::Big).iter());
            }
            else
            {
                serialized.extend(to_u8_vec_macro!(u16, &(*to_item_ID_entry as u16), &Endian::Big).iter());
            }
        }

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
ItemReferenceBox
{
    fn
    serialize
    (
        &self
    ) 
    -> Vec<u8>
    {
        let mut serialized = self.header.serialize();

        for reference in &self.references
        {
            serialized.extend(reference.serialize());
        }

        return serialized;
    }

    fn as_any         (&    self) -> &    dyn std::any::Any {      self        }
    fn as_any_mut     (&mut self) -> &mut dyn std::any::Any {      self        }
    fn get_header     (&    self) -> &        BoxHeader     { &    self.header }
    fn get_header_mut (&mut self) -> &mut     BoxHeader     { &mut self.header }
}
