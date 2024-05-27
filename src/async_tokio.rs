use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt, BufStream};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;

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
    pub async fn connect<Address>(address: Address) -> std::io::Result<Self>
    where
        Address: ToSocketAddrs + Send,
    {
        let stream = TcpStream::connect(address).await?;
        let mut stream = BufStream::new(stream);

        let mut protocol_version = [0; 1];
        stream.read_exact(&mut protocol_version).await?;
        if protocol_version[0] != 1 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Protocol version is not 1",
            ));
        }

        let mut buf = [0; 2];
        stream.read_exact(&mut buf).await?;
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
    pub async fn flush(&self) -> std::io::Result<()> {
        self.stream.lock().await.flush().await
    }

    /// Set one pixel of the matrix to the given color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the data could not be written to the send buffer
    ///
    /// [flush]: Self::flush
    pub async fn pixel(&self, x: u8, y: u8, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
        self.stream
            .lock()
            .await
            .write_all(&[Command::Pixel as u8, x, y, red, green, blue])
            .await
    }

    /// Fill the whole matrix with one color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub async fn fill(&self, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
        self.stream
            .lock()
            .await
            .write_all(&[Command::Fill as u8, red, green, blue])
            .await
    }

    #[allow(clippy::too_many_arguments)]
    /// Fill the given rectangular area with one color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the command could not be sent
    ///
    /// [flush]: Self::flush
    pub async fn rectangle(
        &self,
        x: u8,
        y: u8,
        width: u8,
        height: u8,
        red: u8,
        green: u8,
        blue: u8,
    ) -> std::io::Result<()> {
        self.stream
            .lock()
            .await
            .write_all(&[
                Command::Rectangle as u8,
                x,
                y,
                width,
                height,
                red,
                green,
                blue,
            ])
            .await
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
    pub async fn contiguous(
        &self,
        x: u8,
        y: u8,
        width: u8,
        height: u8,
        colors: &[u8],
    ) -> std::io::Result<()> {
        let too_wide = x
            .checked_add(width)
            .map_or(true, |width| width > self.width);
        let too_high = y
            .checked_add(height)
            .map_or(true, |height| height > self.height);
        if too_wide || too_high {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "area too big for display",
            ));
        }

        let expected_length = (width as usize) * (height as usize) * 3;
        if expected_length != colors.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "colors is wrong length",
            ));
        }

        let mut stream = self.stream.lock().await;

        stream
            .write_all(&[Command::Contiguous as u8, x, y, width, height])
            .await?;
        stream.write_all(colors).await
    }
}
