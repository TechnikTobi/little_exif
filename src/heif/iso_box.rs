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
BaseBoxHeader
{
    box_size:    usize,
    box_type:    BoxType,
    header_size: usize,
}

pub struct
ExtendedBoxHeader
{
    base:    BaseBoxHeader,
    version: u8,
    flags:   u32,
}