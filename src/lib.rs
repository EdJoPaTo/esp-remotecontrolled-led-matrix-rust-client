#[cfg(feature = "async-tokio")]
pub mod async_tokio;

#[cfg(feature = "sync")]
pub mod sync;

#[derive(PartialEq)]
#[repr(u8)]
pub enum Command {
    Fill = 1,
    Pixel = 2,
    Rectangle = 3,
}
