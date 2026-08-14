use crate::theme::models::{ThemeDoc, TOKENS_OPEN, TOKENS_CLOSE};
use crate::theme::color::{DEFAULT_ACCENT, RAMP_NAMES, derive_ramp, fmt_triplet, num};

impl ThemeDoc {
    fn accent_is_default(&self) -> bool {
        self.accent
            .iter()
            .zip(DEFAULT_ACCENT.iter())
            .all(|(a, b)| (a - b).abs() < 0.001)
    }
}

pub fn emit_css(doc: &ThemeDoc) -> String {
    let mut root: Vec<String> = Vec::new();

    if !doc.accent_is_default() {
        let ramp = derive_ramp(doc.accent);
        for (name, hsl) in RAMP_NAMES.iter().zip(ramp.iter()) {
            root.push(format!("\t--fds-{}: {};", name, fmt_triplet(*hsl)));
        }
        root.push("\t--fds-accent-default: hsl(var(--fds-accent-base));".into());
        root.push("\t--fds-accent-text-primary: hsl(var(--fds-accent-base));".into());
        root.push("\t--fds-accent-text-secondary: hsl(var(--fds-accent-base));".into());
        root.push("\t--fds-accent-text-tertiary: hsl(var(--fds-accent-base));".into());
        root.push("\t--fds-accent-secondary: hsla(var(--fds-accent-base), 90%);".into());
        root.push("\t--fds-accent-tertiary: hsla(var(--fds-accent-base), 80%);".into());
    }
    if let Some(r) = doc.control_corner_radius {
        root.push(format!("\t--fds-control-corner-radius: {}px;", num(r)));
    }
    if let Some(r) = doc.overlay_corner_radius {
        root.push(format!("\t--fds-overlay-corner-radius: {}px;", num(r)));
    }
    for (name, value) in &doc.token_overrides {
        root.push(format!("\t{}: {};", name.trim(), value.trim()));
    }

    let mut css = String::new();

    for url in &doc.imports {
        let url = url.trim();
        if url.is_empty() {
            continue;
        }
        css.push_str(&format!("@import url(\"{url}\");\n"));
    }
    if !doc.imports.is_empty() {
        css.push('\n');
    }

    css.push_str(TOKENS_OPEN);
    css.push_str(
        "\n/* Bu blok görsel kontrollerden üretilir; elle düzenlerseniz\n   \
         kontroller de güncellenir. Blok dışına yazdığınız her şey korunur. */\n",
    );
    if !root.is_empty() {
        css.push_str(":root,\n.fds-theme-light,\n.fds-theme-dark {\n");
        css.push_str(&root.join("\n"));
        css.push_str("\n}\n");
    }
    if !doc.accent_is_default() {
        css.push_str(
            ".list-item.selected, .list-item.selected *, .list-item.selected svg, .list-item.selected path {\n\
            \tcolor: var(--fds-accent-default) !important;\n\
            \tfill: currentColor !important;\n\
            }\n"
        );
    }
    for (selector, body) in &doc.rule_overrides {
        css.push_str(selector.trim());
        css.push_str(" {\n\t");
        css.push_str(body.trim());
        css.push_str("\n}\n");
    }
    css.push_str(TOKENS_CLOSE);
    css.push('\n');

    if !doc.raw_css.trim().is_empty() {
        css.push('\n');
        css.push_str(doc.raw_css.trim());
        css.push('\n');
    }

    css
}
