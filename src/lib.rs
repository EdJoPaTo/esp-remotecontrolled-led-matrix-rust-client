#[cfg(feature = "tokio")]
pub mod async_tokio;
#[cfg(feature = "sync")]
pub mod sync;

#[cfg(any(feature = "sync", feature = "tokio"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[repr(u8)]
pub(crate) enum Command {
    Fill = 1,
    Pixel = 2,
    Rectangle = 3,
    Contiguous = 4,
}
