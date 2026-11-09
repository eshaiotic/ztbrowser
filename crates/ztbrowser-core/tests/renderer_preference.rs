//! Which GTK renderer a start of ZTBrowser uses: the OpenGL renderer unless the person chose another one.

use std::ffi::OsStr;

use ztbrowser_core::rendering::renderer_preference::RendererPreference;

struct StartScenario {
    chosen_renderer: Option<String>,
}

impl StartScenario {
    fn given_environment_without_renderer() -> Self {
        Self { chosen_renderer: None }
    }

    fn given_person_chose(renderer: &str) -> Self {
        Self { chosen_renderer: Some(renderer.to_owned()) }
    }

    fn then_restarts_to_pick_opengl(&self) -> bool {
        RendererPreference::needs_restart(self.chosen_renderer.as_deref().map(OsStr::new))
    }
}

#[test]
fn should_restart_with_opengl_when_no_renderer_was_chosen() {
    assert!(StartScenario::given_environment_without_renderer().then_restarts_to_pick_opengl());
}

#[test]
fn should_restart_when_the_variable_is_set_but_empty() {
    assert!(StartScenario::given_person_chose("").then_restarts_to_pick_opengl());
}

#[test]
fn should_keep_the_renderer_the_person_chose() {
    for renderer in ["vulkan", "cairo", "gl", "ngl"] {
        assert!(!StartScenario::given_person_chose(renderer).then_restarts_to_pick_opengl(), "{renderer}");
    }
}

#[test]
fn should_name_the_variable() {
    assert_eq!(RendererPreference::VARIABLE, "GSK_RENDERER");
}

#[test]
fn should_call_the_opengl_renderer_ngl_before_gtk_4_16() {
    assert_eq!(RendererPreference::preferred_for(14), "ngl");
}

#[test]
fn should_call_the_opengl_renderer_gl_from_gtk_4_16() {
    assert_eq!((RendererPreference::preferred_for(16), RendererPreference::preferred_for(24)), ("gl", "gl"));
}
