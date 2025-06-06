// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::util::read_1_bytes;
use crate::util::read_3_bytes;
use crate::util::read_4_bytes;
use crate::util::read_be_u32;
use crate::util::read_be_u64;

use super::box_type::BoxType;

#[derive(Debug)]
pub struct
BoxHeader
{
    box_size:    usize,
    box_type:    BoxType,
    header_size: usize,           // not sure if needed
    version:     Option<u8>,      // only if box type uses full headers
    flags:       Option<[u8; 3]>, // only if box type uses full headers
}

impl
BoxHeader
{
    pub(super) fn
    read_box_header
    <T: Seek + Read>
    (
        cursor: &mut T
    )
    -> Result<Self, std::io::Error>
    {
        // Read in the size
        let box_size = read_be_u32(cursor)?;

        // Read in the box type
        let box_type = BoxType::from_4_bytes(read_4_bytes(cursor)?);

        let mut header = Self {
            box_size:    box_size as usize,
            box_type:    box_type.clone(),
            header_size: 0,
            version:     None,
            flags:       None,
        };

        if box_type.extends_fullbox()
        {
            header.version = Some(read_1_bytes(cursor)?[0]);
            header.flags   = Some(read_3_bytes(cursor)?);
        }

        // Uses largesize box size
        if header.box_size == 1
        {
            header.box_size = read_be_u64(cursor)? as usize;
        }

        return Ok(header);
    }

    pub(super) fn
    get_box_size
    (
        &self
    )
    -> usize
    {
        return self.box_size;
    }

    pub(super) fn
    get_header_size
    (
        &self
    )
    -> usize
    {
        if self.version.is_none() && self.flags.is_none()
        {
            return 8;
        }

        if self.version.is_some() && self.flags.is_some()
        {
            return 12;
        }

        panic!("This should not happen!");
    }
}


fn
get_next_box
<T: Seek + Read>
(
    cursor: &mut T
)
-> Result<Box<dyn GenericIsoBox>, std::io::Error>
{
    // Read header
    let header = BoxHeader::read_box_header(cursor)?;

    println!("{:?}: {}", header.box_type, header.box_size);

    todo!()
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
    item_count: u16,
}


pub trait
GenericIsoBox
{

}

