use std::collections::BTreeMap;
use crate::theme::models::{ThemeDoc, TOKENS_OPEN, TOKENS_CLOSE};
use crate::theme::color::{DEFAULT_ACCENT, Hsl, parse_triplet, parse_color_to_hsl};

pub const DERIVED_TOKEN_NAMES: [&str; 13] = [
    "accent-light-3",
    "accent-light-2",
    "accent-light-1",
    "accent-base",
    "accent-dark-1",
    "accent-dark-2",
    "accent-dark-3",
    "accent-default",
    "accent-text-primary",
    "accent-text-secondary",
    "accent-text-tertiary",
    "accent-secondary",
    "accent-tertiary",
];

fn find_decl<'a>(block: &'a str, name: &str) -> Option<&'a str> {
    let bytes = block.as_bytes();
    let mut from = 0usize;

    while let Some(rel) = block[from..].find(name) {
        let at = from + rel;
        let ident_before = at > 0
            && matches!(bytes[at - 1], b'-' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9');
        let after = &block[at + name.len()..];
        let trimmed = after.trim_start();

        if !ident_before && trimmed.starts_with(':') {
            let val_start = at + name.len() + (after.len() - trimmed.len()) + 1;
            let end = block[val_start..]
                .find(';')
                .map(|e| val_start + e)
                .unwrap_or(block.len());
            return Some(block[val_start..end].trim());
        }
        from = at + name.len();
    }
    None
}

pub fn parse_px(value: &str) -> Option<f64> {
    let v = value.trim();
    if v.ends_with("rem") {
        let num: f64 = v.trim_end_matches("rem").trim().parse().ok()?;
        Some((num * 16.0 * 10.0).round() / 10.0)
    } else if v.ends_with("em") {
        let num: f64 = v.trim_end_matches("em").trim().parse().ok()?;
        Some((num * 16.0 * 10.0).round() / 10.0)
    } else {
        v.trim_end_matches("px").trim().parse().ok()
    }
}

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn split_rules(block: &str) -> Vec<(String, String)> {
    let bytes = block.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut selector_start = 0usize;

    while i < block.len() {
        if bytes[i] == b'{' {
            let selector = block[selector_start..i].trim().to_string();
            let mut depth = 1usize;
            let mut j = i + 1;
            while j < block.len() && depth > 0 {
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            let body_end = if depth == 0 { j - 1 } else { block.len() };
            if !selector.is_empty() {
                out.push((selector, block[i + 1..body_end].trim().to_string()));
            }
            i = j;
            selector_start = j;
        } else {
            i += 1;
        }
    }
    out
}

fn parse_decls(body: &str) -> Vec<(String, String)> {
    body.split(';')
        .filter_map(|decl| {
            let (key, value) = decl.split_once(':')?;
            let (key, value) = (key.trim(), value.trim());
            if key.is_empty() || value.is_empty() {
                None
            } else {
                Some((key.to_string(), value.to_string()))
            }
        })
        .collect()
}

fn split_imports(text: &str) -> (Vec<String>, String) {
    let mut imports = Vec::new();
    let mut rest = String::with_capacity(text.len());

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("@import") {
            let inner = trimmed
                .trim_start_matches("@import")
                .trim()
                .trim_end_matches(';')
                .trim();
            let inner = inner
                .strip_prefix("url(")
                .map(|s| s.trim_end_matches(')'))
                .unwrap_or(inner);
            let url = inner.trim().trim_matches(['"', '\''].as_ref()).trim();
            if !url.is_empty() {
                imports.push(url.to_string());
            }
        } else {
            rest.push_str(line);
            rest.push('\n');
        }
    }

    (imports, rest)
}

pub fn parse_css(text: &str, known_selectors: &[String], current: &ThemeDoc) -> ThemeDoc {
    let (imports, without_imports) = split_imports(text);
    let text = without_imports.as_str();

    let open = text.find(TOKENS_OPEN);
    let close = text.find(TOKENS_CLOSE);

    let (managed, raw) = match (open, close) {
        (Some(o), Some(c)) if c > o => {
            let managed = &text[o + TOKENS_OPEN.len()..c];
            let before = text[..o].trim();
            let after = text[c + TOKENS_CLOSE.len()..].trim();
            let raw = match (before.is_empty(), after.is_empty()) {
                (true, true) => String::new(),
                (true, false) => after.to_string(),
                (false, true) => before.to_string(),
                (false, false) => format!("{before}\n\n{after}"),
            };
            (Some(managed), raw)
        }
        _ => (None, text.trim().to_string()),
    };

    let Some(managed) = managed else {
        return parse_foreign_css(text, known_selectors, current);
    };

    let cleaned = strip_comments(managed);
    let mut rule_overrides = BTreeMap::new();
    let mut root_body = String::new();

    for (selector, body) in split_rules(&cleaned) {
        let normalized = selector.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.starts_with(":root") {
            root_body.push_str(&body);
            root_body.push(';');
        } else if normalized.starts_with(".list-item.selected") {
            
        } else {
            rule_overrides.insert(normalized, body);
        }
    }

    let derived: Vec<String> = DERIVED_TOKEN_NAMES.iter().map(|n| format!("--fds-{n}")).collect();
    let mut token_overrides = BTreeMap::new();
    for (prop, value) in parse_decls(&root_body) {
        let structural = prop == "--fds-control-corner-radius"
            || prop == "--fds-overlay-corner-radius"
            || derived.iter().any(|d| d == &prop);
        if !structural {
            token_overrides.insert(prop, value);
        }
    }

    ThemeDoc {
        accent: find_decl(&root_body, "--fds-accent-base")
            .and_then(parse_triplet)
            .unwrap_or(DEFAULT_ACCENT),
        mode: current.mode,
        control_corner_radius: find_decl(&root_body, "--fds-control-corner-radius")
            .and_then(parse_px),
        overlay_corner_radius: find_decl(&root_body, "--fds-overlay-corner-radius")
            .and_then(parse_px),
        imports,
        token_overrides,
        rule_overrides,
        raw_css: raw,
    }
}

struct SpannedRule {
    selector: String,
    body: String,
    start: usize,
    end: usize,
}

fn split_rules_spanned(text: &str) -> Vec<SpannedRule> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut selector_start = 0usize;

    fn skip_noise(bytes: &[u8], len: usize, i: usize) -> Option<usize> {
        if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let mut j = i + 2;
            while j + 1 < len && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
                j += 1;
            }
            return Some((j + 2).min(len));
        }
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            let mut j = i + 1;
            while j < len && bytes[j] != quote {
                if bytes[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            return Some((j + 1).min(len));
        }
        None
    }

    while i < len {
        if let Some(next) = skip_noise(bytes, len, i) {
            i = next;
            continue;
        }

        if bytes[i] == b'{' {
            let selector_raw = &text[selector_start..i];
            let lead = selector_raw.len() - selector_raw.trim_start().len();

            let mut depth = 1usize;
            let mut j = i + 1;
            while j < len && depth > 0 {
                if let Some(next) = skip_noise(bytes, len, j) {
                    j = next;
                    continue;
                }
                match bytes[j] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                j += 1;
            }
            let body_end = if depth == 0 { j - 1 } else { len };

            let selector = selector_raw.trim().to_string();
            if !selector.is_empty() {
                out.push(SpannedRule {
                    selector,
                    body: text[i + 1..body_end].trim().to_string(),
                    start: selector_start + lead,
                    end: j,
                });
            }
            i = j;
            selector_start = j;
        } else {
            i += 1;
        }
    }

    out
}

fn resolve_var(val: &str, all_vars: &BTreeMap<String, String>) -> String {
    let mut current = val.trim().to_string();
    for _ in 0..5 {
        if let Some(start) = current.find("var(") {
            if let Some(end) = current[start..].find(')') {
                let inner = current[start + 4..start + end].trim();
                let (var_name, fallback) = if let Some((n, f)) = inner.split_once(',') {
                    (n.trim(), Some(f.trim()))
                } else {
                    (inner, None)
                };

                let replacement = if let Some(v) = all_vars.get(var_name) {
                    v.as_str()
                } else if let Some(fb) = fallback {
                    fb
                } else {
                    ""
                };

                current = format!("{}{}{}", &current[..start], replacement, &current[start + end + 1..]);
                continue;
            }
        }
        break;
    }
    current.trim().to_string()
}

fn extract_accent_from_vars(all_vars: &BTreeMap<String, String>) -> Option<Hsl> {
    let priority_names = [
        "--fds-accent-base",
        "--accent",
        "--accent-color",
        "--accent-primary",
        "--accent-base",
        "--accent-1",
        "--accent-main",
        "--accent-h",
        "--accent-hsl",
        "--primary",
        "--primary-color",
        "--primary-accent",
        "--primary-base",
        "--brand",
        "--brand-color",
        "--theme-color",
        "--main-color",
        "--highlight",
        "--highlight-color",
        "--active-color",
        "--link-color",
        "--color-primary",
        "--color-accent",
        "--btn-bg",
        "--button-bg",
    ];

    for name in priority_names {
        if let Some(val) = all_vars.get(name) {
            let resolved = resolve_var(val, all_vars);
            if let Some(hsl) = parse_color_to_hsl(&resolved) {
                return Some(hsl);
            }
        }
    }

    for (name, val) in all_vars {
        let lower = name.to_lowercase();
        if lower.contains("accent") || lower.contains("primary") || lower.contains("brand") {
            if name.starts_with("--fds-accent-") && name != "--fds-accent-base" {
                continue;
            }
            let resolved = resolve_var(val, all_vars);
            if let Some(hsl) = parse_color_to_hsl(&resolved) {
                return Some(hsl);
            }
        }
    }

    None
}

fn extract_accent_from_rules(rules: &[SpannedRule], all_vars: &BTreeMap<String, String>) -> Option<Hsl> {
    let target_selectors = [
        ".button", ".btn", "a", ".accent", ".primary", ".active",
        ".topbar", ".navbar", "[class*=\"accent\"]", "[class*=\"primary\"]"
    ];

    for rule in rules {
        let norm = normalize_selector(&rule.selector);
        if target_selectors.iter().any(|s| norm.contains(s)) {
            let decls = parse_decls(&rule.body);
            for (prop, val) in decls {
                if prop == "background-color" || prop == "background" || prop == "color" {
                    let resolved = resolve_var(&val, all_vars);
                    if let Some(hsl) = parse_color_to_hsl(&resolved) {
                        if hsl[1] > 10.0 && hsl[2] > 10.0 && hsl[2] < 95.0 {
                            return Some(hsl);
                        }
                    }
                }
            }
        }
    }

    None
}

fn extract_radius_from_vars(all_vars: &BTreeMap<String, String>) -> Option<f64> {
    let priority_names = [
        "--fds-control-corner-radius",
        "--border-radius",
        "--radius",
        "--corner-radius",
        "--control-radius",
        "--card-radius",
        "--main-radius",
        "--btn-radius",
        "--ui-radius",
    ];

    for name in priority_names {
        if let Some(val) = all_vars.get(name) {
            let resolved = resolve_var(val, all_vars);
            if let Some(px) = parse_px(&resolved) {
                return Some(px);
            }
        }
    }

    for (name, val) in all_vars {
        let lower = name.to_lowercase();
        if lower.contains("radius") || lower.contains("corner") {
            let resolved = resolve_var(val, all_vars);
            if let Some(px) = parse_px(&resolved) {
                return Some(px);
            }
        }
    }

    None
}

fn extract_radius_from_rules(rules: &[SpannedRule], all_vars: &BTreeMap<String, String>) -> Option<f64> {
    for rule in rules {
        let norm = normalize_selector(&rule.selector);
        if norm.contains("card") || norm.contains("button") || norm.contains("btn") || norm.contains("dialog") || norm.contains("modal") {
            let decls = parse_decls(&rule.body);
            for (prop, val) in decls {
                if prop == "border-radius" {
                    let resolved = resolve_var(&val, all_vars);
                    if let Some(px) = parse_px(&resolved) {
                        return Some(px);
                    }
                }
            }
        }
    }
    None
}

fn extract_tokens_from_vars(all_vars: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut overrides = BTreeMap::new();
    let derived: Vec<String> = DERIVED_TOKEN_NAMES.iter().map(|n| format!("--fds-{n}")).collect();

    for (name, val) in all_vars.iter() {
        if name.starts_with("--") && !derived.contains(name) {
            let resolved = resolve_var(val, all_vars);
            overrides.insert(name.clone(), resolved);
        }
    }

    let bg_names = [
        "--bg-primary", "--bg-main", "--main-bg", "--background-base", "--bg-color", "--background", "--background-color", "--surface"
    ];
    for name in bg_names {
        if !overrides.contains_key("--fds-solid-background-base") {
            if let Some(val) = all_vars.get(name) {
                let resolved = resolve_var(val, all_vars);
                overrides.insert("--fds-solid-background-base".to_string(), resolved.clone());
                overrides.insert("--fds-card-background-default".to_string(), resolved);
            }
        }
    }

    let text_names = ["--text-primary", "--text-color", "--color-text", "--text-main", "--text"];
    for name in text_names {
        if !overrides.contains_key("--fds-text-primary") {
            if let Some(val) = all_vars.get(name) {
                let resolved = resolve_var(val, all_vars);
                overrides.insert("--fds-text-primary".to_string(), resolved);
            }
        }
    }

    let text_sec_names = ["--text-secondary", "--text-muted", "--text-sub", "--text-dim"];
    for name in text_sec_names {
        if !overrides.contains_key("--fds-text-secondary") {
            if let Some(val) = all_vars.get(name) {
                let resolved = resolve_var(val, all_vars);
                overrides.insert("--fds-text-secondary".to_string(), resolved);
            }
        }
    }

    overrides
}

fn normalize_selector(selector: &str) -> String {
    selector
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_token_block(normalized: &str) -> bool {
    normalized.split(',').any(|part| {
        let p = part.trim();
        p.contains(":root")
            || p == "html"
            || p == "body"
            || p == ":host"
            || p == "*"
            || p.contains("data-theme")
            || p.contains("theme-")
            || p.contains("fds-theme")
    })
}

fn remove_spans(text: &str, spans: &mut Vec<(usize, usize)>) -> String {
    spans.sort_by_key(|(start, _)| *start);

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for &(start, end) in spans.iter() {
        if start < cursor {
            continue;
        }
        out.push_str(&text[cursor..start]);
        cursor = end;
    }
    out.push_str(&text[cursor..]);

    let mut tidy = String::with_capacity(out.len());
    let mut blanks = 0usize;
    for line in out.lines() {
        if line.trim().is_empty() {
            blanks += 1;
            if blanks > 1 {
                continue;
            }
        } else {
            blanks = 0;
        }
        tidy.push_str(line.trim_end());
        tidy.push('\n');
    }

    tidy.trim().to_string()
}

pub fn parse_foreign_css(
    text: &str,
    known_selectors: &[String],
    current: &ThemeDoc,
) -> ThemeDoc {
    let (imports, body) = split_imports(text);

    let known: BTreeMap<String, String> = known_selectors
        .iter()
        .map(|s| (normalize_selector(s), s.clone()))
        .collect();

    let rules = split_rules_spanned(&body);

    let mut all_vars = BTreeMap::new();
    for rule in &rules {
        let decls = parse_decls(&rule.body);
        for (prop, val) in decls {
            if prop.starts_with("--") {
                all_vars.insert(prop, val);
            }
        }
    }

    let accent = extract_accent_from_vars(&all_vars)
        .or_else(|| extract_accent_from_rules(&rules, &all_vars));

    let control_corner_radius = extract_radius_from_vars(&all_vars)
        .or_else(|| extract_radius_from_rules(&rules, &all_vars));

    let explicit_overlay = if let Some(val) = all_vars.get("--fds-overlay-corner-radius") {
        parse_px(&resolve_var(val, &all_vars))
    } else {
        None
    };

    let overlay_corner_radius = explicit_overlay.or_else(|| {
        control_corner_radius.map(|r| (r + 4.0).min(24.0))
    });

    let token_overrides = extract_tokens_from_vars(&all_vars);

    let mut rule_overrides = BTreeMap::new();
    let mut consumed: Vec<(usize, usize)> = Vec::new();

    for rule in &rules {
        let normalized = normalize_selector(&rule.selector);

        if is_token_block(&normalized) {
            let decls = parse_decls(&rule.body);
            let only_custom = !decls.is_empty() && decls.iter().all(|(p, _)| p.starts_with("--"));
            if only_custom {
                consumed.push((rule.start, rule.end));
            }
            continue;
        }

        if let Some(original) = known.get(&normalized) {
            rule_overrides.insert(original.clone(), rule.body.clone());
            consumed.push((rule.start, rule.end));
        }
    }

    ThemeDoc {
        accent: accent.unwrap_or(current.accent),
        mode: current.mode,
        control_corner_radius: control_corner_radius.or(current.control_corner_radius),
        overlay_corner_radius: overlay_corner_radius.or(current.overlay_corner_radius),
        imports,
        token_overrides,
        rule_overrides,
        raw_css: remove_spans(&body, &mut consumed),
    }
}
