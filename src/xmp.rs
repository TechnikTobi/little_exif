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
pub(crate) fn remove_exif_from_xmp(data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_reader(data);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    // Needed by the reader
    let mut read_buffer = Vec::new();

    // Needed for skipping stuff like
    // <exif:Description>Hi</exif:Description>\n
    let mut skip_depth = 0u32;
    let mut skip_next_nl = false;

    loop {
        // Read in the event
        let read_event = reader.read_event_into(&mut read_buffer);

        match read_event {
            Ok(Event::Start(ref event)) => {
                let event_name = String::from_utf8(event.name().0.to_vec())?;

                if event_name.starts_with("exif:") {
                    skip_depth += 1;
                } else if skip_depth == 0 {
                    writer.write_event(Event::Start(get_exif_filtered_event(event)?))?;
                }
            }

            Ok(Event::Empty(ref event)) => {
                let event_name = String::from_utf8(event.name().0.to_vec())?;

                if event_name.starts_with("exif:") {
                    // do nothing
                } else if skip_depth == 0 {
                    writer.write_event(Event::Empty(get_exif_filtered_event(event)?))?;
                }
            }

            Ok(Event::End(ref event)) => {
                if skip_depth > 0 {
                    skip_depth -= 1;
                    skip_next_nl = true;
                } else {
                    writer.write_event(Event::End(event.clone()))?;
                }
            }

            Ok(Event::Eof) => {
                assert_eq!(skip_depth, 0);
                break;
            }

            Ok(Event::Text(ref event)) => {
                let event_string = String::from_utf8(event.to_vec())?;

                let characters = event_string
                    .chars()
                    .filter(|c| *c == '\n' || !c.is_whitespace())
                    .collect::<Vec<char>>();

                if characters == vec!['\n'] && skip_next_nl {
                    skip_next_nl = false;
                } else if skip_depth == 0 {
                    writer.write_event(Event::Text(event.clone()))?;
                }
            }

            Ok(other_event) => {
                if skip_depth == 0 {
                    writer.write_event(other_event)?;
                }
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

fn get_exif_filtered_event<'a>(
    event: &'a BytesStart<'a>,
) -> Result<BytesStart<'a>, Box<dyn std::error::Error>> {
    let mut new_event = BytesStart::new(std::str::from_utf8(event.name().0)?);

    new_event.extend_attributes(
        event
            .attributes()
            .filter_map(Result::ok)
            .filter(|attribute| {
                if let Ok(key) = std::str::from_utf8(attribute.key.as_ref()) {
                    !key.starts_with("exif:")
                } else {
                    true
                }
            }),
    );

    return Ok(new_event);
}
