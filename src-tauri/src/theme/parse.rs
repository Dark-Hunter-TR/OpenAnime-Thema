use std::collections::{BTreeMap, BTreeSet};
use crate::theme::models::{
    AccentAlias, ImportedRule, RadiusAlias, ThemeDoc, TokenAlias, TOKENS_OPEN, TOKENS_CLOSE,
};
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

/// `i` konumunda "atlanacak" bir bölüm başlıyorsa (yorum ya da tırnaklı dizge)
/// o bölümün BİTİŞİNİ döndürür; başlamıyorsa `None`.
///
/// CSS'i tararken karşılaşılan her `;`, `{`, `:` gerçek bir sınır değil: yorum
/// metninin ya da bir dizgenin içindekiler sıradan karakterdir. Tek bir yerden
/// geçen bu yardımcı, aşağıdaki bütün tarayıcıların aynı kuralı kullanmasını
/// sağlıyor — biri diğerinden ayrı düşerse aynı dosyayı iki farklı şekilde
/// bölerdik.
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

/// Bir bildirim gövdesini ÜST DÜZEY `;` sınırlarından ayırır.
///
/// `str::split(';')` neden yetmiyor: CSS değerlerinin içinde noktalı virgül
/// geçebiliyor ve gerçek temalarda geçiyor da — en yaygını
/// `url(data:image/png;base64,…)`. Naif bölme o bildirimi tam ortasından kesip
/// `url(` parantezini açık bırakıyor; sonuç yalnızca o değerin kaybı değil,
/// çünkü yeniden üretilen CSS'te açık kalan parantez kendisinden SONRAKİ
/// bildirimleri de yutuyor ve blok bütünüyle bozuluyor. Bu yüzden `;` ancak
/// parantezlerin, tırnakların ve yorumların dışındaysa sınır sayılıyor.
pub(crate) fn split_top_level(body: &str) -> Vec<&str> {
    let bytes = body.as_bytes();
    let len = bytes.len();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut i = 0usize;

    while i < len {
        if let Some(next) = skip_noise(bytes, len, i) {
            i = next;
            continue;
        }
        match bytes[i] {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b';' if depth == 0 => {
                out.push(&body[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }

    if start < len {
        out.push(&body[start..]);
    }
    out
}

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
            // Değerin sonu, `split_top_level` ile AYNI kuralla bulunuyor:
            // parantez/tırnak içindeki `;` bitiş değil.
            let end = split_top_level(&block[val_start..])
                .first()
                .map(|first| val_start + first.len())
                .unwrap_or(block.len());
            return Some(block[val_start..end].trim());
        }
        from = at + name.len();
    }
    None
}

pub fn parse_px(value: &str) -> Option<f64> {
    let mut v = value.trim();
    if v.ends_with("!important") {
        v = v.trim_end_matches("!important").trim();
    }
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

/// `emit_css`in eklediği özgüllük önekini seçiciden çıkarır.
///
/// Yönetilen token bloğunun seçicisi (`:root:root`) etkilenmiyor: önek ÜÇ
/// `:root` ve ardından boşluk, token bloğunda ise iki tane var ve boşluk yok.
pub(crate) fn strip_specificity_boost(selector: &str) -> String {
    selector
        .split(',')
        .map(|part| {
            let part = part.trim();
            match part.strip_prefix(crate::theme::emit::SPECIFICITY_BOOST) {
                // Ardından boşluk gelmeli — `:root:root:root.fds-theme-dark`
                // gibi bir seçici önek değil, gerçek bir kuraldır.
                Some(rest) if rest.starts_with(char::is_whitespace) => rest.trim_start(),
                _ => part,
            }
            .to_string()
        })
        .collect::<Vec<_>>()
        .join(", ")
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
    // Yorumlar önce siliniyor: `split_top_level` onları zaten atlıyor ama
    // bildirimin İÇİNDE kalan bir yorum (ör. `--x: 4px /* not */`) değerin
    // parçası gibi görünürdü. Gerçek temalarda her satırın sonunda açıklama
    // yorumu var (bkz. REzero teması), yani bu istisna değil kural.
    let cleaned = strip_comments(body);
    split_top_level(&cleaned)
        .into_iter()
        .filter_map(|decl| {
            // İlk `:` özellik sınırı — özellik adları iki nokta içeremez,
            // dolayısıyla `url(https://…)` gibi değerler bunu bozmuyor.
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

/// `@import` kuralının parantez/tırnak kabuğunu soyup çıplak adresi verir.
fn extract_import_url(inner: &str) -> Option<String> {
    let s = inner.trim();

    // `url( … )` sarmalı varsa içini al; yoksa metnin kendisi aday.
    let candidate = match s.get(..4) {
        Some(head) if head.eq_ignore_ascii_case("url(") => {
            let close = s.find(')')?;
            s[4..close].trim()
        }
        _ => s,
    };

    // Tırnaklı biçimde adres tırnakların ARASINDA bitiyor; sonrasında bir
    // medya sorgusu (`@import "a.css" screen;`) gelebilir ve o adrese dahil
    // değil. Bu yüzden kapanış tırnağını arıyoruz, `trim_matches` ile iki
    // ucu birden kırpmıyoruz.
    let url = match candidate.chars().next() {
        Some(q @ ('"' | '\'')) => {
            let body = &candidate[q.len_utf8()..];
            let end = body.find(q)?;
            &body[..end]
        }
        _ => candidate,
    };

    let url = url.trim();
    if url.is_empty() {
        None
    } else {
        Some(url.to_string())
    }
}

/// `@import` kurallarını metinden ayırır.
///
/// Satır bazlı bir tarama neden yetmiyor: gerçek temalarda `@import` satırının
/// sonunda açıklama yorumu oluyor (`@import url('…'); /* font */`). Satırı
/// `;`'den kırpmaya çalışan eski yol, yorumu adresin parçası sanıp
/// `https://…swap'); /* font */` gibi bir "adres" üretiyordu; yeniden
/// üretilen `@import url("…")` geçersiz oluyor ve font hiç yüklenmiyordu.
/// Burada kuralın sonu, gerçek CSS sınırı olan üst düzey `;` ile bulunuyor ve
/// satırın kalanı (yorum dahil) metinde olduğu gibi kalıyor.
fn split_imports(text: &str) -> (Vec<String>, String) {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut imports = Vec::new();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut depth = 0usize;
    let mut i = 0usize;

    while i < len {
        if let Some(next) = skip_noise(bytes, len, i) {
            i = next;
            continue;
        }
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                i += 1;
                continue;
            }
            _ => {}
        }

        // Yalnızca üst düzeydeki `@import`ler: bir kuralın içindekiler zaten
        // geçersiz CSS ve oraya dokunmuyoruz.
        let is_import = depth == 0
            && bytes[i] == b'@'
            && text.get(i..i + 7).is_some_and(|h| h.eq_ignore_ascii_case("@import"));
        if !is_import {
            i += 1;
            continue;
        }

        let mut j = i + 7;
        let mut parens = 0usize;
        while j < len {
            if let Some(next) = skip_noise(bytes, len, j) {
                j = next;
                continue;
            }
            match bytes[j] {
                b'(' => parens += 1,
                b')' => parens = parens.saturating_sub(1),
                // `{` bir sonraki kuralın başlangıcı: `;` unutulmuşsa kuralın
                // içine taşmadan duruyoruz.
                b';' | b'{' if parens == 0 => break,
                _ => {}
            }
            j += 1;
        }

        if let Some(url) = extract_import_url(&text[i + 7..j]) {
            imports.push(url);
        }
        // `;` varsa o da silinsin; yoksa (dosya sonu / `{`) olduğu yerde dur.
        let end = if j < len && bytes[j] == b';' { j + 1 } else { j };
        spans.push((i, end));
        i = end;
    }

    let mut rest = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for (start, end) in spans {
        rest.push_str(&text[cursor..start]);
        cursor = end;
    }
    rest.push_str(&text[cursor..]);

    (imports, rest)
}

pub fn parse_css(text: &str, known_selectors: &[String], current: &ThemeDoc) -> ThemeDoc {
    let (imports, without_imports) = split_imports(text);
    let text = without_imports.as_str();

    let open = text.find(TOKENS_OPEN);
    let close = text.find(TOKENS_CLOSE);

    // Bloğun ÜSTÜ ile ALTI ayrı alanlara gidiyor ve bu ayrım keyfi değil:
    // yayılma sırasını belirleyen şey zaten konum. Üstteki metin tabandır
    // (içe aktarılan tema), alttaki metin bloğu ezer (kullanıcının ham CSS'i).
    // İkisini tek bir `raw_css`te birleştirmek — önceki davranış — bu bilgiyi
    // yok ediyordu: yeniden üretimde ikisi de bloğun ALTINA düşüyor ve içe
    // aktarılan tema kontrolleri sürekli eziyordu.
    let (managed, imported, raw) = match (open, close) {
        (Some(o), Some(c)) if c > o => (
            Some(&text[o + TOKENS_OPEN.len()..c]),
            // Bloğun üstü metin olarak DEĞİL, kural kural okunuyor: kullanıcı
            // kod editöründe ithal bir kuralı elle düzenlediğinde değişiklik
            // modele girsin diye (gerekçe: `ThemeDoc::imported_rules`).
            decompose_imported(&text[..o]),
            text[c + TOKENS_CLOSE.len()..].trim().to_string(),
        ),
        _ => (None, Vec::new(), text.trim().to_string()),
    };

    let Some(managed) = managed else {
        return parse_foreign_css(text, known_selectors, current);
    };

    let cleaned = strip_comments(managed);
    let mut rule_overrides = BTreeMap::new();
    let mut root_body = String::new();

    for (selector, body) in split_rules(&cleaned) {
        // Özgüllük öneki emit sırasında ekleniyor, burada geri sökülüyor.
        // Sökülmeseydi iki şey birden bozulurdu: anahtar artık kontrolün
        // tanıdığı seçici olmazdı ve her turda bir önek daha eklenirdi.
        let selector = strip_specificity_boost(&selector);
        let normalized = selector.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.starts_with(":root") {
            root_body.push_str(&body);
            root_body.push(';');
        } else if normalized.starts_with(".list-item.selected") {
            
        } else {
            rule_overrides.insert(normalized, body);
        }
    }

    let accent = find_decl(&root_body, "--fds-accent-base")
        .and_then(parse_triplet)
        .unwrap_or(DEFAULT_ACCENT);

    let derived: Vec<String> = DERIVED_TOKEN_NAMES.iter().map(|n| format!("--fds-{n}")).collect();
    let mut token_overrides = BTreeMap::new();
    let mut root_decls = BTreeMap::new();
    for (prop, value) in parse_decls(&root_body) {
        root_decls.insert(prop.clone(), value.clone());
        let structural = prop == "--fds-control-corner-radius"
            || prop == "--fds-overlay-corner-radius"
            || derived.iter().any(|d| d == &prop);
        if !structural {
            token_overrides.insert(prop, value);
        }
    }
    // Yönetilen blok bir sonraki üretimde tabandan yeniden hesaplanıyor; elle
    // yazılmış ve türetilenden AYRILAN bir vurgu basamağı varsa korunmalı
    // (gerekçe: `explicit_derived_overrides`).
    token_overrides.extend(explicit_derived_overrides(&root_decls, &root_decls, accent));

    ThemeDoc {
        accent,
        mode: current.mode,
        control_corner_radius: find_decl(&root_body, "--fds-control-corner-radius")
            .and_then(parse_px),
        overlay_corner_radius: find_decl(&root_body, "--fds-overlay-corner-radius")
            .and_then(parse_px),
        imports,
        token_overrides,
        rule_overrides,
        // Yönetilen turda yeniden hesaplanmıyor, taşınıyor: bu alan CSS'e
        // yazılmadığı için metinden geri okunamaz. Taşınmasaydı kullanıcı kod
        // editöründe tek bir harf değiştirdiğinde kontroller boşalırdı.
        seed_tokens: current.seed_tokens.clone(),
        // `seed_tokens` ile aynı gerekçe: bu alan CSS'ten geri okunamaz,
        // taşınmazsa kod editöründe tek harf değişince aile kaybolurdu.
        accent_aliases: current.accent_aliases.clone(),
        radius_aliases: current.radius_aliases.clone(),
        token_aliases: current.token_aliases.clone(),
        // `seed_tokens` ile aynı gerekçe: CSS'ten geri okunamaz, taşınmazsa
        // kod editöründe tek harf değişince "kullanıcı vurguyu değiştirdi"
        // sanılırdı.
        imported_accent: current.imported_accent,
        imported_rules: imported,
        // Aynı gerekçe: içe aktarma anındaki değerler CSS'e yazılmıyor.
        imported_tokens: current.imported_tokens.clone(),
        imported_css: String::new(),
        raw_css: raw,
    }
}

struct SpannedRule {
    selector: String,
    body: String,
    start: usize,
    end: usize,
    /// Kuralı saran koşul at-kurallarının (`@media`, `@supports`,
    /// `@container` …) zinciri — en dıştan içe. Boşsa kural koşulsuz.
    ///
    /// Koşullu kurallar ASLA tüketilmiyor ve değişkenleri `:root`'a
    /// taşınmıyor: `@media (max-width: 768px) { :root { --logo: 18px } }`
    /// yalnızca dar ekranda geçerli, oysa taşınsa her ekranda geçerli olurdu.
    /// Gerçek temalar mobil kırılımlarını tam olarak böyle yazıyor.
    ///
    /// Bayrak yerine zincir tutuluyor çünkü kural yeniden ÜRETİLİYOR: yalnızca
    /// "koşulluydu" bilgisi, onu tekrar doğru sarmalayıcının içine yazmaya
    /// yetmez.
    at: Vec<String>,
}

impl SpannedRule {
    fn conditional(&self) -> bool {
        !self.at.is_empty()
    }
}

/// Koşul at-kuralları: gövdeleri bildirim değil, İÇ İÇE KURAL taşır.
const CONDITIONAL_AT_RULES: [&str; 5] = [
    "@media",
    "@supports",
    "@container",
    "@layer",
    "@scope",
];

fn is_conditional_at_rule(selector: &str) -> bool {
    let lower = selector.trim().to_ascii_lowercase();
    CONDITIONAL_AT_RULES
        .iter()
        .any(|at| lower.starts_with(at) && lower[at.len()..].starts_with([' ', '(', '\t', '\n']))
}

/// Metnin başındaki boşluk + yorum dizisinin bittiği bayt konumu.
///
/// Yalnızca BAŞTAKİLER: ilk yorum olmayan karakterde duruyor, yani seçicinin
/// ortasındaki bir yoruma (`.a /* x */ .b`) dokunmuyor — orada yorum gerçekten
/// seçicinin parçası.
fn leading_comment_end(text: &str) -> usize {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0usize;

    loop {
        while i < len && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < len && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            match text[i + 2..].find("*/") {
                Some(end) => i = i + 2 + end + 2,
                None => return len,
            }
        } else {
            return i;
        }
    }
}

fn split_rules_spanned(text: &str) -> Vec<SpannedRule> {
    split_rules_spanned_inner(text, 0, &[])
}

fn split_rules_spanned_inner(text: &str, offset: usize, at: &[String]) -> Vec<SpannedRule> {
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut selector_start = 0usize;

    while i < len {
        if let Some(next) = skip_noise(bytes, len, i) {
            i = next;
            continue;
        }

        if bytes[i] == b'{' {
            let selector_raw = &text[selector_start..i];
            // Seçicinin ÖNÜNDEKİ yorumlar seçicinin parçası değil.
            //
            // Ayrılmadıklarında seçici `/* [Açıklama] Kartlar. */ .anime-card`
            // gibi bir şey oluyordu. CSS bunu kabul ediyor ama iki şeyi
            // bozuyordu:
            //
            //   * Tanınan seçici eşleşmesi (`known.get(&normalized)`) hiç
            //     tutmuyordu — açıklamalı yazılmış bir temada kontrole
            //     bağlanabilecek kural bulunamıyordu. Ölçüldü: örnek temanın
            //     103 kuralından yalnızca 1'i eşleşiyordu.
            //   * Kuralın başlangıç konumu yorumu da kapsadığı için açıklama
            //     ayrı bir alana (`ImportedRule::note`) alınamıyordu.
            let lead = leading_comment_end(selector_raw);

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

            let selector = selector_raw[lead..].trim().to_string();
            if !selector.is_empty() {
                if is_conditional_at_rule(&selector) {
                    // At-kuralının KENDİSİ bir kural değil; gövdesindeki iç
                    // kurallar var. Onları zincire ekleyerek açıyoruz ki hem
                    // tüketilmesinler, hem değişkenleri küresele sızmasın, hem
                    // de yeniden üretilirken aynı sarmalayıcıya geri girsinler.
                    let mut nested = at.to_vec();
                    nested.push(selector);
                    out.extend(split_rules_spanned_inner(
                        &text[i + 1..body_end],
                        offset + i + 1,
                        &nested,
                    ));
                } else {
                    out.push(SpannedRule {
                        selector,
                        body: text[i + 1..body_end].trim().to_string(),
                        start: offset + selector_start + lead,
                        end: offset + j,
                        at: at.to_vec(),
                    });
                }
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

/// Algılanan vurgu ve KAYNAĞININ ne olduğu.
///
/// `true` ise renk doğrudan `--fds-accent-base`ten geldi, yani zaten rampanın
/// tabanı. `false` ise temanın kendi boyama rengi — o da rampanın tabanına
/// değil `--fds-accent-default`a karşılık gelir (gerekçe:
/// `color::base_for_step`).
fn extract_accent_from_vars(all_vars: &BTreeMap<String, String>) -> Option<(Hsl, bool)> {
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
                return Some((hsl, name == "--fds-accent-base"));
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
                return Some((hsl, name == "--fds-accent-base"));
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

/// Temanın, site token'larını GÖLGELEYEN kendi değişkenlerini bulur.
///
/// `detect_accent_aliases` ve `detect_radius_aliases` iki ekseni çözüyordu; bu
/// geri kalan her kontrolü çözüyor (gerekçe: `TokenAlias`).
///
/// `taken`, o iki ailenin zaten sahiplendiği adlar — onların kendi sapma
/// hesapları var, buradan ikinci kez yazılmamalılar.
fn detect_token_aliases(
    tokens: &BTreeMap<String, String>,
    all_vars: &BTreeMap<String, String>,
    used: &BTreeSet<String>,
    taken: &BTreeSet<String>,
) -> Vec<TokenAlias> {
    let mut out = Vec::new();

    for (name, raw) in all_vars {
        // `--fds-*`'ı zaten kendimiz yazıyoruz.
        if name.starts_with("--fds-") || taken.contains(name) {
            continue;
        }
        // Tema bu adı hiç kullanmıyorsa yeniden yazmanın etkisi olmaz.
        if !used.contains(name) {
            continue;
        }

        let value = resolve_var(raw, all_vars);
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Değeri BİREBİR aynı olan site token'ı. Birden fazlaysa ilki
        // (`BTreeMap` sırası) alınıyor — hangisi seçilirse seçilsin değer aynı
        // olduğu için görünen sonuç da aynı.
        let Some((source, _)) = tokens
            .iter()
            .find(|(token, token_value)| {
                token.starts_with("--fds-") && token_value.trim() == trimmed
            })
        else {
            continue;
        };

        out.push(TokenAlias {
            name: name.clone(),
            source: source.clone(),
        });
    }

    out
}

/// Temanın kendi YUVARLAKLIK değişkenlerini bulur.
///
/// Tespit ad'a değil KULLANIMA bakıyor: bir değişken, `border-radius` ailesinden
/// bir bildirimin değerinde geçiyorsa yuvarlaklık değişkenidir. Ada bakmak
/// çalışmazdı — gerçek temalarda adlar `--radius-card`, `--ayar-kose-
/// yuvarlakligi-genel`, `--rounding` gibi birbirinden bağımsız ve dile bağlı.
///
/// Toplanan her ad için içe aktarma anındaki yuvarlaklıktan SAPMA saklanıyor
/// (gerekçe: `RadiusAlias`).
fn detect_radius_aliases(
    rules: &[SpannedRule],
    all_vars: &BTreeMap<String, String>,
    base: f64,
) -> Vec<RadiusAlias> {
    let mut names: BTreeMap<String, ()> = BTreeMap::new();

    for rule in rules {
        for (prop, value) in parse_decls(&rule.body) {
            let prop = prop.trim().to_ascii_lowercase();
            // `border-radius`, `border-top-left-radius`,
            // `--fds-control-corner-radius`, `--radius-card` …
            if !prop.ends_with("radius") {
                continue;
            }
            for name in var_names(&value) {
                names.insert(name, ());
            }
        }
    }

    let mut out = Vec::new();
    for (name, _) in names {
        // `--fds-*`'ı zaten kendimiz yazıyoruz; ikinci kez yazmak çakışma olurdu.
        if name.starts_with("--fds-") {
            continue;
        }
        let Some(raw) = all_vars.get(&name) else {
            continue;
        };
        // Yalnızca TEK bir uzunluk çözülebiliyorsa alınıyor. `0 0 12px 12px`
        // gibi dört köşeli bir değer ya da `50%` (tam daire) kaydırıcıya
        // bağlanamaz; bağlansaydı avatarın yuvarlaklığı bozulurdu.
        let Some(px) = parse_px(&resolve_var(raw, all_vars)) else {
            continue;
        };
        out.push(RadiusAlias {
            name,
            delta: px - base,
        });
    }
    out
}

/// Bir değerin içindeki `var(--x)` adları.
fn var_names(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = value.as_bytes();
    let mut from = 0usize;

    while let Some(rel) = value[from..].find("var(") {
        let start = from + rel + 4;
        let mut i = start;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let name_start = i;
        while i < bytes.len()
            && matches!(bytes[i], b'-' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')
        {
            i += 1;
        }
        if i > name_start && value[name_start..].starts_with("--") {
            out.push(value[name_start..i].to_string());
        }
        from = start;
    }
    out
}

/// Temanın kendi vurgu AİLESİNİ bulur: aynı tonun etrafındaki değişkenler.
///
/// Neden gerekli: gelişmiş temalar `--fds-accent-*`'ı hiç kullanmıyor. Kendi
/// adlarını tanımlayıp (`--accent-primary`, `--accent-secondary`,
/// `--accent-tertiary`) her yerde `var(--accent-primary) !important` ile
/// boyuyorlar. Kaydırıcı yalnızca `--fds-accent-*`'ı yazdığında temanın
/// boyadığı hiçbir yer değişmiyor — kullanıcının gördüğü "renk değişimi tam
/// değil" tam olarak bu.
///
/// Aile, ADA değil DEĞERE bakılarak bulunuyor: tonu vurguya yakın ve renkli
/// (gri olmayan) her değişken vurgunun bir tonu sayılıyor. Ada bakmak
/// (`*accent*` içerenler) hem kaçırırdı (`--brand`, `--mor`) hem de yanlış
/// yakalardı (`--accent-text` bir metin rengi olabilir).
///
/// Saklanan şey mutlak renk değil, vurgudan SAPMA. Kaydırıcı oynadığında her
/// değişken kendi sapmasıyla yeniden hesaplanıyor; böylece açık ton açık, koyu
/// ton koyu kalıyor. Mutlak renk saklansaydı aile ilk oynatmada tek renge
/// çökerdi.
fn detect_accent_aliases(
    vars: &BTreeMap<String, String>,
    all_vars: &BTreeMap<String, String>,
    accent: Hsl,
) -> Vec<AccentAlias> {
    /// Vurgu ailesinden sayılmak için tona en fazla bu kadar uzaklık (derece).
    const HUE_TOLERANCE: f64 = 18.0;
    /// Bunun altındaki kroma gri/beyaz sayılıyor.
    ///
    /// Eskiden ölçüt ham doygunluktu (%12) ve bu, gerçek bir temada üç ayrı
    /// hataya yol açıyordu: `--mn-white: #e8eaf2` (beyaz metin, kroma 0.04),
    /// `--mn-muted: #4a4f68` (gri, 0.12) ve `--mn-sub1: #9da3bb` (0.12)
    /// "vurgu ailesi" sayılıyor, kullanıcı vurgu rengini oynattığında temanın
    /// metin renkleri de kayıyordu. Gerçek vurgu tonları bu eşiğin belirgin
    /// biçimde üstünde: `#8db4f7` 0.42, `#6b9ef5` 0.54, `#b8d4ff` 0.28.
    const MIN_CHROMA: f64 = 0.20;

    let mut out = Vec::new();

    for (name, raw) in vars {
        // `--fds-accent-*` zaten rampadan yazılıyor; ikinci kez yazmak
        // çakışma olurdu.
        if name.starts_with("--fds-") {
            continue;
        }
        let resolved = resolve_var(raw, all_vars);
        let Some(hsl) = parse_color_to_hsl(&resolved) else {
            continue;
        };
        if crate::theme::color::chroma(hsl) < MIN_CHROMA {
            continue;
        }

        // Ton farkı dairesel: 350° ile 10° arası 20°, 340° değil.
        let mut dh = hsl[0] - accent[0];
        if dh > 180.0 {
            dh -= 360.0;
        } else if dh < -180.0 {
            dh += 360.0;
        }
        if dh.abs() > HUE_TOLERANCE {
            continue;
        }

        out.push(AccentAlias {
            name: name.clone(),
            delta: [dh, hsl[1] - accent[1], hsl[2] - accent[2]],
            // Alfa sapmaya girmiyor: kaydırıcı rengi değiştirir, saydamlığı
            // değiştirmez (gerekçe: `AccentAlias::alpha`).
            alpha: crate::theme::color::parse_alpha(&resolved).unwrap_or(1.0),
        });
    }

    out
}

/// Dosyanın ELLE yazdığı, türetilenden farklı vurgu token'larını toplar.
///
/// Vurgu rampasının yedi basamağı normalde tabandan hesaplanıyor, bu yüzden
/// ayrıştırıcı onları atıyordu. Ama temalar rampanın tek tek basamaklarını
/// bilerek eziyor: ytanime teması tabanı sitenin varsayılanında bırakıp
/// `--fds-accent-light-3` ve `--fds-accent-light-1`'i beyaza çekiyor. Atıldığı
/// için o iki değer yeniden üretimde maviye dönüyordu — dosyada açıkça yazan
/// bir renk, kullanıcı hiçbir şeye dokunmadan değişiyordu.
///
/// Bu yüzden karar değere bakılarak veriliyor: değer, o taban için türetilecek
/// olanla aynıysa gereksiz tekrar sayılıp atlanıyor; FARKLIYSA kullanıcının
/// kastı kabul edilip override olarak korunuyor.
fn explicit_derived_overrides(
    vars: &BTreeMap<String, String>,
    all_vars: &BTreeMap<String, String>,
    accent: Hsl,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for name in DERIVED_TOKEN_NAMES {
        let key = format!("--fds-{name}");
        let Some(raw) = vars.get(&key) else { continue };
        let Some(derived) = crate::theme::emit::derived_token_value(name, accent) else {
            continue;
        };

        // Karşılaştırma İKİ biçimde de yapılıyor. Ham hâlleri farklı olabilir
        // ama aynı şeyi anlatabilirler: yönetilen blok bu token'ları
        // `hsl(var(--fds-accent-base))` diye yazıyor, dosyadaki karşılığı ise
        // çözülmüş `hsl(120, 60%, 40%)` olabilir. Yalnızca ham metne bakılsaydı
        // uygulamanın KENDİ ürettiği blok bile "elle yazılmış" sayılır ve her
        // tur bir kopya daha eklerdi.
        let same_raw = raw.trim() == derived;
        let same_resolved =
            resolve_var(raw, all_vars) == resolve_var(&derived, all_vars);
        if !same_raw && !same_resolved {
            out.insert(key, resolve_var(raw, all_vars));
        }
    }
    out
}

/// Yapısal token'lar: `ThemeDoc`'ta kendi alanları var ve `emit_css` onları
/// ayrıca yazıyor. `token_overrides`'a da girerlerse aynı bildirim blokta iki
/// kez çıkar; ikisi ayrışırsa (kullanıcı yuvarlaklığı kontrolden değiştirir,
/// eski kopya blokta kalır) hangisinin kazandığı sıraya bağlı hâle gelir.
pub(crate) const STRUCTURAL_TOKENS: [&str; 2] = [
    "--fds-control-corner-radius",
    "--fds-overlay-corner-radius",
];

/// `root_vars`: `token_overrides`'a girmeye aday olanlar (koşulsuz, küresel).
/// `all_vars`: `var(--a)` başvurularını çözmek için gereken TAM harita.
fn extract_tokens_from_vars(
    root_vars: &BTreeMap<String, String>,
    all_vars: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut overrides = BTreeMap::new();
    let derived: Vec<String> = DERIVED_TOKEN_NAMES.iter().map(|n| format!("--fds-{n}")).collect();

    for (name, val) in root_vars.iter() {
        if name.starts_with("--")
            && !derived.contains(name)
            && !STRUCTURAL_TOKENS.contains(&name.as_str())
        {
            let resolved = resolve_var(val, all_vars);
            overrides.insert(name.clone(), resolved);
        }
    }

    let bg_names = [
        "--bg-page", "--mn-bg0", "--bg-layer", "--bg-card",
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

/// Seçiciyi karşılaştırılabilir tek biçime indirger.
///
/// Özgüllük öneki burada sökülüyor çünkü o bizim kendi ürettiğimiz bir ek
/// (bkz. `emit::SPECIFICITY_BOOST`). Sökülmediğinde, bu editörden çıkmış bir
/// dosya yeniden açıldığında yönetilen bloktaki kurallar tanınan seçicilerle
/// EŞLEŞMİYOR ve kontrollere geri bağlanamıyorlardı.
pub(crate) fn normalize_selector(selector: &str) -> String {
    strip_specificity_boost(selector)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Blok, değişkenleri KOŞULSUZ olarak belgenin tamamına yayıyor mu.
///
/// Yalnızca bu bloklar tüketilebilir (metinden çıkarılıp `token_overrides`'a
/// taşınabilir), çünkü yeniden üretilen `:root, .fds-theme-light,
/// .fds-theme-dark` bloğu onlarla aynı kapsamda.
fn is_global_token_block(normalized: &str) -> bool {
    normalized.split(',').any(|part| {
        let p = part.trim();
        p.contains(":root") || p == "html" || p == "body" || p == ":host" || p == "*"
    })
}

/// Blok, yalnızca BİR TEMA KİPİNDE geçerli olan değişkenler taşıyor mu
/// (`.fds-theme-dark`, `[data-theme="light"]`, `.theme-x` …).
///
/// Bunlar tüketilmiyor ve `token_overrides`'a alınmıyor. Sebebi somut: ytanime
/// teması bütün koyu kip renklerini `.fds-theme-dark` altında tanımlıyor;
/// taşınsalardı yeniden üretilen blok onları `.fds-theme-light`e de yazar ve
/// açık kip koyu renklerle açılırdı. Metinde bıraktığımızda tarayıcı özgün
/// kapsamıyla uyguluyor — yani hiçbir şey kaybolmadan doğru davranıyor.
fn is_theme_scoped_block(normalized: &str) -> bool {
    normalized.split(',').any(|part| {
        let p = part.trim();
        p.contains("data-theme") || p.contains("theme-") || p.contains("fds-theme")
    })
}

/// `start` konumundaki kuralın hemen üstündeki açıklama yorumu.
///
/// Elle yazılmış temalarda kuralın ne yaptığı bu yorumlarda anlatılıyor —
/// örnek temada 100'den fazla var. Kural modele girip yorumu girmeseydi
/// kullanıcı temasını `.css` olarak geri kaydettiğinde dosyası sessizce
/// açıklamalarından arınmış olurdu.
///
/// Arka arkaya yazılmış yorum blokları (bölüm başlıkları böyle) tek not
/// sayılıyor. Araya yorum olmayan bir şey girerse tarama duruyor: bir önceki
/// kuralın gövdesine ya da bildirimine uzanmıyoruz.
fn leading_note(text: &str, start: usize) -> Option<String> {
    let bytes = text.as_bytes();
    let mut begin = start.min(text.len());

    loop {
        let mut probe = begin;
        while probe > 0 && bytes[probe - 1].is_ascii_whitespace() {
            probe -= 1;
        }
        if probe < 2 || bytes[probe - 2] != b'*' || bytes[probe - 1] != b'/' {
            break;
        }
        let Some(open) = rfind_comment_open(bytes, probe - 2) else {
            break;
        };
        begin = open;
    }

    if begin >= start {
        return None;
    }
    let note = text.get(begin..start)?.trim();
    if note.is_empty() {
        None
    } else {
        Some(note.to_string())
    }
}

/// `from`'dan geriye doğru en yakın `/*`. CSS'te yorumlar iç içe geçmiyor,
/// dolayısıyla en yakını doğru eşleşme.
fn rfind_comment_open(bytes: &[u8], from: usize) -> Option<usize> {
    let mut i = from;
    while i >= 2 {
        if bytes[i - 2] == b'/' && bytes[i - 1] == b'*' {
            return Some(i - 2);
        }
        i -= 1;
    }
    None
}

/// Taranmış kuralları modele çevirir.
///
/// `consumed`, kontrollere taşınmış (yani başka bir alanda temsil edilen)
/// kuralların aralıkları — onlar iki kez yazılmasın diye atlanıyor.
fn decompose_rules(
    text: &str,
    rules: &[SpannedRule],
    consumed: &[(usize, usize)],
) -> Vec<ImportedRule> {
    rules
        .iter()
        .filter(|rule| {
            !consumed
                .iter()
                .any(|&(start, end)| rule.start >= start && rule.end <= end)
        })
        .filter(|rule| !rule.body.trim().is_empty())
        .map(|rule| ImportedRule {
            at: rule.at.clone(),
            selector: rule.selector.clone(),
            body: rule.body.clone(),
            note: leading_note(text, rule.start),
        })
        .collect()
}

/// İşaretleyici yorumlarını siler, aralarındaki CSS'i olduğu yerde bırakır.
///
/// İçe aktarma yolunda kullanılıyor (`parse_foreign_css`): dosyanın tamamı
/// yeniden çözümlenecekse blok bir sınır olmaktan çıkmalı. İÇERİK silinmiyor —
/// blok içi `--fds-*` token'ları ve tanınan seçiciler yabancı-CSS yolundan
/// kontrollere bağlanıyor.
///
/// Açılış işaretleyicisinin hemen ardındaki açıklama yorumu da atılıyor: onu
/// bırakmak, bir sonraki kurala ait "not" gibi görünmesine ve her turda
/// çıktıya yeniden yazılmasına yol açardı.
fn strip_marker_block(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(at) = rest.find(TOKENS_OPEN) {
        out.push_str(&rest[..at]);
        rest = &rest[at + TOKENS_OPEN.len()..];

        // Ardından gelen açıklama yorumunu da at.
        let after = rest.trim_start();
        if after.starts_with("/*") {
            if let Some(end) = after.find("*/") {
                rest = &after[end + 2..];
            }
        }

        // Bloğun İÇİ ayrı ele alınıyor: oradaki türetilmiş vurgu basamakları
        // atılıyor (gerekçe: `drop_derived_ramp`).
        let (inside, tail) = match rest.find(TOKENS_CLOSE) {
            Some(close) => (&rest[..close], &rest[close + TOKENS_CLOSE.len()..]),
            None => (rest, ""),
        };
        out.push_str(&drop_generated_accent_rule(&drop_derived_ramp(inside)));
        rest = tail;
    }
    out.push_str(rest);

    out.replace(TOKENS_CLOSE, "")
}

/// Yönetilen blok içindeki ÜRETİLMİŞ seçili-öğe vurgu kuralını atar.
///
/// `emit_css`, kullanıcı vurguyu değiştirdiğinde şu kuralı yazıyor:
///
/// ```text
/// .list-item.selected, .list-item.selected * { color: var(--fds-accent-default) … }
/// ```
///
/// Bu bizim çıktımız, temanın içeriği değil. Dosya yeniden içe aktarıldığında
/// işaretleyiciler sıyrıldığı için o kural "temanın bir kuralı" sayılıp
/// taşınıyordu: kullanıcı vurguya hiç dokunmasa bile kural dosyada kalmaya
/// devam ediyor ve temanın kendi seçili-ikon rengini eziyordu. Koyu kipte
/// `--fds-accent-default` rampanın `light-2` basamağından türediği için, o
/// basamak beyaza kırpılmış eski bir dosyada ikon düpedüz beyaz kalıyordu.
///
/// Eşleşme dar tutuldu: hem seçicide `.list-item.selected` hem gövdede
/// `var(--fds-accent-default)` aranıyor. Temanın kendi yazdığı bir
/// `.list-item.selected` kuralı bu ikisini birden taşımaz.
fn drop_generated_accent_rule(block: &str) -> String {
    let bytes = block.as_bytes();
    let mut out = String::with_capacity(block.len());
    let mut i = 0usize;
    let mut start = 0usize;

    while i < bytes.len() {
        if bytes[i] != b'{' {
            i += 1;
            continue;
        }

        let selector = &block[start..i];
        let mut depth = 1usize;
        let mut j = i + 1;
        while j < bytes.len() && depth > 0 {
            match bytes[j] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            j += 1;
        }
        let body = &block[i + 1..j.saturating_sub(1)];

        let generated = selector.contains(".list-item.selected")
            && body.contains("var(--fds-accent-default)");
        if !generated {
            out.push_str(&block[start..j]);
        }

        i = j;
        start = j;
    }
    out.push_str(&block[start..]);
    out
}

/// Yönetilen blok içindeki TÜRETİLMİŞ vurgu basamaklarını atar.
///
/// `--fds-accent-base` kalıyor; geri kalan altı basamak tamamen ondan
/// türetiliyor, yani taşınmalarının tek etkisi ESKİ çıktıyı sabitlemek.
///
/// Somut hata: bu editörün eski bir sürümüyle üretilmiş bir dosya, rampayı
/// sabit ışıklılık ekleriyle hesaplanmış hâliyle taşıyordu ve açık bir vurguda
/// üst basamaklar `%100` (beyaz) olarak yazılmıştı. Dosya yeniden içe
/// aktarıldığında o değerler "tema yazarının elle seçtiği" sayılıp
/// korunuyordu (`explicit_derived_overrides`) ve düzeltilmiş türetmenin
/// ÜSTÜNE biniyordu — seçili kenar çubuğu ikonu beyaz kalmaya devam ediyordu.
///
/// Yabancı bir temanın (işaretleyicisi olmayan dosyanın) elle yazdığı
/// basamaklar bu yoldan geçmiyor; onlar korunmaya devam ediyor.
fn drop_derived_ramp(block: &str) -> String {
    let mut out = block.to_string();

    for name in crate::theme::color::RAMP_NAMES {
        if name == "accent-base" {
            continue;
        }
        let needle = format!("--fds-{name}");
        loop {
            let Some(at) = out.find(&needle) else { break };

            // Özellik sınırı: `--fds-accent-light-1` ararken
            // `--fds-accent-light-10` gibi bir adın ortasına düşmeyelim.
            let after = &out[at + needle.len()..];
            let trimmed = after.trim_start();
            if !trimmed.starts_with(':') {
                break;
            }

            let end = after
                .find(';')
                .map(|i| at + needle.len() + i + 1)
                .unwrap_or(out.len());
            out.replace_range(at..end, "");
        }
    }

    out
}

/// Bir CSS metnini olduğu gibi ithal kurallara çevirir.
///
/// İki yerden çağrılıyor: eski projelerin tek parça `imported_css`ini taşımak
/// (`ThemeDoc::migrate_imported`) ve kod editöründe işaretleyici bloğun
/// ÜSTÜNDE kalan metni geri okumak (`parse_css`). İkisinde de tüketilecek bir
/// şey yok — kontrollere eşleme yalnızca ilk içe aktarmada, `parse_foreign_css`
/// içinde yapılıyor.
pub fn decompose_imported(text: &str) -> Vec<ImportedRule> {
    let (_, body) = split_imports(text);
    let rules = split_rules_spanned(&body);
    decompose_rules(&body, &rules, &[])
}

pub fn parse_foreign_css(
    text: &str,
    known_selectors: &[String],
    current: &ThemeDoc,
) -> ThemeDoc {
    // Dosya bu editörden çıkmışsa kendi işaretleyici bloğumuzu taşıyor.
    // İşaretleyiciler SİLİNİP dosyanın tamamı yeniden çözümleniyor.
    //
    // İçe aktarmanın anlamı "bu dosyanın tamamı bir temadır". Blok korunursa
    // dosyanın yalnızca blok İÇİ kısmı kontrollere bağlanıyor, geri kalanı
    // konumuna göre ikiye bölünüyordu: üstü taban, ALTI ise `raw_css` — yani
    // kontrolleri ezen, tek parça, düzenlenemeyen bir yığın.
    //
    // Ölçülmüş bir fark: aynı editörden çıkmış iki dosyadan biri (113 satır,
    // tamamı blok içi) sorunsuz açılıyor, diğeri (1492 satır, 141 seçicilik
    // gövdesi bloğun ALTINDA) bozuk geliyordu. İkinci dosya, ithal temayı
    // `raw_css`e yazan eski bir sürümle üretilmişti; blok korunduğunda o eski
    // yerleşim sadakatle sürdürülüyordu.
    //
    // Blok içeriğinin kaybı yok: içi zaten `--fds-*` token'ları ve tanınan
    // seçicilerden ibaret, ikisini de aşağıdaki yabancı-CSS yolu kontrollere
    // bağlıyor. Yan faydası, çıktıda ikinci bir blok oluşmaması.
    let stripped;
    let text = if text.contains(TOKENS_OPEN) && text.contains(TOKENS_CLOSE) {
        stripped = strip_marker_block(text);
        stripped.as_str()
    } else {
        text
    };

    let (imports, body) = split_imports(text);

    let known: BTreeMap<String, String> = known_selectors
        .iter()
        .map(|s| (normalize_selector(s), s.clone()))
        .collect();

    let rules = split_rules_spanned(&body);

    // İki ayrı değişken haritası tutuluyor ve ayrım kasıtlı:
    //
    //   all_vars  — dosyadaki HER `--x` tanımı. Yalnızca ÇÖZÜMLEME için:
    //               `var(--a)` başvurusunu izlerken ya da vurgu rengini
    //               ararken, değişkenin nerede tanımlandığı önemli değil.
    //
    //   root_vars — yalnızca koşulsuz, küresel bloklarda (`:root`, `html`…)
    //               tanımlananlar. `token_overrides`'a YALNIZCA bunlar giriyor,
    //               çünkü yeniden üretilen blok küresel: bir `@media` ya da
    //               `.fds-theme-dark` değişkenini oraya taşımak, o değişkeni
    //               ait olmadığı bağlamlarda da geçerli kılardı.
    let mut all_vars = BTreeMap::new();
    let mut root_vars = BTreeMap::new();

    // Koşulsuz tanımlar önce, belge sırasıyla (sonraki öncekini ezer).
    for rule in rules.iter().filter(|r| !r.conditional()) {
        let global = !rule.selector.trim_start().starts_with('@')
            && is_global_token_block(&normalize_selector(&rule.selector));
        for (prop, val) in parse_decls(&rule.body) {
            if prop.starts_with("--") {
                if global {
                    root_vars.insert(prop.clone(), val.clone());
                }
                all_vars.insert(prop, val);
            }
        }
    }

    // Koşullu tanımlar yalnızca BOŞLUK DOLDURUYOR. Bir `@media` içindeki
    // `--accent`, koşulsuz tanımın yerine geçmemeli: temanın "asıl" değeri
    // koşulsuz olandır, mobil kırılımdaki değil. Yalnızca değişken başka
    // hiçbir yerde tanımlı değilse buradan okunuyor ki `var()` zincirleri
    // yine de çözülebilsin.
    for rule in rules.iter().filter(|r| r.conditional()) {
        for (prop, val) in parse_decls(&rule.body) {
            if prop.starts_with("--") {
                all_vars.entry(prop).or_insert(val);
            }
        }
    }

    // Kural gövdesinden yakalanan renk de bir BOYAMA rengidir (bir düğmenin,
    // bağlantının rengi), rampanın tabanı değil — o yüzden `false`.
    let detected = extract_accent_from_vars(&all_vars)
        .or_else(|| extract_accent_from_rules(&rules, &all_vars).map(|hsl| (hsl, false)));

    // Taban geriye çözülüyor.
    //
    // Sitenin vurguyla boyadığı her şey `--fds-accent-default` kullanıyor —
    // rampanın tabanını değil. Ölçüldü:
    // `.list-item::before { background-color: var(--fds-accent-default) }`
    // (seçili menü göstergesi). Kütüphane o token'ı kipe göre `light-2` ya da
    // `dark-1` basamağından türetiyor.
    //
    // Temanın kendi vurgu rengi doğrudan taban yapıldığında sitenin çizdiği
    // öğeler bir basamak kayıyordu: koyu kipte `light-2` tabandan açık olduğu
    // için gösterge çubuğu ve seçili ikon, temanın kendi renginden görünür
    // biçimde AÇIK çıkıyordu. Renk "tutmuyordu".
    //
    // `--fds-accent-base` doğrudan yazılmışsa dokunulmuyor: o zaten taban.
    let accent = detected.map(|(hsl, is_base)| {
        if is_base {
            hsl
        } else {
            crate::theme::color::base_for_step(
                hsl,
                crate::theme::color::accent_default_step(matches!(
                    current.mode,
                    crate::theme::models::ThemeMode::Light
                )),
            )
        }
    });

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

    let accent_value = accent.unwrap_or(current.accent);
    let mut token_overrides = extract_tokens_from_vars(&root_vars, &all_vars);
    token_overrides.extend(explicit_derived_overrides(&root_vars, &all_vars, accent_value));

    // İçe aktarma ANINDAKİ değerler. Sahiplik kararı buna bakıyor: kullanıcı
    // bir kontrolü oynatana kadar `token_overrides` bununla aynı kalır ve
    // temanın kipe/ekrana bağlı kendi tanımlarına dokunulmaz
    // (gerekçe: `ThemeDoc::imported_tokens`).
    let mut imported_tokens = token_overrides.clone();
    for name in STRUCTURAL_TOKENS {
        if let Some(val) = all_vars.get(name) {
            imported_tokens.insert(name.to_string(), resolve_var(val, &all_vars));
        }
    }

    // Kapsamı olduğu için yazılmayan, ama kontrollere gösterilmesi gereken
    // değerler (gerekçe: `ThemeDoc::seed_tokens`). `all_vars` küresel olanları
    // da içeriyor; onlar zaten `token_overrides`'ta, tekrar etmiyoruz.
    // Yapısal token'lar dışarıda: `ThemeDoc`'ta kendi alanları ve kendi
    // kontrolleri var, `emit_css` onları oradan yazıyor. Buraya da girselerdi
    // aynı değer iki farklı kaynaktan yönetiliyor olurdu.
    // Vurgu ailesi TÜM değişkenlerden aranıyor, yalnızca küresel olanlardan
    // değil: temalar renklerini çoğunlukla kipe bağlı bloklarda tanımlıyor.
    let accent_aliases = detect_accent_aliases(&all_vars, &all_vars, accent_value);

    // Yuvarlaklık ailesi: temanın `border-radius` bildirimlerinde kullandığı
    // kendi değişkenleri (gerekçe: `RadiusAlias`). Taban, içe aktarmadan gelen
    // kontrol yuvarlaklığı; o da yoksa sapma hesaplanamayacağı için aile boş
    // kalıyor ve kaydırıcı eski davranışını sürdürüyor.
    let radius_aliases = match control_corner_radius.or(current.control_corner_radius) {
        Some(base) => detect_radius_aliases(&rules, &all_vars, base),
        None => Vec::new(),
    };

    // Temanın `var()` ile gerçekten KULLANDIĞI adlar. Kullanılmayan bir
    // değişkeni yeniden yazmanın hiçbir etkisi olmaz; listeye alınırsa
    // yönetilen blok gereksiz yere şişer.
    let mut used_vars: BTreeSet<String> = BTreeSet::new();
    for rule in &rules {
        for (_, value) in parse_decls(&rule.body) {
            for name in var_names(&value) {
                used_vars.insert(name);
            }
        }
    }

    // Vurgu ve yuvarlaklık ailelerinin sahiplendiği adlar; onların kendi
    // sapma hesapları var, ikinci kez yazılmamalılar.
    let taken: BTreeSet<String> = accent_aliases
        .iter()
        .map(|a| a.name.clone())
        .chain(radius_aliases.iter().map(|a| a.name.clone()))
        .collect();

    let token_aliases = detect_token_aliases(&token_overrides, &all_vars, &used_vars, &taken);

    let seed_tokens: BTreeMap<String, String> = all_vars
        .iter()
        .filter(|(name, _)| {
            !token_overrides.contains_key(*name) && !STRUCTURAL_TOKENS.contains(&name.as_str())
        })
        .map(|(name, val)| (name.clone(), resolve_var(val, &all_vars)))
        .collect();

    let mut rule_overrides = BTreeMap::new();
    let mut consumed: Vec<(usize, usize)> = Vec::new();

    for rule in &rules {
        let normalized = normalize_selector(&rule.selector);

        // Koşullu kurallar (bir `@media`/`@supports` içindekiler) hiçbir zaman
        // tüketilmiyor: metinden çıkarılıp koşulsuz olarak yeniden üretilseler
        // her ekran boyutunda geçerli olurlardı.
        if rule.conditional() {
            continue;
        }

        if is_theme_scoped_block(&normalized) {
            // Tema kipine bağlı değişken blokları olduğu yerde kalıyor
            // (gerekçe: `is_theme_scoped_block`).
            continue;
        }

        if is_global_token_block(&normalized) {
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
        accent: accent_value,
        mode: current.mode,
        control_corner_radius: control_corner_radius.or(current.control_corner_radius),
        overlay_corner_radius: overlay_corner_radius.or(current.overlay_corner_radius),
        imports,
        token_overrides,
        rule_overrides,
        seed_tokens,
        // Dosyadan gelen her şey TABAN: kullanıcı henüz kendi ham CSS'ini
        // yazmadı, dolayısıyla `raw_css` boş başlıyor.
        accent_aliases,
        radius_aliases,
        token_aliases,
        // Kontrollere taşınmayan her kural burada, TEK TEK. Eskiden bu bir
        // metin parçasıydı ve sayfaya yabancı bir stil sayfası olarak
        // basılıyordu (gerekçe: `ThemeDoc::imported_rules`).
        imported_rules: decompose_rules(&body, &rules, &consumed),
        imported_tokens,
        // İçe aktarma anındaki vurgu; kullanıcının kaydırıcıya gerçekten
        // dokunup dokunmadığı buradan anlaşılıyor (gerekçe: `imported_accent`).
        imported_accent: Some(accent_value),
        imported_css: String::new(),
        raw_css: String::new(),
    }
}
