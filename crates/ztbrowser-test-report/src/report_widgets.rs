//! The small pieces the report page is built from: badges, sparklines, run strips, result bars and donuts.
//! Each turns values into an HTML fragment and nothing else, so they are plain functions.

use std::fmt::Write;

use crate::percent::Percent;

const SPARK_WIDTH: f64 = 148.0;
const SPARK_HEIGHT: f64 = 26.0;

/// One slice of a donut chart or of a result bar.
pub struct ChartSlice<'label> {
    pub label: &'label str,
    pub colour: &'label str,
    pub value: u32,
}

/// One bar of a card's strip: its colour and the tooltip describing the run.
pub struct StripBar {
    pub colour: &'static str,
    pub title: String,
}

/// Text for HTML element content and attribute values.
pub fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

pub fn badge(label: &str, colour: &str) -> String {
    format!(
        r#"<span class="badge" style="color:{colour};border-color:{colour}55;background:{colour}14"><i style="background:{colour}"></i>{label}</span>"#
    )
}

/// `x,y` points of a line through `values`, scaled to the sparkline's box; one value draws a flat line.
pub fn sparkline_points(values: &[f64]) -> String {
    let series: Vec<f64> = if values.len() == 1 { vec![values[0], values[0]] } else { values.to_vec() };
    let highest = series.iter().copied().fold(f64::MIN, f64::max);
    let lowest = series.iter().copied().fold(f64::MAX, f64::min);
    let span = highest - lowest;
    let step = SPARK_WIDTH / f64::from(u32::try_from(series.len() - 1).unwrap_or(u32::MAX));
    let mut points = String::new();
    let mut x = 0.0;
    for value in series {
        let y = if span > 0.0 { (highest - value) / span * SPARK_HEIGHT } else { SPARK_HEIGHT / 2.0 };
        if !points.is_empty() {
            points.push(' ');
        }
        let _ = write!(points, "{},{}", (x * 100.0_f64).round() / 100.0, (y * 100.0_f64).round() / 100.0 + 5.0);
        x += step;
    }
    points
}

pub fn sparkline(values: &[f64]) -> String {
    if values.is_empty() {
        return r#"<svg class="spark" viewBox="0 0 148 36"></svg>"#.to_owned();
    }
    let points = sparkline_points(values);
    format!(
        r##"<svg class="spark" viewBox="0 0 148 36" preserveAspectRatio="none"><polygon points="0,36 {points} 148,36" fill="url(#spark-fill)"></polygon><polyline points="{points}" fill="none" stroke="#c9d1d9" stroke-width="1.2"></polyline></svg>"##
    )
}

/// Room for `window` runs; runs that do not exist yet show as empty slots on the left.
pub fn run_strip(bars: &[StripBar], window: usize) -> String {
    let mut strip = String::from(r#"<div class="strip">"#);
    for _ in bars.len()..window {
        strip.push_str(r#"<span class="slot"></span>"#);
    }
    for bar in bars {
        let _ = write!(strip, r#"<span style="background:{}" title="{}"></span>"#, bar.colour, escape_html(&bar.title));
    }
    strip.push_str("</div>");
    strip
}

pub fn result_bar(slices: &[ChartSlice<'_>]) -> String {
    let mut bar = String::from(r#"<div class="result-bar">"#);
    for slice in slices.iter().filter(|slice| slice.value > 0) {
        let _ = write!(
            bar,
            r#"<span style="flex:{value};background:{colour}" title="{label}: {value}"></span>"#,
            value = slice.value,
            colour = slice.colour,
            label = slice.label.to_lowercase(),
        );
    }
    bar.push_str("</div>");
    bar
}

pub fn type_tag(kind: &str) -> String {
    format!(r#"<span class="tag {}">{kind}</span>"#, kind.to_lowercase())
}

/// A ring chart: one arc per non-empty slice, the share of `total` in its legend, a value in the middle.
pub fn donut(title: &str, slices: &[ChartSlice<'_>], total: u32, center_value: &str, center_caption: &str) -> String {
    let mut arcs = String::new();
    let mut consumed = 0.0;
    for slice in slices.iter().filter(|slice| slice.value > 0 && total > 0) {
        let percent = f64::from(slice.value) / f64::from(total) * 100.0;
        let _ = write!(
            arcs,
            r#"<circle r="15.9155" cx="21" cy="21" fill="none" stroke="{colour}" stroke-width="5" stroke-dasharray="{percent} {rest}" stroke-dashoffset="{offset}"><title>{label}: {value}</title></circle>"#,
            colour = slice.colour,
            rest = 100.0 - percent,
            offset = 25.0 - consumed,
            label = slice.label,
            value = slice.value,
        );
        consumed += percent;
    }
    let mut legend = String::new();
    for slice in slices {
        let _ = write!(
            legend,
            r#"<li><i style="background:{}"></i><b>{}</b> {}<small>{}</small></li>"#,
            slice.colour,
            Percent::of(slice.value, total),
            slice.label,
            slice.value,
        );
    }
    format!(
        r##"<figure class="donut"><figcaption class="kpi-label">{title}</figcaption><svg viewBox="0 0 42 42" role="img" aria-label="{title}"><circle r="15.9155" cx="21" cy="21" fill="none" stroke="#ffffff10" stroke-width="5"></circle>{arcs}<text x="21" y="21.5" class="donut-value">{center_value}</text><text x="21" y="26.5" class="donut-caption">{center_caption}</text></svg><ul>{legend}</ul></figure>"##
    )
}
