use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufStream};

use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;

use crate::Command;

pub struct Client<S: AsyncRead + AsyncWrite> {
    stream: Arc<Mutex<BufStream<S>>>,
    width: u8,
    height: u8,
}

impl<S: AsyncRead + AsyncWrite> Clone for Client<S> {
    fn clone(&self) -> Self {
        Self {
            stream: self.stream.clone(),
            ..*self
        }
    }
}

impl Client<TcpStream> {
    /// Connect to the server via TCP
    ///
    /// # Errors
    /// Errors when the connection could not be established.
    pub async fn connect<A>(addr: A) -> std::io::Result<Self>
    where
        A: ToSocketAddrs + Send,
    {
        let stream = TcpStream::connect(addr).await?;
        Self::new(stream).await
    }
}

impl<S: AsyncRead + AsyncWrite> Client<S> {
    #[must_use]
    pub fn width(&self) -> u8 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u8 {
        self.height
    }

    #[must_use]
    pub fn total_pixels(&self) -> u16 {
        u16::from(self.width) * u16::from(self.height)
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> Client<S> {
    /// Create a client via a connection stream
    ///
    /// ```no_run
    /// # use tokio::net::TcpStream;
    /// # use esp_wlan_led_matrix_client::async_tokio::Client;
    /// # #[tokio::main]
    /// # async fn main() -> std::io::Result<()> {
    /// let stream = TcpStream::connect("espPixelmatrix:1337").await?;
    /// let client = Client::new(stream).await?;
    /// # Ok(())
    /// # }
    /// ```
    /// # Errors
    /// Errors when the response is not correct to the protocol
    pub async fn new(stream: S) -> std::io::Result<Self> {
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

    /// Flushes the internal buffer and sends everything to the server
    ///
    /// # Errors
    /// Errors when the command could not be sent
    pub async fn flush(&mut self) -> std::io::Result<()> {
        self.stream.lock().await.flush().await
    }

    /// Set one pixel of the matrix to the given color.
    /// Do not forget to also run [flush] afterwards.
    ///
    /// # Errors
    /// Errors when the data could not be written to the send buffer
    ///
    /// [flush]: Self::flush
    pub async fn pixel(
        &mut self,
        x: u8,
        y: u8,
        red: u8,
        green: u8,
        blue: u8,
    ) -> std::io::Result<()> {
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
    pub async fn fill(&mut self, red: u8, green: u8, blue: u8) -> std::io::Result<()> {
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
        &mut self,
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
