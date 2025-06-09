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
    pdin,
    mvhd,
    tkhd,
    mdhd,
    nmhd,
    elng,
    stsd,
    stdp,
    stts,
    ctts,
    cslg,
    stss,
    stsh,
    sdtp,
    elst,
    url ,
    urn ,
    dref,
    stsz,
    stz2,
    stsc,
    stco,
    co64,
    padb,
    subs,
    saiz,
    saio,
    mehd,
    trex,
    mfhd,
    tfhd,
    trun,
    tfra,
    mfro,
    tfdt,
    leva,
    trep,
    assp,
    sbgp,
    sgpd,
    cprt,
    tsel,
    kind,
    xml ,
    bxml,
    ipro,
    mere,
    schm,
    fiin,
    fpar,
    fecr,
    gitn,
    fire,
    stri,
    stsg,
    stvi,
    sidx,
    ssix,
    prft,
    srpp,
    vmhd,
    smhd,
    srat,
    chnl,
    dmix,
    ludt,
    txtC,
    uri ,
    uriI,
    hmhd,
    sthd,
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
            "pdin" => BoxType::pdin,
            "mvhd" => BoxType::mvhd,
            "tkhd" => BoxType::tkhd,
            "mdhd" => BoxType::mdhd,
            "nmhd" => BoxType::nmhd,
            "elng" => BoxType::elng,
            "stsd" => BoxType::stsd,
            "stdp" => BoxType::stdp,
            "stts" => BoxType::stts,
            "ctts" => BoxType::ctts,
            "cslg" => BoxType::cslg,
            "stss" => BoxType::stss,
            "stsh" => BoxType::stsh,
            "sdtp" => BoxType::sdtp,
            "elst" => BoxType::elst,
            "url " => BoxType::url ,
            "urn " => BoxType::urn ,
            "dref" => BoxType::dref,
            "stsz" => BoxType::stsz,
            "stz2" => BoxType::stz2,
            "stsc" => BoxType::stsc,
            "stco" => BoxType::stco,
            "co64" => BoxType::co64,
            "padb" => BoxType::padb,
            "subs" => BoxType::subs,
            "saiz" => BoxType::saiz,
            "saio" => BoxType::saio,
            "mehd" => BoxType::mehd,
            "trex" => BoxType::trex,
            "mfhd" => BoxType::mfhd,
            "tfhd" => BoxType::tfhd,
            "trun" => BoxType::trun,
            "tfra" => BoxType::tfra,
            "mfro" => BoxType::mfro,
            "tfdt" => BoxType::tfdt,
            "leva" => BoxType::leva,
            "trep" => BoxType::trep,
            "assp" => BoxType::assp,
            "sbgp" => BoxType::sbgp,
            "sgpd" => BoxType::sgpd,
            "cprt" => BoxType::cprt,
            "tsel" => BoxType::tsel,
            "kind" => BoxType::kind,
            "xml " => BoxType::xml ,
            "bxml" => BoxType::bxml,
            "ipro" => BoxType::ipro,
            "mere" => BoxType::mere,
            "schm" => BoxType::schm,
            "fiin" => BoxType::fiin,
            "fpar" => BoxType::fpar,
            "fecr" => BoxType::fecr,
            "gitn" => BoxType::gitn,
            "fire" => BoxType::fire,
            "stri" => BoxType::stri,
            "stsg" => BoxType::stsg,
            "stvi" => BoxType::stvi,
            "sidx" => BoxType::sidx,
            "ssix" => BoxType::ssix,
            "prft" => BoxType::prft,
            "srpp" => BoxType::srpp,
            "vmhd" => BoxType::vmhd,
            "smhd" => BoxType::smhd,
            "srat" => BoxType::srat,
            "chnl" => BoxType::chnl,
            "dmix" => BoxType::dmix,
            "ludt" => BoxType::ludt,
            "txtC" => BoxType::txtC,
            "uri " => BoxType::uri ,
            "uriI" => BoxType::uriI,
            "hmhd" => BoxType::hmhd,
            "sthd" => BoxType::sthd,
            "uuid" => BoxType::uuid { usertype: [0u8; 16] },
            _      => BoxType::unknown { box_type: String::from(box_type_str) }
        }
    }

    pub(super) fn
    to_4_bytes
    (
        &self
    )
    -> Vec<u8>
    {
        match self
        {
            BoxType::ftyp => "ftyp",
            BoxType::meta => "meta", 
            BoxType::hdlr => "hdlr", 
            BoxType::dinf => "dinf",
            BoxType::pitm => "pitm", 
            BoxType::iinf => "iinf", 
            BoxType::infe => "infe",
            BoxType::iloc => "iloc", 
            BoxType::iref => "iref", 
            BoxType::iprp => "iprp", 
            BoxType::ipco => "ipco", 
            BoxType::ipma => "ipma", 
            BoxType::mdat => "mdat", 
            BoxType::idat => "idat", 
            BoxType::pdin => "pdin",
            BoxType::mvhd => "mvhd",
            BoxType::tkhd => "tkhd",
            BoxType::mdhd => "mdhd",
            BoxType::nmhd => "nmhd",
            BoxType::elng => "elng",
            BoxType::stsd => "stsd",
            BoxType::stdp => "stdp",
            BoxType::stts => "stts",
            BoxType::ctts => "ctts",
            BoxType::cslg => "cslg",
            BoxType::stss => "stss",
            BoxType::stsh => "stsh",
            BoxType::sdtp => "sdtp",
            BoxType::elst => "elst",
            BoxType::url  => "url ",
            BoxType::urn  => "urn ",
            BoxType::dref => "dref",
            BoxType::stsz => "stsz",
            BoxType::stz2 => "stz2",
            BoxType::stsc => "stsc",
            BoxType::stco => "stco",
            BoxType::co64 => "co64",
            BoxType::padb => "padb",
            BoxType::subs => "subs",
            BoxType::saiz => "saiz",
            BoxType::saio => "saio",
            BoxType::mehd => "mehd",
            BoxType::trex => "trex",
            BoxType::mfhd => "mfhd",
            BoxType::tfhd => "tfhd",
            BoxType::trun => "trun",
            BoxType::tfra => "tfra",
            BoxType::mfro => "mfro",
            BoxType::tfdt => "tfdt",
            BoxType::leva => "leva",
            BoxType::trep => "trep",
            BoxType::assp => "assp",
            BoxType::sbgp => "sbgp",
            BoxType::sgpd => "sgpd",
            BoxType::cprt => "cprt",
            BoxType::tsel => "tsel",
            BoxType::kind => "kind",
            BoxType::xml  => "xml ",
            BoxType::bxml => "bxml",
            BoxType::ipro => "ipro",
            BoxType::mere => "mere",
            BoxType::schm => "schm",
            BoxType::fiin => "fiin",
            BoxType::fpar => "fpar",
            BoxType::fecr => "fecr",
            BoxType::gitn => "gitn",
            BoxType::fire => "fire",
            BoxType::stri => "stri",
            BoxType::stsg => "stsg",
            BoxType::stvi => "stvi",
            BoxType::sidx => "sidx",
            BoxType::ssix => "ssix",
            BoxType::prft => "prft",
            BoxType::srpp => "srpp",
            BoxType::vmhd => "vmhd",
            BoxType::smhd => "smhd",
            BoxType::srat => "srat",
            BoxType::chnl => "chnl",
            BoxType::dmix => "dmix",
            BoxType::ludt => "ludt",
            BoxType::txtC => "txtC",
            BoxType::uri  => "uri ",
            BoxType::uriI => "uriI",
            BoxType::hmhd => "hmhd",
            BoxType::sthd => "sthd",
            BoxType::uuid { usertype: _ } => "uuid",
            BoxType::unknown { box_type } => box_type
        }.as_bytes().to_vec()
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
            BoxType::iloc |
            BoxType::pitm |
            BoxType::iref |
            BoxType::pdin |
            BoxType::mvhd |
            BoxType::tkhd |
            BoxType::mdhd |
            BoxType::nmhd |
            BoxType::elng |
            BoxType::stsd |
            BoxType::stdp |
            BoxType::stts |
            BoxType::ctts |
            BoxType::cslg |
            BoxType::stss |
            BoxType::stsh |
            BoxType::sdtp |
            BoxType::elst |
            BoxType::url  |
            BoxType::urn  |
            BoxType::dref |
            BoxType::stsz |
            BoxType::stz2 |
            BoxType::stsc |
            BoxType::stco |
            BoxType::co64 |
            BoxType::padb |
            BoxType::subs |
            BoxType::saiz |
            BoxType::saio |
            BoxType::mehd |
            BoxType::trex |
            BoxType::mfhd |
            BoxType::tfhd |
            BoxType::trun |
            BoxType::tfra |
            BoxType::mfro |
            BoxType::tfdt |
            BoxType::leva |
            BoxType::trep |
            BoxType::assp |
            BoxType::sbgp |
            BoxType::sgpd |
            BoxType::cprt |
            BoxType::tsel |
            BoxType::kind |
            BoxType::xml  |
            BoxType::bxml |
            BoxType::ipro |
            BoxType::mere |
            BoxType::schm |
            BoxType::fiin |
            BoxType::fpar |
            BoxType::fecr |
            BoxType::gitn |
            BoxType::fire |
            BoxType::stri |
            BoxType::stsg |
            BoxType::stvi |
            BoxType::sidx |
            BoxType::ssix |
            BoxType::prft |
            BoxType::srpp |
            BoxType::vmhd |
            BoxType::smhd |
            BoxType::srat |
            BoxType::chnl |
            BoxType::dmix |
            BoxType::ludt |
            BoxType::txtC |
            BoxType::uri  |
            BoxType::uriI |
            BoxType::hmhd |
            BoxType::sthd 
            => true,

            _ 
            => false,
        }
    }
}