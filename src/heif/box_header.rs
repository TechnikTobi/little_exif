// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::endian::Endian;
use crate::u8conversion::U8conversion;
use crate::u8conversion::to_u8_vec_macro;
use crate::util::read_16_bytes;
use crate::util::read_1_bytes;
use crate::util::read_3_bytes;
use crate::util::read_4_bytes;
use crate::util::read_be_u32;
use crate::util::read_be_u64;

use super::box_type::BoxType;

#[derive(Clone, Debug)]
pub struct
BoxHeader
{
    box_size:    usize,
    largesize:   bool,
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
            largesize:   false,
            box_type:    box_type.clone(),
            header_size: 8,
            version:     None,
            flags:       None,
        };

        if box_type.extends_fullbox()
        {
            header.version = Some(read_1_bytes(cursor)?[0]);
            header.flags   = Some(read_3_bytes(cursor)?);

            // Adjust header size information
            header.header_size += 4;
        }

        // Uses largesize box size
        if header.box_size == 1
        {
            header.box_size  = read_be_u64(cursor)? as usize;
            header.largesize = true;

            // Adjust header size information
            header.header_size += 8;
        }

        if let BoxType::uuid { usertype: _ } = header.box_type
        {
            let new_usertype = read_16_bytes(cursor)?;
            header.box_type = BoxType::uuid { usertype: new_usertype };

            // Adjust header size information
            header.header_size += 16;
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
    get_box_type
    (
        &self
    )
    -> BoxType
    {
        return self.box_type.clone();
    }

    pub(super) fn
    get_header_size
    (
        &self
    )
    -> usize
    {
        return self.header_size;
    }

    pub(super) fn
    get_version
    (
        &self
    )
    -> u8
    {
        return self.version.unwrap();
    }

    pub(super) fn
    serialize
    (
        &self
    )
    -> Vec<u8>
    {
        let mut serialized = Vec::new();

        // Serialize box size - Part 1
        if self.largesize
        {
            serialized.extend(to_u8_vec_macro!(u32, &1, &Endian::Big).iter());
        }
        else
        {
            serialized.extend(to_u8_vec_macro!(u32, &(self.box_size as u32), &Endian::Big).iter());
        }
        
        // Serialize box type - Part 1
        serialized.extend(self.box_type.to_4_bytes());

        // Serialize version and flags (if present)
        if self.box_type.extends_fullbox()
        {
            serialized.push(self.version.unwrap());
            for flag in self.flags.unwrap()
            {
                serialized.push(flag);
            }
        }

        // Serialize box size - Part 2
        if self.largesize
        {
            serialized.extend(to_u8_vec_macro!(u64, &(self.box_size as u64), &Endian::Big).iter());
        }

        // Serialize box type - Part 2
        if let BoxType::uuid { usertype } = self.box_type
        {
            for byte in usertype
            {
                serialized.push(byte);
            }
        }

        return serialized;
    }
}