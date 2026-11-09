//! Makes GTK draw with the renderer that suits WebKit's frames (see [`RendererPreference`]).
//!
//! GTK reads `GSK_RENDERER` when the first window is realised, and this program may not change its own
//! environment (`unsafe` is forbidden), so when the variable is not set the process starts itself again with
//! GTK's OpenGL renderer, once, before it has created anything. The copy finds the variable set and carries on.

use std::os::unix::process::CommandExt;
use std::process::Command;

use ztbrowser_core::rendering::renderer_preference::RendererPreference;

pub struct GraphicsRenderer;

impl GraphicsRenderer {
    const LOG_DOMAIN: &'static str = "ztbrowser";

    /// Replaces this process by a copy of itself that uses the OpenGL renderer; returns when the person chose a
    /// renderer or when the copy cannot start (then GTK keeps its default, which is logged).
    pub fn prefer_opengl() {
        if !RendererPreference::needs_restart(std::env::var_os(RendererPreference::VARIABLE).as_deref()) {
            return;
        }
        let renderer = RendererPreference::preferred_for(gtk::minor_version());
        let error = std::env::current_exe().map(|program| {
            Command::new(program).args(std::env::args_os().skip(1)).env(RendererPreference::VARIABLE, renderer).exec()
        });
        glib::g_warning!(Self::LOG_DOMAIN, "cannot start with the {renderer} renderer: {error:?}");
    }
}
