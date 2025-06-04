// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

pub enum
BoxType
{
    FTYP,
    META,
    UNKNOWN { box_type: String }
}

pub struct
BoxHeader
{
    box_size:    usize,
    box_type:    BoxType,
    header_size: usize,
    version:     Option<u8>,
    flags:       Option<u32>,
}

pub struct
IsoBox
{
    header:    BoxHeader,
    sub_boxes: Option<Vec<IsoBox>>,
    data:      Vec<u8>,
}

// Examples:
// - infe
// 00000015: size of 0x15 bytes (including the 0x04 bytes of the size field itself) 
// 696E6665: byte representation of `infe` 
// 02:       version 2
// 00000100 190000 
// 6876633100: null terminated string of "hvc1"

impl
IsoBox
{
    pub(super) fn
    parse
    (
        data: &[u8]
    )
    {

    }
}