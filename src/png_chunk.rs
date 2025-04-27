// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

#[allow(non_camel_case_types, dead_code)]
pub(crate) enum
PngChunkOrdering
{
    FIRST,
    BEFORE_IDAT,
    BEFORE_PLTE_AND_IDAT,
    AFTER_PLTE_BEFORE_IDAT,
    LAST,
    NONE
}

/// This macro builds the enum for the different type of PNG chunks
macro_rules! build_png_chunk_type_enum {
    (
        $( (
            $tag:ident,
            $critical:expr,
            $multiple:expr,
            $ordering:ident
        ) ),*
    )
    =>
    {
        /// These are the different PNG chunk types currently known to
        /// little_exif. These might be expanded in the future if necessary.
        #[allow(non_camel_case_types)]
        pub(crate) enum
        PngChunk
        {
            UNKNOWN(String, u32),
            $(
                $tag(u32),
            )*
        }

        impl PngChunk
        {
            pub(crate) fn
            length
            (
                &self
            )
            -> u32
            {
                match *self
                {
                    PngChunk::UNKNOWN(_, length) => length,
                    $(
                        PngChunk::$tag(  length) => length,
                    )*
                }
            }

            pub(crate) fn
            as_string
            (
                &self
            )
            -> String
            {
                match self
                {
                    PngChunk::UNKNOWN(name, _) => name.clone(),
                    $(
                        PngChunk::$tag(_) => String::from(stringify!($tag)),
                    )*
                }
            }

            pub(crate) fn
            from_string
            (
                string_name: &String,
                length: u32
            )
            -> Result<PngChunk, PngChunk>
            {
                match &(*string_name.as_str())
                {
                    $(
                        stringify!($tag) => Ok(PngChunk::$tag(length)),
                    )*
                    _ => Err(PngChunk::UNKNOWN(string_name.clone(), length)),
                }
            }
        }
    }
}

build_png_chunk_type_enum![
    // Tag  Critical    Multiple    Ordering
    (IHDR,  true,       false,      FIRST),
    (PLTE,  true,       false,      BEFORE_IDAT),
    (IDAT,  true,       true,       NONE),
    (IEND,  true,       false,      LAST),

    (cHRM,  false,      false,      BEFORE_PLTE_AND_IDAT),
    (gAMA,  false,      false,      BEFORE_PLTE_AND_IDAT),
    (iCCP,  false,      false,      BEFORE_PLTE_AND_IDAT),
    (cICP,  false,      false,      BEFORE_PLTE_AND_IDAT),
    (sBIT,  false,      false,      BEFORE_PLTE_AND_IDAT),
    (sRGB,  false,      false,      BEFORE_PLTE_AND_IDAT),

    (bKGD,  false,      false,      AFTER_PLTE_BEFORE_IDAT),
    (hIST,  false,      false,      AFTER_PLTE_BEFORE_IDAT),
    (tRNS,  false,      false,      AFTER_PLTE_BEFORE_IDAT),

    (pHYs,  false,      false,      BEFORE_IDAT),
    (sPLT,  false,      true,       BEFORE_IDAT),

    (eXIf,  false,      false,      NONE), // not sure if ordering is correct
    (tIME,  false,      false,      NONE),
    (iTXt,  false,      true,       NONE),
    (tEXt,  false,      true,       NONE),
    (vpAg,  false,      false,      NONE),
    (zTXt,  false,      true,       NONE)
];
