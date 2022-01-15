use image::imageops::FilterType;
use image::{EncodableLayout, ImageBuffer, ImageError, Primitive, Rgb};
use std::ops::Index;
use std::path::Path;

pub struct Image<T>
where
    T: 'static + Primitive,
    [T]: EncodableLayout,
{
    width: u32,
    height: u32,
    data: ImageBuffer<Rgb<T>, Vec<T>>,
}

impl<T> Image<T>
where
    T: 'static + Primitive,
    [T]: EncodableLayout,
{
    pub fn new<U, V>(width: usize, height: usize, pixels: U) -> Self
    where
        U: Index<usize, Output = V>,
        V: Into<[T; 3]> + Clone,
    {
        // TODO Move matrix_index_to_vec to another file
        let index_converter = crate::matrix_index_to_vec(width);

        let width = width as u32;
        let height = height as u32;

        let data = ImageBuffer::from_fn(width, height, |column, line| {
            image::Rgb(
                pixels[index_converter(line as usize, column as usize)]
                    .clone()
                    .into(),
            )
        });

        Self {
            width,
            height,
            data,
        }
    }

    pub fn resize(&self, scale: u32) -> Self {
        let data = image::imageops::resize(
            &self.data,
            self.width * scale,
            self.height * scale,
            FilterType::Nearest,
        );

        Self {
            width: self.width,
            height: self.height,
            data,
        }
    }

    pub fn save<U: AsRef<Path>>(&self, path: U) -> Result<(), ImageError> {
        self.data.save(path)
    }
}
