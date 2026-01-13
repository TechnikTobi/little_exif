// Copyright © 2024-2026 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;

use super::ImageFileDirectory;

impl
ImageFileDirectory
{
    /// Sets the value of an image file directory. Checks if the group of the
    /// IFD and the default group of the tag match and prints a warning
    /// otherwise. 
    /// If the tag already exists in the IFD, it is replaced by the given tag.
    /// All tags in the IFD are sorted after the insert. 
    pub fn
    set_tag
    (
        &mut self,
        input_tag: ExifTag,	
    )
    {
        if input_tag.get_group() != self.ifd_type
        {
            log::warn!("The tag {input_tag:?} is set in an IFD that has not a matching group.");
        }
        self.tags.retain(|tag| tag.as_u16() != input_tag.as_u16());
        self.tags.push(input_tag);
        self.sort_tags();
    }

    /// Removes a tag with a given hex value from the image file directory.
    /// If the tag is removed successfully, nothing happens.
    /// If no such tag exists, nothing happens.
    pub fn
    remove_tag
    (
        &mut self,
        tag_hex: u16
    )
    {
        self.tags.retain(|tag| tag.as_u16() != tag_hex);
        self.sort_tags();
    }
}
