//! The fixed header of a PNG file, read without decoding the image.
//!
//! Web app icons are downloaded from the site and decoded in the browser process, outside WebKit's sandbox.
//! ZTBrowser therefore accepts only PNG (one widely audited decoder) and checks the declared dimensions first:
//! a 30 000 × 30 000 "icon" is a few kilobytes on the wire but gigabytes once decoded.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PngHeader {
    pub width: u32,
    pub height: u32,
}

impl PngHeader {
    const SIGNATURE: &'static [u8] = b"\x89PNG\r\n\x1a\n";
    /// The first chunk must be `IHDR`; its type sits after the 4-byte chunk length.
    const HEADER_CHUNK: &'static [u8] = b"IHDR";
    const LARGEST_ICON_SIDE: u32 = 2048;

    /// `None` unless `bytes` start like a PNG file.
    #[must_use]
    pub fn read(bytes: &[u8]) -> Option<Self> {
        let rest = bytes.strip_prefix(Self::SIGNATURE)?;
        let chunk = rest.get(4..)?.strip_prefix(Self::HEADER_CHUNK)?;
        let width = u32::from_be_bytes(chunk.get(0..4)?.try_into().ok()?);
        let height = u32::from_be_bytes(chunk.get(4..8)?.try_into().ok()?);
        Some(Self { width, height })
    }

    #[must_use]
    pub fn is_reasonable_icon(&self) -> bool {
        (1..=Self::LARGEST_ICON_SIDE).contains(&self.width) && (1..=Self::LARGEST_ICON_SIDE).contains(&self.height)
    }
}
