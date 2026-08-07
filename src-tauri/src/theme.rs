//! Tema modeli ve CSS üreticisi.
//!
//! Burası projenin TEK CSS kaynağıdır. Hem önizlemeye enjekte edilen string
//! hem de kullanıcının dışa aktardığı dosya bu modülden çıkar — böylece
//! "önizlemede başka, export'ta başka" sınıfı hatalar yapısal olarak imkânsız.
//!
//! Üretilen CSS'in şekli için bkz. PLAN.md §2.5.

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

pub type Hsl = [f64; 3];

/// Üretilen CSS'te "editörün yönettiği" bölgeyi sınırlayan işaretleyiciler.
///
/// Kod editörü modunda kullanıcı CSS'i serbestçe düzenleyebiliyor. Bu iki
/// yorum, hangi kısmın kontrollerden türediğini (dolayısıyla yeniden
/// üretilebilir olduğunu) ve hangi kısmın kullanıcıya ait olduğunu
/// belirsizliğe yer bırakmadan ayırır — parse ↔ emit turu böylece kararlı olur.
pub const TOKENS_OPEN: &str = "/* <oa:tokens> */";
pub const TOKENS_CLOSE: &str = "/* </oa:tokens> */";

/// Uygulama genelinde paylaşılan taslak tema.
#[derive(Default)]
pub struct ThemeState(pub Mutex<ThemeDoc>);

/// fluent-svelte-extra `theme.css` içindeki varsayılan accent tabanı.
pub const DEFAULT_ACCENT: Hsl = [206.0, 100.0, 42.0];

/// `--fds-accent-*` token adları (açıktan koyuya).
pub const RAMP_NAMES: [&str; 7] = [
    "accent-light-3",
    "accent-light-2",
    "accent-light-1",
    "accent-base",
    "accent-dark-1",
    "accent-dark-2",
    "accent-dark-3",
];

/// Taban renge göre (H, S, L) ofsetleri.
///
/// Bu sayılar uydurma değil: fluent-svelte-extra'nın `theme.css` dosyasındaki
/// yedi varsayılan değerin `--fds-accent-base`'e (206, 100%, 42%) göre farkı
/// alınarak çıkarıldı. Dolayısıyla taban renk varsayılana eşitken üretilen
/// rampa, kütüphanenin varsayılanlarıyla birebir aynı olur.
const RAMP_OFFSETS: [(f64, f64, f64); 7] = [
    (-15.0, -2.0, 38.0),  // light-3
    (-7.0, -1.0, 27.0),   // light-2
    (-1.0, 0.0, 7.0),     // light-1
    (0.0, 0.0, 0.0),      // base
    (3.0, 0.0, -6.0),     // dark-1
    (9.0, 0.0, -13.0),    // dark-2
    (20.0, 0.0, -22.0),   // dark-3
];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    /// Sitenin `localStorage.theme` sözleşmesi: 0=system, 1=light, 2=dark.
    pub fn as_site_value(self) -> u8 {
        match self {
            ThemeMode::System => 0,
            ThemeMode::Light => 1,
            ThemeMode::Dark => 2,
        }
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ThemeDoc {
    pub accent: Hsl,
    pub mode: ThemeMode,
    /// `--fds-control-corner-radius` (px). `None` = siteye dokunma.
    pub control_corner_radius: Option<f64>,
    /// `--fds-overlay-corner-radius` (px). `None` = siteye dokunma.
    pub overlay_corner_radius: Option<f64>,
    /// `@import` satırları (yazı tipi yüklemek için).
    ///
    /// Ayrı bir alan olmalarının sebebi teknik: CSS `@import`'un dosyanın EN
    /// BAŞINDA olmasını şart koşar, sonra gelen bir `@import` sessizce yok
    /// sayılır. Bu yüzden bunlar işaretleyici bloğundan da önce yazılır.
    #[serde(default)]
    pub imports: Vec<String>,

    /// Doğrudan token ezmeleri: tam property adı -> değer.
    /// Örn. `"--fds-subtle-fill-secondary" -> "hsla(0,0%,100%,6%)"`.
    ///
    /// Hover renkleri, geçiş süreleri, buton dolguları gibi "tek token = tek
    /// ayar" olan her yeni seçenek buradan geçer. Jenerik olduğu için kod
    /// editörü ↔ kontrol senkronu yeni seçenek eklendiğinde kendiliğinden
    /// çalışır; Rust'ın seçeneğin ne anlama geldiğini bilmesi gerekmez.
    #[serde(default)]
    pub token_overrides: BTreeMap<String, String>,

    /// Karşılığında token BULUNMAYAN şeyler için selector -> bildirim gövdesi.
    /// Örn. logo değişimi (`.topbar .logo img`) ya da buton metin rengi
    /// (`.button`), çünkü sitede bunlar için ayrılmış bir `--fds-*` yok.
    #[serde(default)]
    pub rule_overrides: BTreeMap<String, String>,

    /// Kullanıcının serbestçe yazdığı ham CSS (kaçış kapağı).
    pub raw_css: String,
}

impl Default for ThemeDoc {
    fn default() -> Self {
        Self {
            accent: DEFAULT_ACCENT,
            mode: ThemeMode::default(),
            control_corner_radius: None,
            overlay_corner_radius: None,
            imports: Vec::new(),
            token_overrides: BTreeMap::new(),
            rule_overrides: BTreeMap::new(),
            raw_css: String::new(),
        }
    }
}

fn wrap_hue(h: f64) -> f64 {
    let m = h % 360.0;
    if m < 0.0 {
        m + 360.0
    } else {
        m
    }
}

/// Sayıyı gereksiz ondalık olmadan yazar: 42.0 -> "42", 42.5 -> "42.5".
fn num(v: f64) -> String {
    let r = (v * 100.0).round() / 100.0;
    if (r - r.round()).abs() < f64::EPSILON {
        format!("{}", r.round() as i64)
    } else {
        format!("{r}")
    }
}

/// Tek bir accent basamağını sitenin beklediği formata çevirir.
///
/// Dikkat: `hsl()` sarmalayıcısı YOK. Site bu değeri
/// `hsl(var(--fds-accent-dark-1))` şeklinde kullanıyor, yani token'ın kendisi
/// çıplak bir triplet olmak zorunda.
pub fn fmt_triplet(hsl: Hsl) -> String {
    format!("{}, {}%, {}%", num(hsl[0]), num(hsl[1]), num(hsl[2]))
}

/// Taban renkten yedi basamaklı accent rampasını türetir.
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

impl ThemeDoc {
    fn accent_is_default(&self) -> bool {
        self.accent
            .iter()
            .zip(DEFAULT_ACCENT.iter())
            .all(|(a, b)| (a - b).abs() < 0.001)
    }

    /// Sitenin `localStorage.theme_content`'ine yazılacak CSS'i üretir.
    ///
    /// Yalnızca kullanıcının GERÇEKTEN değiştirdiği token'lar yazılır.
    /// Dokunulmayanlar sitenin kendi varsayılanında kalır — böylece çıktı küçük
    /// olur ve site ileride varsayılanlarını güncellerse tema onunla birlikte
    /// evrilir.
    pub fn emit_css(&self) -> String {
        let mut root: Vec<String> = Vec::new();

        if !self.accent_is_default() {
            for (name, hsl) in RAMP_NAMES.iter().zip(derive_ramp(self.accent).iter()) {
                root.push(format!("\t--fds-{}: {};", name, fmt_triplet(*hsl)));
            }
        }
        if let Some(r) = self.control_corner_radius {
            root.push(format!("\t--fds-control-corner-radius: {}px;", num(r)));
        }
        if let Some(r) = self.overlay_corner_radius {
            root.push(format!("\t--fds-overlay-corner-radius: {}px;", num(r)));
        }
        // Jenerik token ezmeleri (hover, süreler, buton dolguları …).
        // BTreeMap olduğu için sıra deterministik — çıktı her seferinde aynı.
        for (name, value) in &self.token_overrides {
            root.push(format!("\t{}: {};", name.trim(), value.trim()));
        }

        let mut css = String::new();

        // --- @import'lar en başta olmak ZORUNDA ---
        for url in &self.imports {
            let url = url.trim();
            if url.is_empty() {
                continue;
            }
            css.push_str(&format!("@import url(\"{url}\");\n"));
        }
        if !self.imports.is_empty() {
            css.push('\n');
        }

        // --- Editörün yönettiği bölge ---
        css.push_str(TOKENS_OPEN);
        css.push_str(
            "\n/* Bu blok görsel kontrollerden üretilir; elle düzenlerseniz\n   \
             kontroller de güncellenir. Blok dışına yazdığınız her şey korunur. */\n",
        );
        if !root.is_empty() {
            // Mod-bağımsız token'lar. Üç selector birden yazılıyor ki kullanıcı
            // hangi modda olursa olsun aynı değeri görsün.
            css.push_str(":root,\n.fds-theme-light,\n.fds-theme-dark {\n");
            css.push_str(&root.join("\n"));
            css.push_str("\n}\n");
        }
        // Token karşılığı olmayan kurallar (logo, buton metni …).
        for (selector, body) in &self.rule_overrides {
            css.push_str(selector.trim());
            css.push_str(" {\n\t");
            css.push_str(body.trim());
            css.push_str("\n}\n");
        }
        css.push_str(TOKENS_CLOSE);
        css.push('\n');

        // --- Kullanıcıya ait bölge ---
        if !self.raw_css.trim().is_empty() {
            css.push('\n');
            css.push_str(self.raw_css.trim());
            css.push('\n');
        }

        css
    }
}

/// `--fds-x` gibi bir bildirimin değerini bulur.
///
/// Regex yerine elle tarama: token adının bir başka adın son eki olmadığını
/// (`--fds-accent-base` ile `--x--fds-accent-base` karışmasın) ve hemen
/// ardından `:` geldiğini doğrular.
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

/// `"206, 100%, 42%"` -> `[206.0, 100.0, 42.0]`
fn parse_triplet(value: &str) -> Option<Hsl> {
    let parts: Vec<&str> = value.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return None;
    }
    let h: f64 = parts[0].trim_end_matches("deg").trim().parse().ok()?;
    let s: f64 = parts[1].trim_end_matches('%').trim().parse().ok()?;
    let l: f64 = parts[2].trim_end_matches('%').trim().parse().ok()?;
    Some([h, s, l])
}

/// `"12px"` -> `12.0`
fn parse_px(value: &str) -> Option<f64> {
    value.trim().trim_end_matches("px").trim().parse().ok()
}

/// `/* … */` yorumlarını atar.
///
/// base64 alfabesinde `*` yok, dolayısıyla logo data URI'lerinin içinde
/// yanlışlıkla yorum sınırı oluşamaz.
fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out, // kapanmamış yorum: gerisini at
        }
    }
    out.push_str(rest);
    out
}

/// Bir CSS bloğunu `(selector, gövde)` çiftlerine ayırır.
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

/// Bir kural gövdesindeki `prop: value` bildirimlerini ayırır.
///
/// Yalnızca `:root` bloğu için kullanılır — orada değerler `;` içermez.
/// Diğer kuralların gövdesi (logo data URI'si gibi `;` barındırabilenler)
/// hiç ayrıştırılmaz, olduğu gibi saklanır.
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

/// `@import` satırlarını metinden ayırır.
///
/// CSS `@import`'un dosyanın en başında olmasını şart koşuyor; ham CSS'e
/// karışırlarsa bir sonraki `emit_css` turunda dosyanın ortasına düşüp
/// sessizce geçersiz olurlar. Bu yüzden ayrı bir alanda taşınıyorlar.
fn split_imports(text: &str) -> (Vec<String>, String) {
    let mut imports = Vec::new();
    let mut rest = String::with_capacity(text.len());

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("@import") {
            // url("…") ya da '…' / çıplak biçimlerinden URL'yi çek.
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

/// `emit_css`'in tersi: kod editöründeki metni tekrar `ThemeDoc`'a çevirir.
///
/// `current`, CSS'te temsil edilmeyen alanlar (açık/koyu modu gibi) için
/// kullanılır — parse bunları sıfırlamaz.
///
/// İşaretleyiciler bulunamazsa (kullanıcı sildiyse) kontrollerin değerleri
/// olduğu gibi korunur ve metnin tamamı kullanıcı CSS'i sayılır; böylece
/// beklenmedik bir sıfırlama yaşanmaz.
pub fn parse_css(text: &str, current: &ThemeDoc) -> ThemeDoc {
    // `@import` satırlarını ayır — bunlar ham CSS'e karışmamalı, yoksa bir
    // sonraki emit'te dosyanın ortasına düşüp geçersiz olurlar.
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
        return ThemeDoc {
            raw_css: raw,
            imports,
            ..current.clone()
        };
    };

    // Yönetilen bölgeyi kurallara ayır: `:root…` bloğu token'ları taşır,
    // geri kalan her kural (logo, buton metni …) olduğu gibi korunur.
    let cleaned = strip_comments(managed);
    let mut rule_overrides = BTreeMap::new();
    let mut root_body = String::new();

    for (selector, body) in split_rules(&cleaned) {
        let normalized = selector.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.starts_with(":root") {
            root_body.push_str(&body);
            root_body.push(';');
        } else {
            rule_overrides.insert(normalized, body);
        }
    }

    // Türetilmiş accent basamakları ve yapısal alanlar haritaya girmez;
    // onlar kendi alanlarından yeniden üretilir.
    let derived: Vec<String> = RAMP_NAMES.iter().map(|n| format!("--fds-{n}")).collect();
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
        // Blokta accent yoksa kullanıcı onu silmiş demektir -> varsayılana dön.
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

// ---------------------------------------------------------------------------
// Dışarıdan gelen (editörün üretmediği) CSS'i kontrollere eşleme
// ---------------------------------------------------------------------------

/// Ham metindeki yerini de taşıyan bir CSS kuralı.
///
/// Konum bilgisi şart: içe aktarmada kontrollere eşlenen kuralları ham CSS'ten
/// ÇIKARMAK, eşlenemeyenleri ise metinde olduğu gibi (yorumlarıyla birlikte)
/// bırakmak istiyoruz. Yalnız (selector, gövde) çiftleri elde etseydik ham
/// bölümü yeniden yazmak zorunda kalır, kullanıcının biçimlendirmesini ve
/// yorumlarını kaybederdik.
struct SpannedRule {
    selector: String,
    body: String,
    start: usize,
    end: usize,
}

/// Üst düzey kuralları, yorum ve string sınırlarına saygı göstererek ayırır.
///
/// `split_rules`'tan farkı: burada tarayıcı `/* … */` içindeki ve tırnak
/// içindeki süslü parantezleri saymaz. İçe aktarılan temalarda bunlar gerçekten
/// oluyor (ör. `content: "}"` ya da yorum içine alınmış eski kurallar) ve
/// sayılsalardı kural sınırları kayardı.
///
/// İç içe bloklar (`@media { … }`) tek bir üst düzey kural olarak döner;
/// gövdeleri açılmaz. Bu bilinçli: mod'a özel `:root` blokları tek bir
/// kontrol setine indirilemez, dolayısıyla ham CSS'te olduğu gibi kalmalılar.
fn split_rules_spanned(text: &str) -> Vec<SpannedRule> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut selector_start = 0usize;

    /// Yorum veya string başlıyorsa sonundaki konumu döner.
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
                // Kaçırılmış tırnak stringi bitirmez.
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

/// Karşılaştırma için selector'ı tek biçime indirir.
///
/// Hem boşlukları hem virgül aralıklarını normalleştiriyoruz; aksi hâlde
/// `.a,.b` ile `.a, .b` farklı sayılır ve içe aktarılan tema tanınmazdı.
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

/// Selector, tema token'larının tanımlandığı bir blok mu?
fn is_token_block(normalized: &str) -> bool {
    normalized.split(',').any(|part| {
        let p = part.trim();
        p.starts_with(":root") || p == "html" || p.starts_with(".fds-theme-")
    })
}

/// Verilen aralıkları metinden çıkarır ve arta kalan boş satırları toparlar.
fn remove_spans(text: &str, spans: &mut Vec<(usize, usize)>) -> String {
    spans.sort_by_key(|(start, _)| *start);

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for &(start, end) in spans.iter() {
        // Örtüşen aralık (olmamalı, ama sessizce bozulmaktansa atla).
        if start < cursor {
            continue;
        }
        out.push_str(&text[cursor..start]);
        cursor = end;
    }
    out.push_str(&text[cursor..]);

    // Çıkarılan kuralların ardında üst üste boş satırlar kalıyor.
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

/// Dışarıdan gelen bir temayı kontrollere eşler.
///
/// `parse_css`'ten AYRI olmasının sebebi davranış farkı: o fonksiyon editörün
/// kendi işaretleyici bloğuna güvenir ve blok yoksa metnin tamamını ham CSS
/// sayar — kod editörü ↔ kontrol turunun kararlı olması için doğru olan bu.
/// GitHub'dan çekilen bir temada ise işaretleyici hiç bulunmaz, dolayısıyla o
/// yol her şeyi ham CSS'e bırakırdı. Burada eşlemeyi yaparken mevcut davranışa
/// hiç dokunmuyoruz.
///
/// `known_selectors`: uygulamanın kontrol paneline bağladığı selector'lar.
/// Listeyi frontend veriyor (`$lib/advanced.ts`); tek doğruluk kaynağı orada
/// kalsın diye Rust'a kopyalanmıyor.
///
/// **Kayıp yok garantisi:** eşlenemeyen her kural ham CSS'te olduğu gibi
/// kalır, yani içe aktarılan tema görsel olarak eksiksiz uygulanır.
pub fn parse_foreign_css(
    text: &str,
    known_selectors: &[String],
    current: &ThemeDoc,
) -> ThemeDoc {
    let (imports, body) = split_imports(text);

    // Normalleştirilmiş hâl -> frontend'in verdiği ÖZGÜN hâl.
    // Anahtarı özgün hâliyle saklamak şart: `adoptDoc` kuralları
    // `advanced.ts`'teki sabitlerle birebir arıyor.
    let known: BTreeMap<String, String> = known_selectors
        .iter()
        .map(|s| (normalize_selector(s), s.clone()))
        .collect();

    let derived: Vec<String> = RAMP_NAMES.iter().map(|n| format!("--fds-{n}")).collect();

    let mut token_overrides = BTreeMap::new();
    let mut rule_overrides = BTreeMap::new();
    let mut accent: Option<Hsl> = None;
    let mut control_corner_radius: Option<f64> = None;
    let mut overlay_corner_radius: Option<f64> = None;
    let mut consumed: Vec<(usize, usize)> = Vec::new();

    for rule in split_rules_spanned(&body) {
        let normalized = normalize_selector(&rule.selector);

        if is_token_block(&normalized) {
            let decls = parse_decls(&rule.body);

            // Bloğu ancak TAMAMI custom property ise devralıyoruz. İçinde
            // sıradan bir bildirim de varsa (ör. `:root { color-scheme: dark }`)
            // bloğu bölmek zorunda kalırdık; bunun yerine olduğu gibi ham
            // CSS'te bırakıyoruz — hem uygulanır hem de bir şey kaybolmaz.
            let only_custom = !decls.is_empty() && decls.iter().all(|(p, _)| p.starts_with("--"));
            let has_fds = decls.iter().any(|(p, _)| p.starts_with("--fds-"));

            if only_custom && has_fds {
                for (prop, value) in decls {
                    match prop.as_str() {
                        "--fds-accent-base" => accent = parse_triplet(&value).or(accent),
                        "--fds-control-corner-radius" => {
                            control_corner_radius = parse_px(&value).or(control_corner_radius)
                        }
                        "--fds-overlay-corner-radius" => {
                            overlay_corner_radius = parse_px(&value).or(overlay_corner_radius)
                        }
                        // Türetilen accent basamakları taban renkten yeniden
                        // üretiliyor; haritaya girerlerse çift yazılırlardı.
                        _ if derived.iter().any(|d| d == &prop) => {}
                        _ => {
                            token_overrides.insert(prop, value);
                        }
                    }
                }
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
        accent: accent.unwrap_or(DEFAULT_ACCENT),
        // Mod CSS'te temsil edilmiyor; kullanıcının seçimi korunuyor.
        mode: current.mode,
        control_corner_radius,
        overlay_corner_radius,
        imports,
        token_overrides,
        rule_overrides,
        raw_css: remove_spans(&body, &mut consumed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varsayilan_taban_kutuphane_rampasini_uretir() {
        // theme.css'teki yedi varsayılan değerin birebir aynısı çıkmalı.
        let expected = [
            "191, 98%, 80%",
            "199, 99%, 69%",
            "205, 100%, 49%",
            "206, 100%, 42%",
            "209, 100%, 36%",
            "215, 100%, 29%",
            "226, 100%, 20%",
        ];
        let ramp = derive_ramp(DEFAULT_ACCENT);
        for (got, want) in ramp.iter().zip(expected.iter()) {
            assert_eq!(&fmt_triplet(*got), want);
        }
    }

    #[test]
    fn varsayilan_dokuman_token_yazmaz() {
        let css = ThemeDoc::default().emit_css();
        assert!(!css.contains("--fds-"), "beklenmedik token: {css}");
    }

    #[test]
    fn degistirilmis_accent_yedi_basamak_yazar() {
        let doc = ThemeDoc {
            accent: [280.0, 80.0, 50.0],
            ..Default::default()
        };
        let css = doc.emit_css();
        for name in RAMP_NAMES {
            assert!(css.contains(&format!("--fds-{name}:")), "eksik: {name}");
        }
        assert!(css.contains(":root,\n.fds-theme-light,\n.fds-theme-dark"));
    }

    #[test]
    fn hue_360_etrafinda_sarmalanir() {
        let ramp = derive_ramp([355.0, 100.0, 42.0]);
        // dark-3 ofseti +20 => 375 => 15
        assert_eq!(num(ramp[6][0]), "15");
        // light-3 ofseti -15 => 340
        assert_eq!(num(ramp[0][0]), "340");
    }

    #[test]
    fn doygunluk_ve_isik_kirpilir() {
        let ramp = derive_ramp([200.0, 1.0, 95.0]);
        assert!(ramp.iter().all(|c| c[1] >= 0.0 && c[1] <= 100.0));
        assert!(ramp.iter().all(|c| c[2] >= 0.0 && c[2] <= 100.0));
    }

    /// Kod editörü ↔ görsel kontrol iki yönlü senkronun temeli:
    /// emit -> parse turu dokümanı aynen geri vermeli.
    #[test]
    fn emit_parse_turu_dokumani_korur() {
        let original = ThemeDoc {
            accent: [280.0, 72.5, 48.0],
            mode: ThemeMode::Light,
            control_corner_radius: Some(10.0),
            overlay_corner_radius: Some(18.0),
            raw_css: "body { letter-spacing: .2px; }".into(),
            ..Default::default()
        };

        let round = parse_css(&original.emit_css(), &original);

        assert_eq!(round.accent, original.accent);
        assert_eq!(round.control_corner_radius, Some(10.0));
        assert_eq!(round.overlay_corner_radius, Some(18.0));
        assert_eq!(round.raw_css, "body { letter-spacing: .2px; }");
        // Mod CSS'te temsil edilmiyor; parse onu bozmamalı.
        assert_eq!(round.mode, ThemeMode::Light);
    }

    /// Yeni seçenekler (hover, animasyon, buton) jenerik haritadan geçtiği için
    /// kod editörü senkronu onlar için de kendiliğinden çalışmalı.
    #[test]
    fn token_ve_kural_ezmeleri_tur_atlatir() {
        let mut token_overrides = BTreeMap::new();
        token_overrides.insert("--fds-subtle-fill-secondary".into(), "hsla(0, 0%, 100%, 8%)".into());
        token_overrides.insert("--fds-control-normal-duration".into(), "500ms".into());

        let mut rule_overrides = BTreeMap::new();
        rule_overrides.insert(
            ".topbar .logo img".into(),
            "content: url(\"data:image/png;base64,iVBORw0KGgo=\");".into(),
        );
        rule_overrides.insert(".button".into(), "color: #ff0000;".into());

        let original = ThemeDoc {
            accent: [120.0, 60.0, 40.0],
            control_corner_radius: Some(6.0),
            token_overrides: token_overrides.clone(),
            rule_overrides: rule_overrides.clone(),
            raw_css: "footer { opacity: .5; }".into(),
            ..Default::default()
        };

        let round = parse_css(&original.emit_css(), &original);

        assert_eq!(round.token_overrides, token_overrides);
        assert_eq!(round.rule_overrides, rule_overrides);
        assert_eq!(round.accent, [120.0, 60.0, 40.0]);
        assert_eq!(round.control_corner_radius, Some(6.0));
        assert_eq!(round.raw_css, "footer { opacity: .5; }");
    }

    /// Logo data URI'si `;` içeriyor (`data:image/png;base64,`). Kural gövdeleri
    /// hiç ayrıştırılmadığı için bu bozulmamalı.
    /// `@import` en başta olmak zorunda; tur atlarken hem korunmalı hem de
    /// ham CSS'e sızmamalı.
    #[test]
    fn import_satirlari_en_basta_kalir_ve_tur_atlar() {
        let doc = ThemeDoc {
            imports: vec![
                "https://fonts.googleapis.com/css2?family=Inter&display=swap".into(),
            ],
            raw_css: "body { color: red; }".into(),
            ..Default::default()
        };

        let css = doc.emit_css();
        assert!(
            css.starts_with("@import url(\"https://fonts.googleapis.com"),
            "@import dosyanın en başında olmalı: {css}"
        );

        let round = parse_css(&css, &doc);
        assert_eq!(round.imports, doc.imports);
        assert_eq!(round.raw_css, "body { color: red; }");
        assert!(!round.raw_css.contains("@import"), "ham CSS'e sızmamalı");
    }

    #[test]
    fn logo_data_uri_bozulmaz() {
        let uri = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciLz4=";
        let mut rule_overrides = BTreeMap::new();
        rule_overrides.insert(".topbar .logo img".into(), format!("content: url(\"{uri}\");"));

        let doc = ThemeDoc {
            rule_overrides,
            ..Default::default()
        };
        let round = parse_css(&doc.emit_css(), &doc);

        assert_eq!(
            round.rule_overrides.get(".topbar .logo img").map(String::as_str),
            Some(format!("content: url(\"{uri}\");").as_str())
        );
    }

    #[test]
    fn turetilmis_accent_basamaklari_haritaya_sizmaz() {
        let doc = ThemeDoc {
            accent: [10.0, 50.0, 50.0],
            ..Default::default()
        };
        let round = parse_css(&doc.emit_css(), &doc);
        // Yedi basamak CSS'e yazılır ama override haritasına girmemeli,
        // yoksa bir sonraki emit'te iki kez yazılırlardı.
        assert!(round.token_overrides.is_empty(), "sızan: {:?}", round.token_overrides);
    }

    #[test]
    fn strip_comments_kapanmamis_yorumu_yutar() {
        assert_eq!(strip_comments("a /* b */ c"), "a  c");
        assert_eq!(strip_comments("a /* b"), "a ");
    }

    #[test]
    fn split_rules_ic_ice_bloklari_tek_kural_sayar() {
        let rules = split_rules("@media (min-width: 5px) { .a { color: red; } } .b { color: blue; }");
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[1].0, ".b");
    }

    #[test]
    fn varsayilan_dokuman_da_tur_atlatir() {
        let original = ThemeDoc::default();
        let round = parse_css(&original.emit_css(), &original);
        assert_eq!(round.accent, DEFAULT_ACCENT);
        assert_eq!(round.control_corner_radius, None);
        assert_eq!(round.raw_css, "");
    }

    /// Kullanıcı kod editöründe accent satırını elle değiştirirse kontrol
    /// güncellenmeli.
    #[test]
    fn elle_duzenlenen_accent_kontrole_yansir() {
        let doc = ThemeDoc {
            accent: [206.0, 100.0, 42.0],
            ..Default::default()
        };
        let edited = format!(
            "{TOKENS_OPEN}\n:root {{\n\t--fds-accent-base: 120, 60%, 40%;\n}}\n{TOKENS_CLOSE}\n"
        );
        let parsed = parse_css(&edited, &doc);
        assert_eq!(parsed.accent, [120.0, 60.0, 40.0]);
    }

    #[test]
    fn isaretleyici_yoksa_kontroller_korunur_metin_ham_sayilir() {
        let current = ThemeDoc {
            accent: [300.0, 50.0, 50.0],
            control_corner_radius: Some(9.0),
            ..Default::default()
        };
        let parsed = parse_css("body { color: red; }", &current);
        assert_eq!(parsed.accent, [300.0, 50.0, 50.0], "accent sıfırlanmamalı");
        assert_eq!(parsed.control_corner_radius, Some(9.0));
        assert_eq!(parsed.raw_css, "body { color: red; }");
    }

    #[test]
    fn blok_disina_yazilan_css_korunur() {
        let doc = ThemeDoc::default();
        let text = format!("h1 {{ color: red; }}\n{TOKENS_OPEN}\n{TOKENS_CLOSE}\nh2 {{ color: blue; }}");
        let parsed = parse_css(&text, &doc);
        assert!(parsed.raw_css.contains("h1"));
        assert!(parsed.raw_css.contains("h2"));
    }

    #[test]
    fn find_decl_benzer_isimleri_karistirmaz() {
        let block = "--fds-control-corner-radius: 4px; --fds-overlay-corner-radius: 8px;";
        assert_eq!(find_decl(block, "--fds-control-corner-radius"), Some("4px"));
        assert_eq!(find_decl(block, "--fds-overlay-corner-radius"), Some("8px"));
        // Son ek olarak eşleşmemeli.
        assert_eq!(find_decl("--x--fds-accent-base: 1,2%,3%;", "--fds-accent-base"), None);
    }

    #[test]
    fn yaricap_ve_ham_css_yazilir() {
        let doc = ThemeDoc {
            control_corner_radius: Some(12.0),
            overlay_corner_radius: Some(20.0),
            raw_css: "body { letter-spacing: .2px; }".into(),
            ..Default::default()
        };
        let css = doc.emit_css();
        assert!(css.contains("--fds-control-corner-radius: 12px;"));
        assert!(css.contains("--fds-overlay-corner-radius: 20px;"));
        assert!(css.contains("letter-spacing"));
    }

    // --- İçe aktarma: yabancı CSS eşlemesi ---------------------------------

    /// İçe aktarmada kullanılan örnek selector listesi (frontend'in
    /// `advanced.ts`'ten gönderdiğinin küçük bir alt kümesi).
    fn known() -> Vec<String> {
        vec![
            ".anime-card, .slider-card, .grid-view-item".to_string(),
            ".topbar a.logo::before".to_string(),
        ]
    }

    #[test]
    fn yabanci_css_token_ve_kurallari_kontrollere_esler() {
        let doc = ThemeDoc::default();
        let css = r#"
:root {
    --fds-accent-base: 340, 82%, 52%;
    --fds-control-corner-radius: 10px;
    --fds-overlay-corner-radius: 18px;
    --fds-text-primary: hsl(0, 0%, 95%);
}

.anime-card, .slider-card, .grid-view-item {
    border-radius: 14px;
}
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.accent, [340.0, 82.0, 52.0]);
        assert_eq!(parsed.control_corner_radius, Some(10.0));
        assert_eq!(parsed.overlay_corner_radius, Some(18.0));
        assert_eq!(
            parsed.token_overrides.get("--fds-text-primary").map(String::as_str),
            Some("hsl(0, 0%, 95%)")
        );
        // Kural, frontend'in verdiği ÖZGÜN anahtar altında durmalı —
        // `adoptDoc` onu bu adla arıyor.
        assert_eq!(
            parsed
                .rule_overrides
                .get(".anime-card, .slider-card, .grid-view-item")
                .map(String::as_str),
            Some("border-radius: 14px;")
        );
        // Eşlenen her şey ham CSS'ten çıkmalı, yoksa iki kez yazılırdı.
        assert!(parsed.raw_css.trim().is_empty(), "kalan: {:?}", parsed.raw_css);
    }

    #[test]
    fn eslenemeyen_kurallar_ham_cssde_korunur() {
        let doc = ThemeDoc::default();
        let css = r#"
:root { --fds-accent-base: 200, 50%, 50%; }

/* bu bizim tanımadığımız bir kural */
.ozel-rozet {
    background: linear-gradient(90deg, #f00, #00f);
}

@media (max-width: 700px) {
    :root { --fds-accent-base: 10, 10%, 10%; }
}
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.accent, [200.0, 50.0, 50.0]);
        // Tanınmayan kural, yorumuyla birlikte korunmalı.
        assert!(parsed.raw_css.contains(".ozel-rozet"));
        assert!(parsed.raw_css.contains("linear-gradient"));
        assert!(parsed.raw_css.contains("tanımadığımız"));
        // Mod'a özel blok tek bir kontrol setine indirilemez; olduğu gibi kalır.
        assert!(parsed.raw_css.contains("@media"));
        assert!(parsed.raw_css.contains("10, 10%, 10%"));
    }

    #[test]
    fn karisik_root_blogu_bolunmez() {
        // İçinde custom property OLMAYAN bir bildirim de varsa blok
        // devralınmaz; bölmek yerine ham CSS'te bırakılır.
        let doc = ThemeDoc::default();
        let css = ":root { --fds-accent-base: 12, 88%, 55%; color-scheme: dark; }";
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.accent, DEFAULT_ACCENT, "blok devralınmamalıydı");
        assert!(parsed.raw_css.contains("color-scheme: dark"));
        assert!(parsed.raw_css.contains("--fds-accent-base"));
    }

    #[test]
    fn turetilmis_accent_basamaklari_haritaya_girmez() {
        let doc = ThemeDoc::default();
        let css = ":root { --fds-accent-base: 280, 70%, 50%; --fds-accent-dark-1: 1, 2%, 3%; }";
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.accent, [280.0, 70.0, 50.0]);
        assert!(
            !parsed.token_overrides.contains_key("--fds-accent-dark-1"),
            "türetilen basamak tabandan yeniden üretilir"
        );
    }

    #[test]
    fn yorum_ve_string_icindeki_parantezler_kural_sinirini_kaydirmaz() {
        let doc = ThemeDoc::default();
        let css = r#"
/* eski kural: .x { color: red } */
.ozel::after { content: "}"; color: lime; }
:root { --fds-accent-base: 100, 40%, 40%; }
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);

        // Yorum ve string doğru atlanmadıysa accent hiç bulunamaz.
        assert_eq!(parsed.accent, [100.0, 40.0, 40.0]);
        assert!(parsed.raw_css.contains(".ozel::after"));
        assert!(parsed.raw_css.contains("color: lime"));
    }

    #[test]
    fn import_satirlari_ayrilir() {
        let doc = ThemeDoc::default();
        let css = "@import url(\"https://fonts.googleapis.com/css2?family=Inter\");\n\
                   :root { --fds-accent-base: 5, 5%, 5%; }";
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.imports, vec!["https://fonts.googleapis.com/css2?family=Inter"]);
        assert!(!parsed.raw_css.contains("@import"));
    }

    /// İçe aktarılan tema, üretilen CSS'e eksiksiz taşınmalı.
    #[test]
    fn ice_aktarilan_tema_uretilen_csste_eksiksiz_kalir() {
        let doc = ThemeDoc::default();
        let css = r#"
:root { --fds-accent-base: 340, 82%, 52%; --fds-text-primary: #fff; }
.anime-card, .slider-card, .grid-view-item { border-radius: 14px; }
.ozel-rozet { background: #f0f; }
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);
        let emitted = parsed.emit_css();

        assert!(emitted.contains("--fds-accent-base: 340, 82%, 52%;"));
        assert!(emitted.contains("--fds-text-primary: #fff;"));
        assert!(emitted.contains("border-radius: 14px;"));
        assert!(emitted.contains(".ozel-rozet"));
    }

    /// Yabancı CSS eşlemesi mevcut kod-editörü turunu ETKİLEMEMELİ.
    #[test]
    fn parse_css_davranisi_degismedi() {
        let doc = ThemeDoc::default();
        let text = ".ozel { color: red; }";
        let parsed = parse_css(text, &doc);
        // İşaretleyici yoksa her şey ham CSS'te kalır — eskiden olduğu gibi.
        assert!(parsed.raw_css.contains(".ozel"));
        assert!(parsed.rule_overrides.is_empty());
        assert_eq!(parsed.accent, doc.accent);
    }
}
