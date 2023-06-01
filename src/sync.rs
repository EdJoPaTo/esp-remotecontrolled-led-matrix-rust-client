use core::time::Duration;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::{Arc, Mutex};

use bufstream::BufStream;

use crate::Command;

#[derive(Clone)]
pub struct Client {
    stream: Arc<Mutex<BufStream<TcpStream>>>,
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
        Self::connect_tcp_stream(stream)
    }

    /// Connect to the server
    ///
    /// For each `SocketAddr` possible from the given `addr` the timeout is used.
    /// When an `addr` resolved to IPv4 and IPv6 the timeout could be used up twice.
    ///
    /// # Errors
    /// Errors when the connection could not be established.
    pub fn connect_timeout(addr: impl ToSocketAddrs, timeout: Duration) -> std::io::Result<Self> {
        let mut last_err = None;
        for addr in addr.to_socket_addrs()? {
            match TcpStream::connect_timeout(&addr, timeout).and_then(Self::connect_tcp_stream) {
                Ok(s) => return Ok(s),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap_or_else(|| {
            std::io::Error::new(ErrorKind::InvalidInput, "could not resolve to any address")
        }))
    }

    fn connect_tcp_stream(stream: TcpStream) -> std::io::Result<Self> {
        let mut stream = BufStream::new(stream);

        let mut protocol_version = [0; 1];
        stream.read_exact(&mut protocol_version)?;
        if protocol_version[0] != 1 {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "Protocol version is not 1",
            ));
        }

        let mut buf = [0; 2];
        stream.read_exact(&mut buf)?;
        let [width, height] = buf;

        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
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
        self.stream.lock().map_err(poison_err)?.flush()
    }

    /// Set one pixel of the matrix to the given color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the data could not be written to the send buffer
    ///
    /// [flush]: Self::flush
    pub fn pixel(&mut self, x: u8, y: u8, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
        let mut stream = self.stream.lock().map_err(poison_err)?;
        stream.write_all(&[Command::Pixel as u8, x, y, red, green, blue])
    }

    /// Fill the whole matrix with one color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub fn fill(&mut self, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
        let mut stream = self.stream.lock().map_err(poison_err)?;
        stream.write_all(&[Command::Fill as u8, red, green, blue])
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
        let mut stream = self.stream.lock().map_err(poison_err)?;
        stream.write_all(&[
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

    /// Send an area full of colors.
    ///
    /// The area begins in the top left at x/y and moves first on the x axis, then on the y axis.
    /// The colors are given in R G B order.
    ///
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub fn contiguous(
        &mut self,
        x: u8,
        y: u8,
        width: u8,
        height: u8,
        colors: &[u8],
    ) -> std::io::Result<()> {
        let too_wide = x.checked_add(width).map_or(true, |w| w > self.width);
        let too_high = y.checked_add(height).map_or(true, |h| h > self.height);
        if too_wide || too_high {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "area too big for display",
            ));
        }

        let expected_length = (width as usize) * (height as usize) * 3;
        if expected_length != colors.len() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "colors is wrong length",
            ));
        }

        let mut stream = self.stream.lock().map_err(poison_err)?;
        stream.write_all(&[Command::Contiguous as u8, x, y, width, height])?;
        stream.write_all(colors)
    }
}

#[cfg(feature = "embedded-graphics")]
mod embedded_graphics {
    use crate::sync::Client;
    use embedded_graphics::prelude::{Dimensions, PointsIter, RgbColor, Size};
    use embedded_graphics::primitives::Rectangle;

    impl embedded_graphics::geometry::OriginDimensions for Client {
        fn size(&self) -> Size {
            Size::new(u32::from(self.width), u32::from(self.height))
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    impl embedded_graphics::prelude::DrawTarget for Client {
        type Color = embedded_graphics::pixelcolor::Rgb888;
        type Error = std::io::Error;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
        {
            let bounding_box = self.bounding_box();
            for p in pixels {
                let point = p.0;
                let color = p.1;
                if bounding_box.contains(point) {
                    self.pixel(
                        point.x as u8,
                        point.y as u8,
                        color.r(),
                        color.g(),
                        color.b(),
                    )?;
                }
            }
            Ok(())
        }

        fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Self::Color>,
        {
            let drawable_area = area.intersection(&self.bounding_box());
            if drawable_area.is_zero_sized() {
                return Ok(());
            }
            let colors = area
                .points()
                .zip(colors)
                .filter(|(pos, _color)| drawable_area.contains(*pos))
                .flat_map(|(_pos, c)| [c.r(), c.g(), c.b()])
                .collect::<Vec<_>>();
            self.contiguous(
                drawable_area.top_left.x as u8,
                drawable_area.top_left.y as u8,
                drawable_area.size.width as u8,
                drawable_area.size.height as u8,
                &colors,
            )
        }

        fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
            let drawable_area = area.intersection(&self.bounding_box());
            if drawable_area.is_zero_sized() {
                return Ok(());
            }
            self.rectangle(
                drawable_area.top_left.x as u8,
                drawable_area.top_left.y as u8,
                drawable_area.size.width as u8,
                drawable_area.size.height as u8,
                color.r(),
                color.g(),
                color.b(),
            )
        }

        fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
            self.fill(color.r(), color.g(), color.b())
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn poison_err<S>(_err: S) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, "Mutex poisoned")
}
