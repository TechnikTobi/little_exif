// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;

use super::Metadata;

impl<'a> IntoIterator for &'a Metadata {
    type Item = &'a ExifTag;
    type IntoIter = MetadataIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        MetadataIterator {
            metadata: self,
            current_ifd_index: 0,
            current_tag_index: 0,
        }
    }
}

pub struct MetadataIterator<'a> {
    metadata: &'a Metadata,
    current_ifd_index: usize,
    current_tag_index: usize,
}

impl<'a> Iterator for MetadataIterator<'a> {
    type Item = &'a ExifTag;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current_ifd_index < self.metadata.image_file_directories.len() {
            if self.current_tag_index
                < self.metadata.image_file_directories[self.current_ifd_index]
                    .get_tags()
                    .len()
            {
                self.current_tag_index += 1;
                return Some(
                    &self.metadata.image_file_directories[self.current_ifd_index].get_tags()
                        [self.current_tag_index - 1],
                );
            } else {
                self.current_tag_index = 0;
                self.current_ifd_index += 1;
            }
        }
        return None;
    }
}
