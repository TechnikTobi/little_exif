// Copyright © 2025 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use log::error;
use quick_xml::events::BytesStart;
use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::Writer;
use std::io::Cursor;

/// Some images also contain XMP metadata, which in turn may include EXIF data
/// that is simply a duplicate from e.g. the eXIf chunk in a PNG.
/// This function takes in the raw XMP information and removes EXIF attributes,
/// while maintaining other XMP information so that the result can be 
/// written back to the image data.
pub(crate) fn
remove_exif_from_xmp
(
    data: &[u8]
)
-> Result<Vec<u8>, Box<dyn std::error::Error>>
{
    let mut reader = Reader::from_reader(data);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Not really needed, but maybe useful in the future
    let mut read_buffer = Vec::new();

    loop 
    {
        // Read in the event
        let read_event = reader.read_event_into(&mut read_buffer);

        match read_event
        {
            Ok(Event::Start(ref event)) => {
                writer.write_event(Event::Start(get_exif_filtered_event(event)?))?;
            }

            Ok(Event::Empty(ref event)) => {
                writer.write_event(Event::Empty(get_exif_filtered_event(event)?))?;
            }

            Ok(Event::Eof) => {
                break;
            }

            Ok(other_event) => {
                writer.write_event(other_event)?;
            }

            Err(error_message) => {
                error!(
                    "Error at position {}: {:?}", 
                    reader.buffer_position(), 
                    error_message
                );
                break;
            }
        };

        read_buffer.clear();
    }

    return Ok(writer.into_inner().into_inner());
}

fn
get_exif_filtered_event<'a>
(
    event: &'a BytesStart<'a>
)
-> Result<BytesStart<'a>, Box<dyn std::error::Error>>
{
    let mut new_event = BytesStart::new(
        std::str::from_utf8(event.name().0)?
    );

    new_event.extend_attributes(
        event.attributes()
            .filter_map(Result::ok)
            .filter(|attribute| 
                {
                    if let Ok(key) = std::str::from_utf8(
                        attribute.key.as_ref()
                    ) 
                    {
                        !key.starts_with("exif:")
                    } else {
                        true
                    }
                }
            ),
    );

    return Ok(new_event);
}