use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

use bufstream::BufStream;

use crate::Command;

pub struct Client {
    stream: BufStream<TcpStream>,
    width: u8,
    height: u8,
}

impl Client {
    /// Connect to the server
    ///
    /// # Errors
    /// Errors when the connection could not be established.
    pub fn connect(addr: impl ToSocketAddrs) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        let mut stream = BufStream::new(stream);

        let mut buf = [0; 2];
        stream.read_exact(&mut buf)?;

        let width = buf[0];
        let height = buf[1];

        Ok(Self {
            stream,
            width,
            height,
        })
    }

    #[must_use]
    pub const fn width(&self) -> u8 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u8 {
        self.height
    }

    #[must_use]
    pub const fn total_pixels(&self) -> u16 {
        (self.width as u16) * (self.height as u16)
    }

    /// Flushes the internal buffer and sends everything to the server
    ///
    /// # Errors
    /// Errors when the command could not be sent
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }

    /// Set one pixel of the matrix to the given color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the data could not be written to the send buffer
    ///
    /// [flush]: Self::flush
    pub fn pixel(&mut self, x: u8, y: u8, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
        self.stream
            .write_all(&[Command::Pixel as u8, x, y, red, green, blue])
    }

    /// Fill the whole matrix with one color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub fn fill(&mut self, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
        self.stream
            .write_all(&[Command::Fill as u8, red, green, blue])
    }

    #[allow(clippy::too_many_arguments)]
    /// Fill the given rectangular area with one color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub fn rectangle(
        &mut self,
        x: u8,
        y: u8,
        width: u8,
        height: u8,
        red: u8,
        green: u8,
        blue: u8,
    ) -> std::io::Result<()> {
        self.stream.write_all(&[
            Command::Rectangle as u8,
            x,
            y,
            width,
            height,
            red,
            green,
            blue,
        ])
    }
}

#[cfg(feature = "embedded-graphics")]
impl embedded_graphics::geometry::OriginDimensions for Client {
    fn size(&self) -> embedded_graphics::prelude::Size {
        embedded_graphics::prelude::Size::new(u32::from(self.width), u32::from(self.height))
    }
}

#[cfg(feature = "embedded-graphics")]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
impl embedded_graphics::prelude::DrawTarget for Client {
    type Color = embedded_graphics::pixelcolor::Rgb888;
    type Error = std::io::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        use embedded_graphics::prelude::RgbColor;
        for p in pixels {
            let point = p.0;
            let color = p.1;

            self.pixel(
                point.x as u8,
                point.y as u8,
                color.r(),
                color.g(),
                color.b(),
            )?;
        }
        self.flush()
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        use embedded_graphics::prelude::RgbColor;
        self.fill(color.r(), color.g(), color.b())?;
        self.flush()
    }
}
