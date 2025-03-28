//! Image loading.

/// Raw image data.
///
/// See [Sprite][crate::graphics::Sprite] for drawing the image.
pub struct Image {
    pub(crate) data: Box<[u8]>,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl Image {
    /// Create a new `Image` from png data.
    ///
    /// # Panics
    ///
    /// Panics if the `Image` could not be created for any reason.
    pub fn from_png<R>(source: R) -> Self
    where
        R: std::io::Read,
    {
        let decoder = png::Decoder::new(source);
        let mut reader = decoder.read_info().unwrap();
        let mut data = vec![0; reader.output_buffer_size()].into_boxed_slice();
        let info = reader.next_frame(&mut data).unwrap();
        Self {
            data,
            width: info.width,
            height: info.height,
        }
    }

    /// Creates a new `Image` from raw data.
    ///
    /// # Panics
    ///
    /// Panics if `data.len() != width * height * 4`.
    pub fn from_data<T>(data: T, width: u32, height: u32) -> Self
    where
        T: Into<Box<[u8]>>,
    {
        let data = data.into();
        if data.len() != width as usize * height as usize * 4 {
            panic!("heart::image: data size does not match width and height");
        }
        Self {
            data,
            width,
            height,
        }
    }

    /// A slice to the underlying data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Width of the image in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the image in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Include a png file and turn it into an [Image].
#[macro_export]
macro_rules! include_png {
    ($($token:tt)*) => {
        heart::image::Image::from_png(include_bytes!($($token)*).as_slice())
    };
}
