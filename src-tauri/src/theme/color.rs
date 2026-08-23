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

/// `RAMP_OFFSETS`teki ışıklılık farklarının ölçüldüğü taban.
///
/// Kütüphanenin varsayılan vurgusu `206, 100%, 42%`; oradaki basamak farkları
/// (+7, +27, +38 / −6, −13, −22) bu ışıklılığa göre yazılmış.
const REFERENCE_LIGHTNESS: f64 = 42.0;

/// Bir basamağın ışıklılığı — sabit ekleme değil, KALAN BOŞLUĞA ORANLI.
///
/// Sabit ekleme yalnızca taban ışıklılık referansa yakınken doğru sonuç
/// veriyordu. Açık bir vurguda üst basamaklar tavana dayanıp kırpılıyordu:
/// içe aktarılan bir temanın vurgusu `217.9, 86.9%, 76.1%` iken `light-2`
/// 76.1 + 27 = 103.1 → %100, yani DÜPEDÜZ BEYAZ çıkıyordu. Koyu kipte
/// `--fds-accent-default` tam olarak `light-2`den türediği için vurgu rengiyle
/// boyanan her şey (seçili kenar çubuğu ikonu, rozet ucu, oynatıcı rayı)
/// beyaza dönüyordu.
///
/// Oranlı hesapta basamaklar tabanın uzaklığına göre ölçekleniyor: açık bir
/// vurguda üst basamaklar sıkışıyor ama AYRIŞMAYA devam ediyor, koyu bir
/// vurguda alt basamaklar için aynısı geçerli.
///
/// Referans tabanda (42) formül eski davranışla BİREBİR aynı sonucu veriyor —
/// `default_base_produces_library_ramp` testi bunu sabitliyor.
fn ramp_lightness(base_lightness: f64, delta: f64) -> f64 {
    let ratio = if delta >= 0.0 {
        // Yukarı doğru: referanstaki tavan boşluğunun ne kadarı kullanılmış.
        delta / (100.0 - REFERENCE_LIGHTNESS)
    } else {
        -delta / REFERENCE_LIGHTNESS
    };

    if delta >= 0.0 {
        base_lightness + (100.0 - base_lightness) * ratio
    } else {
        base_lightness - base_lightness * ratio
    }
}

/// `--fds-accent-default`ın rampadaki karşılığı olan basamak indeksi.
///
/// Kütüphane bunu KİPE bağlı türetiyor (sitenin canlı CSS'inden okundu):
///
/// ```text
/// .fds-theme-light { --fds-accent-default: hsl(var(--fds-accent-dark-1)) }
/// .fds-theme-dark  { --fds-accent-default: hsla(var(--fds-accent-light-2)) }
/// ```
pub const fn accent_default_step(light_mode: bool) -> usize {
    if light_mode {
        4 // dark-1
    } else {
        1 // light-2
    }
}

/// Verilen rengi `step` basamağında ÜRETEN tabanı geriye çözer.
///
/// Neden gerekli: sitenin vurguyla boyadığı her şey (seçili menü göstergesi,
/// seçili ikon, bağlantılar) `--fds-accent-default` kullanıyor — rampanın
/// TABANINI değil. Bir tema kendi vurgu rengini doğrudan boyamada
/// kullandığında o renk `accent-default`a karşılık gelir.
///
/// İçe aktarmada o renk doğrudan taban yapılırsa sitenin çizdiği öğeler bir
/// basamak kayıyor: koyu kipte `light-2` tabandan daha açık olduğu için
/// gösterge çubuğu ve ikonlar temanın kendi renginden görünür biçimde açık
/// çıkıyordu. Tabanı geriye çözünce ikisi çakışıyor.
///
/// `derive_ramp`in tersi: ton ve doygunluk farkları çıkarılıyor, ışıklılık ise
/// `ramp_lightness` tersine çevriliyor.
pub fn base_for_step(target: Hsl, step: usize) -> Hsl {
    let (dh, ds, dl) = RAMP_OFFSETS[step.min(RAMP_OFFSETS.len() - 1)];

    let lightness = if dl >= 0.0 {
        let k = dl / (100.0 - REFERENCE_LIGHTNESS);
        // l' = l(1-k) + 100k  ->  l = (l' - 100k) / (1-k)
        (target[2] - 100.0 * k) / (1.0 - k)
    } else {
        let k = -dl / REFERENCE_LIGHTNESS;
        // l' = l(1-k)  ->  l = l' / (1-k)
        target[2] / (1.0 - k)
    };

    [
        wrap_hue(target[0] - dh),
        (target[1] - ds).clamp(0.0, 100.0),
        lightness.clamp(0.0, 100.0),
    ]
}

pub fn derive_ramp(base: Hsl) -> [Hsl; 7] {
    let mut out = [[0.0; 3]; 7];
    for (i, (dh, ds, dl)) in RAMP_OFFSETS.iter().enumerate() {
        out[i] = [
            wrap_hue(base[0] + dh),
            (base[1] + ds).clamp(0.0, 100.0),
            // Kırpma güvenlik ağı olarak duruyor; oranlı hesap zaten
            // 0–100 aralığından çıkmıyor.
            ramp_lightness(base[2], *dl).clamp(0.0, 100.0),
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

/// Bir rengin ALFASI (0.0–1.0). Renk okunamazsa `None`, alfa yazılmamışsa 1.0.
///
/// `parse_color_to_hsl` alfayı düşürüyor ve bu, vurgu ailesinde gözle görülür
/// bir hataya yol açıyordu: temanın `rgba(141, 180, 247, 0.04)` gibi %4'lük
/// tint'i kaydırıcıya bağlandığında opak `hsl(...)` olarak geri yazılıyor,
/// ince bir arka plan tonu OLDUĞU GİBİ dolu bir vurgu bloğuna dönüşüyordu
/// (arama kutusu, seçili menü öğesi, kart kenarlıkları).
pub fn parse_alpha(value: &str) -> Option<f64> {
    let mut v = value.trim();
    if v.ends_with("!important") {
        v = v.trim_end_matches("!important").trim();
    }

    // `#rrggbbaa` / `#rgba`
    if let Some(hex) = v.strip_prefix('#') {
        let hex: String = hex.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
        return match hex.len() {
            4 => u8::from_str_radix(&hex[3..4].repeat(2), 16).ok().map(|a| a as f64 / 255.0),
            8 => u8::from_str_radix(&hex[6..8], 16).ok().map(|a| a as f64 / 255.0),
            3 | 6 => Some(1.0),
            _ => None,
        };
    }

    if !(v.starts_with("rgb") || v.starts_with("hsl")) {
        return Some(1.0);
    }

    let inner = v.find('(').and_then(|start| v.find(')').map(|end| &v[start + 1..end]))?;
    let parts: Vec<&str> = inner
        .split(|c| c == ',' || c == ' ' || c == '/')
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 4 {
        return Some(1.0);
    }
    let raw = parts[3].trim();
    let (number, percent) = match raw.strip_suffix('%') {
        Some(n) => (n, true),
        None => (raw, false),
    };
    let value: f64 = number.parse().ok()?;
    Some(if percent { value / 100.0 } else { value }.clamp(0.0, 1.0))
}

/// Rengin "renkliliği" (0.0–1.0): HSL doygunluğunun, açıklığın uçlara
/// yaklaştıkça sönmesini hesaba katan hâli.
///
/// Ham doygunluk tek başına yanıltıcı: `#e8eaf2` (neredeyse beyaz) %27.8
/// doygunluk raporluyor, `#4a4f68` (arduvaz grisi) %16.9. İkisi de bir
/// doygunluk eşiğini geçip "vurgu ailesi" sayılıyordu ve kullanıcı vurgu
/// rengini oynattığında temanın BEYAZ METNİ ile gri tonları da kayıyordu.
/// Kroma bu ikisini 0.04 ve 0.12'ye indiriyor; gerçek vurgu tonları
/// (`#8db4f7` 0.42, `#6b9ef5` 0.54, `#b8d4ff` 0.28) yukarıda kalıyor.
pub fn chroma(hsl: Hsl) -> f64 {
    let lightness = hsl[2] / 100.0;
    (1.0 - (2.0 * lightness - 1.0).abs()) * (hsl[1] / 100.0)
}

pub fn parse_color_to_hsl(value: &str) -> Option<Hsl> {
    let mut v = value.trim();
    if v.ends_with("!important") {
        v = v.trim_end_matches("!important").trim();
    }
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
