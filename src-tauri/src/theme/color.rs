pub type Hsl = [f64; 3];

pub const DEFAULT_ACCENT: Hsl = [206.0, 100.0, 42.0];

pub const RAMP_NAMES: [&str; 7] = [
    "accent-light-3",
    "accent-light-2",
    "accent-light-1",
    "accent-base",
    "accent-dark-1",
    "accent-dark-2",
    "accent-dark-3",
];

const RAMP_OFFSETS: [(f64, f64, f64); 7] = [
    (-15.0, -2.0, 38.0),  // light-3
    (-7.0, -1.0, 27.0),   // light-2
    (-1.0, 0.0, 7.0),     // light-1
    (0.0, 0.0, 0.0),      // base
    (3.0, 0.0, -6.0),     // dark-1
    (9.0, 0.0, -13.0),    // dark-2
    (20.0, 0.0, -22.0),   // dark-3
];

pub fn wrap_hue(h: f64) -> f64 {
    let m = h % 360.0;
    if m < 0.0 {
        m + 360.0
    } else {
        m
    }
}

pub fn num(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if (r - r.round()).abs() < f64::EPSILON {
        format!("{}", r.round() as i64)
    } else {
        format!("{r}")
    }
}

pub fn fmt_triplet(hsl: Hsl) -> String {
    format!("{}, {}%, {}%", num(hsl[0]), num(hsl[1]), num(hsl[2]))
}

pub fn derive_ramp(base: Hsl) -> [Hsl; 7] {
    let mut out = [[0.0; 3]; 7];
    for (i, (dh, ds, dl)) in RAMP_OFFSETS.iter().enumerate() {
        out[i] = [
            wrap_hue(base[0] + dh),
            (base[1] + ds).clamp(0.0, 100.0),
            (base[2] + dl).clamp(0.0, 100.0),
        ];
    }
    out
}

pub fn hex_to_rgb(hex: &str) -> Option<[f64; 3]> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 3 || clean.len() == 4 {
        let r = u8::from_str_radix(&clean[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&clean[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&clean[2..3].repeat(2), 16).ok()?;
        Some([r as f64, g as f64, b as f64])
    } else if clean.len() >= 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some([r as f64, g as f64, b as f64])
    } else {
        None
    }
}

pub fn rgb_to_hsl(r: f64, g: f64, b: f64) -> Hsl {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let l = (max + min) / 2.0;
    let mut h = 0.0;
    if d.abs() > f64::EPSILON {
        if (max - r).abs() < f64::EPSILON {
            h = ((g - b) / d) % 6.0;
        } else if (max - g).abs() < f64::EPSILON {
            h = (b - r) / d + 2.0;
        } else {
            h = (r - g) / d + 4.0;
        }
        h *= 60.0;
        if h < 0.0 {
            h += 360.0;
        }
    }
    let s = if d.abs() < f64::EPSILON {
        0.0
    } else {
        d / (1.0 - (2.0 * l - 1.0).abs())
    };
    [
        (h * 10.0).round() / 10.0,
        ((s * 100.0) * 10.0).round() / 10.0,
        ((l * 100.0) * 10.0).round() / 10.0,
    ]
}

pub fn named_color_to_hsl(name: &str) -> Option<Hsl> {
    match name.to_lowercase().trim() {
        "red" => Some([0.0, 100.0, 50.0]),
        "blue" => Some([240.0, 100.0, 50.0]),
        "green" => Some([120.0, 100.0, 25.0]),
        "lime" => Some([120.0, 100.0, 50.0]),
        "purple" => Some([300.0, 100.0, 25.0]),
        "pink" => Some([350.0, 100.0, 88.0]),
        "magenta" => Some([300.0, 100.0, 50.0]),
        "cyan" => Some([180.0, 100.0, 50.0]),
        "yellow" => Some([60.0, 100.0, 50.0]),
        "orange" => Some([39.0, 100.0, 50.0]),
        "teal" => Some([180.0, 100.0, 25.0]),
        "indigo" => Some([275.0, 100.0, 25.0]),
        "violet" => Some([300.0, 76.0, 72.0]),
        "white" => Some([0.0, 0.0, 100.0]),
        "black" => Some([0.0, 0.0, 0.0]),
        "gray" | "grey" => Some([0.0, 0.0, 50.0]),
        _ => None,
    }
}

pub fn parse_color_to_hsl(value: &str) -> Option<Hsl> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }

    if let Some(named) = named_color_to_hsl(v) {
        return Some(named);
    }

    if v.starts_with('#') {
        let rgb = hex_to_rgb(v)?;
        return Some(rgb_to_hsl(rgb[0], rgb[1], rgb[2]));
    }

    if v.starts_with("rgb") {
        let inner = v.find('(').and_then(|start| v.find(')').map(|end| &v[start + 1..end]))?;
        let parts: Vec<&str> = inner
            .split(|c| c == ',' || c == ' ' || c == '/')
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() >= 3 {
            let r: f64 = parts[0].parse().ok()?;
            let g: f64 = parts[1].parse().ok()?;
            let b: f64 = parts[2].parse().ok()?;
            return Some(rgb_to_hsl(r, g, b));
        }
    }

    let inner = if v.starts_with("hsl") {
        v.find('(').and_then(|start| v.find(')').map(|end| &v[start + 1..end]))?
    } else {
        v
    };

    let parts: Vec<&str> = inner
        .split(|c| c == ',' || c == ' ' || c == '/')
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() >= 3 {
        let h: f64 = parts[0].trim_end_matches("deg").parse().ok()?;
        let s: f64 = parts[1].trim_end_matches('%').parse().ok()?;
        let l: f64 = parts[2].trim_end_matches('%').parse().ok()?;
        return Some([wrap_hue(h), s.clamp(0.0, 100.0), l.clamp(0.0, 100.0)]);
    }

    None
}

pub fn parse_triplet(value: &str) -> Option<Hsl> {
    parse_color_to_hsl(value)
}
