// Copyright © 2024 Tobias J. Prisching <tobias.prisching@icloud.com> and CONTRIBUTORS
// See https://github.com/TechnikTobi/little_exif#license for licensing details

use crate::exif_tag::ExifTag;

use super::Metadata;

impl<'a> IntoIterator 
for &'a Metadata
{
	type Item = ExifTag;
	type IntoIter = MetadataIterator<'a>;

	fn
	into_iter
	(
		self
	)
	-> Self::IntoIter
	{
		MetadataIterator 
		{
			metadata: self
		}
	}
}

pub struct
MetadataIterator<'a>
{
	metadata:    &'a Metadata,
	// current_ifd: 
}

impl<'a> Iterator
for MetadataIterator<'a>
{	
	type Item = ExifTag;
	
	fn 
	next
	(
		&mut self
	) 
	-> Option<Self::Item> 
	{
		todo!()
	}
}

/*
impl<'a> IntoIterator for &'a Pixel {
    type Item = i8;
    type IntoIter = PixelIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PixelIterator {
            pixel: self,
            index: 0,
        }
    }
}

pub struct PixelIterator<'a> {
    pixel: &'a Pixel,
    index: usize,
}

impl<'a> Iterator for PixelIterator<'a> {
    type Item = i8;
    fn next(&mut self) -> Option<i8> {
        let result = match self.index {
            0 => self.pixel.r,
            1 => self.pixel.g,
            2 => self.pixel.b,
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}
*/