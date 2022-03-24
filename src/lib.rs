#[cfg(feature = "async-tokio")]
pub mod async_tokio;
#[cfg(feature = "sync")]
pub mod sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
pub(crate) enum Command {
    Fill = 1,
    Pixel = 2,
    Rectangle = 3,
    Contiguous = 4,
}
