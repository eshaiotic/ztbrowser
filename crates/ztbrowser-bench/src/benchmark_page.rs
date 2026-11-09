//! A real page with the scroll script injected.

use crate::scroll_scenario::ScrollScenario;

#[derive(Debug, Clone)]
pub struct BenchmarkPage {
    html: String,
}

impl BenchmarkPage {
    /// `original_html` is fetched from `base_url`; a `<base>` element keeps its relative links working while
    /// the page itself is served from the benchmark server.
    #[must_use]
    pub fn injected_into(original_html: &str, base_url: &str, scenario: &ScrollScenario, server_origin: &str) -> Self {
        let script = scenario.script(&format!("{server_origin}/loaded"), &format!("{server_origin}/result"));
        let injection = format!("<base href=\"{base_url}\"><script>{script}</script>");
        let html = match original_html.find("<head>") {
            Some(position) => {
                let (before, after) = original_html.split_at(position + "<head>".len());
                format!("{before}{injection}{after}")
            }
            None => format!("<head>{injection}</head>{original_html}"),
        };
        Self { html }
    }

    #[must_use]
    pub fn html(&self) -> &str {
        &self.html
    }
}
