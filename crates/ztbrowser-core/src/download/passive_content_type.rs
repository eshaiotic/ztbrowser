//! Which downloaded files may be opened straight from the downloads list.
//!
//! A file from a website can be a program, a script or a launcher dressed as a document. Only types that
//! merely display something (an exact list of types) are opened with one click; everything else is left to the file manager, which
//! asks the person what to do. The decision fails closed: an unknown or missing type is not passive.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassiveContentType(String);

impl PassiveContentType {
    #[must_use]
    pub fn new(content_type: &str) -> Self {
        Self(content_type.trim().to_ascii_lowercase())
    }

    /// Exactly these types are passive: they only display something. Prefix matching would let in SVG, EPS and
    /// playlist types (which can carry scripts or point at other files), so every type is named.
    const PASSIVE: [&'static str; 13] = [
        "application/pdf",
        "text/plain",
        "image/png",
        "image/jpeg",
        "image/gif",
        "image/webp",
        "image/avif",
        "audio/mpeg",
        "audio/ogg",
        "audio/flac",
        "video/mp4",
        "video/webm",
        "video/ogg",
    ];

    #[must_use]
    pub fn is_passive(&self) -> bool {
        Self::PASSIVE.contains(&self.0.as_str())
    }
}
