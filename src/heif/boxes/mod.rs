// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use std::io::Read;
use std::io::Seek;

use crate::debug_println;

use super::box_type::BoxType;
use super::box_header::BoxHeader;

pub(super) mod iso;
pub(super) mod meta;
pub(super) mod item_info;
pub(super) mod item_location;

use iso::IsoBox;
use meta::MetaBox;
use item_info::ItemInfoBox;
use item_location::ItemLocationBox;

#[allow(dead_code)]
pub trait 
GenericIsoBox 
{
    fn as_any         (&    self) -> &    dyn std::any::Any;
    fn as_any_mut     (&mut self) -> &mut dyn std::any::Any;
    fn get_header     (&    self) -> &        BoxHeader;
    fn get_header_mut (&mut self) -> &mut     BoxHeader;
    fn serialize      (&    self) ->          Vec<u8>;
}

pub trait
ParsableIsoBox: GenericIsoBox
{
    fn
    construct_from_cursor
    <T: Seek + Read>
    (
        cursor: &mut T,
        header:  BoxHeader
    )
    -> Result<Box<dyn GenericIsoBox>, std::io::Error>;
}



pub(super) fn
read_box_based_on_header
<T: Seek + Read>
(
    cursor: &mut T,
    header:  BoxHeader
)
-> Result<Box<dyn GenericIsoBox>, std::io::Error>
{
    return match header.get_box_type()
    {
        BoxType::meta => MetaBox::        construct_from_cursor(cursor, header),
        BoxType::iinf => ItemInfoBox::    construct_from_cursor(cursor, header),
        BoxType::iloc => ItemLocationBox::construct_from_cursor(cursor, header),
        _             => IsoBox::         construct_from_cursor(cursor, header)
    };
}

pub(super) fn
read_next_box
<T: Seek + Read>
(
    cursor: &mut T,
)
-> Result<Box<dyn GenericIsoBox>, std::io::Error>
{
    let header = BoxHeader::read_box_header(cursor)?;

    debug_println!("{:?}", header);

    return read_box_based_on_header(cursor, header);
}