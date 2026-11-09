//! GTK's renderer is chosen once, when the first window is realised, from the `GSK_RENDERER` variable.
//!
//! GTK 4.24 picks its Vulkan renderer by default. With NVIDIA's driver that renderer cannot import the tiled
//! buffers (dmabuf with modifier `0x0300000000e08014`) WebKit hands over for every page frame, so GTK downloads
//! each frame into main memory and uploads it again: measured on a scrolling page, 180–210 % of a processor
//! core against 62–64 % with the OpenGL renderer, which imports the buffer without a copy. OpenGL is therefore
//! the default here; a person who set `GSK_RENDERER` themselves keeps their choice.

use std::ffi::OsStr;

pub struct RendererPreference;

impl RendererPreference {
    pub const VARIABLE: &'static str = "GSK_RENDERER";
    /// GTK 4.14 calls the new OpenGL renderer `ngl`; from GTK 4.16 it is `gl` (`ngl` still works there, with a
    /// warning on every start, and `gl` on 4.14 would select the old renderer).
    const FIRST_MINOR_CALLING_IT_GL: u32 = 16;

    /// The name of the OpenGL renderer in the GTK 4 release whose minor version is `gtk_minor`.
    #[must_use]
    pub fn preferred_for(gtk_minor: u32) -> &'static str {
        if gtk_minor >= Self::FIRST_MINOR_CALLING_IT_GL { "gl" } else { "ngl" }
    }

    /// `current` is the value of [`Self::VARIABLE`] in the environment, if it is set at all. An empty value
    /// counts as not chosen.
    #[must_use]
    pub fn needs_restart(current: Option<&OsStr>) -> bool {
        current.is_none_or(OsStr::is_empty)
    }
}
