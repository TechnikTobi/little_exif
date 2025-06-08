// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, PartialEq)]
pub enum
BoxType
{
    ftyp,
    meta,
    hdlr,
    dinf,
    pitm,
    iinf,
    infe,
    iloc,
    iref,
    iprp,
    ipco,
    ipma,
    mdat,
    idat,
    uuid    { usertype: [u8; 16] },
    unknown { box_type: String }
}

impl
BoxType
{
    pub(super) fn
    from_4_bytes
    (
        bytes: [u8; 4]
    )
    -> BoxType
    {
        let box_type_str = std::str::from_utf8(&bytes).unwrap_or("");
        match box_type_str
        {
            "ftyp" => BoxType::ftyp,
            "meta" => BoxType::meta, 
            "hdlr" => BoxType::hdlr, 
            "dinf" => BoxType::dinf,
            "pitm" => BoxType::pitm, 
            "iinf" => BoxType::iinf, 
            "infe" => BoxType::infe,
            "iloc" => BoxType::iloc, 
            "iref" => BoxType::iref, 
            "iprp" => BoxType::iprp, 
            "ipco" => BoxType::ipco, 
            "ipma" => BoxType::ipma, 
            "mdat" => BoxType::mdat, 
            "idat" => BoxType::idat, 
            "uuid" => BoxType::uuid { usertype: [0u8; 16] },
            _      => panic!("Unknown Box Type! {:?}", box_type_str),
        }
    }

    pub(super) fn
    extends_fullbox
    (
        &self
    )
    -> bool
    {
        match self
        {
            BoxType::meta |
            BoxType::hdlr |
            BoxType::iinf |
            BoxType::infe |
            BoxType::iloc
            => true,

            _ 
            => false,
        }
    }

    pub(super) fn 
    is_container_type
    (
        &self
    )
    -> bool
    {
        match self
        {
            BoxType::meta | 
            BoxType::iinf
            => true,

            // All other known box types don't contain other boxes, only data
            _ 
            => false
        }
    }
}