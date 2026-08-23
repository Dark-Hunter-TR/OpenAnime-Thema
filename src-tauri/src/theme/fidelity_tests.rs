//! Tur (parse → emit) sadakat testleri.
//!
//! `tests.rs` tek tek davranışları kısa parçalarla sabitliyor. Buradaki testler
//! tersinden çalışıyor: bütün bir tema dosyasını ayrıştırıp yeniden üretiyor ve
//! "çıkan CSS girdiyle aynı şeyi anlatıyor mu?" diye soruyor. Kullanıcının
//! gördüğü hata bu turda oluşuyordu — dosya açılıyor, ama içindekilerin bir
//! kısmı önizlemeye hiç ulaşmıyordu.
//!
//! Aşağıdaki üç tema SENTETİK. Topluluk temalarından kopyalanmadılar; onlarda
//! gözlenen ve ayrıştırıcıyı gerçekten zorlayan yapılar burada bilerek bir
//! araya getirildi:
//!
//!   * `TOKEN_THEME`     — bütün renkleri kipe bağlı bir blokta tanımlıyor,
//!                         `url(data:image/png;base64,…)` (değerin içinde `;`)
//!                         kullanıyor ve vurgu rampasının tek tek basamaklarını
//!                         elle eziyor.
//!   * `LAYERED_THEME`   — `@import`, `@font-face`, `@media` kırılımları ve
//!                         `var()` zincirleri.
//!   * `COMMENTED_THEME` — her bildirimin sonunda açıklama yorumu, `@import`
//!                         satırının ardında yorum, sorgu dizgili uzun adresler
//!                         ve tanınan bir seçicide kontrollerde karşılığı
//!                         olmayan bildirimler.
//!
//! Doğrulayıcılar bilerek `parse.rs`'in yardımcılarını KULLANMIYOR. Aynı
//! tarayıcıyla hem üretip hem denetlemek, tarayıcıdaki bir hatayı görünmez
//! kılardı: iki taraf da aynı yanlış bölmeyi yapar ve test yeşil kalırdı.

use std::collections::BTreeSet;

use crate::theme::models::{ImportedRule, ThemeDoc};
use crate::theme::parse::parse_foreign_css;

const TOKEN_THEME: &str = r#"
:root {
    --fds-accent-base: 206, 100%, 42%;
    --fds-accent-light-3: 0, 0%, 100%;
    --fds-accent-light-1: 0, 0%, 100%;
    --fds-control-corner-radius: 50px;
    --fds-overlay-corner-radius: 8px;
    --fds-acrylic-noise-asset: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==);
    --fds-acrylic-fallback-filter: blur(32px) saturate(200%);
}

.fds-theme-dark {
    color-scheme: dark !important;
    --fds-text-primary: hsl(0deg 0% 100%);
    --fds-text-secondary: hsl(0deg 0% 68.65% / 78.6%);
    --fds-solid-background-base: hsl(0, 0%, 13%);
    --fds-card-background-default: hsla(0, 0%, 100%, 5.12%);
}

body {
    color: var(--fds-text-primary);
    background-color: hsl(0deg 0% 0%) !important;
    background-size: cover;
}

#search-result {
    padding: 12px;
    border-radius: 3px !important;
}
"#;

const LAYERED_THEME: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;700&display=swap');

@font-face {
    font-family: 'Ornek';
    src: url('https://example.com/ornek.woff2') format('woff2');
    font-display: swap;
}

:root {
    --font-primary: 'Inter', system-ui, sans-serif;
    --radius-base: 10px;
    --radius-card: var(--radius-base);
    --logo-size: 20px;
    --accent: #7aa2f7;
}

.anime-card, .slider-card, .grid-view-item {
    border-radius: var(--radius-card);
    font-family: var(--font-primary);
}

@media (max-width: 768px) {
    :root {
        --logo-size: 14px;
        --radius-base: 4px;
    }

    .anime-card, .slider-card, .grid-view-item {
        border-radius: 4px;
    }
}
"#;

const COMMENTED_THEME: &str = r#"
/* ============================== */
/* BOLUM 0: HARICI KAYNAKLAR      */
/* ============================== */
@import url('https://fonts.googleapis.com/css2?family=Permanent+Marker&display=swap'); /* [Aciklama] Logo yazi tipi. */

:root {
    --boyut-logo: 55px;                       /* [Aciklama] Logonun boyutu. */
    --url-arkaplan: url('https://example.com/bg.png?token=abc.def_ghi&w=1920'); /* [Aciklama] Arka plan. */
    --ayar-yuvarlaklik: 20px;                 /* [Aciklama] Genel kose yuvarlakligi. */
    --fds-control-corner-radius: var(--ayar-yuvarlaklik) !important;
}

.slider.orientation-horizontal {
    block-size: 4px;
    inline-size: 100%;
    justify-content: flex-start;
    position: relative;
    color: red;
}

.ozel-rozet {
    background: linear-gradient(90deg, #f00, #00f);
}
"#;

fn themes() -> [(&'static str, &'static str); 3] {
    [
        ("token", TOKEN_THEME),
        ("layered", LAYERED_THEME),
        ("commented", COMMENTED_THEME),
    ]
}

/// Uygulamanın kontrollere bağladığı seçicilerden, yukarıdaki temaların
/// kullandıkları.
///
/// Tam liste (`src/lib/advanced.ts` -> `KNOWN_SELECTORS`) yerine bu kısa küme
/// yeterli: amaç listenin tamamını değil, TÜKETME YOLUNU sınamak. Buradaki iki
/// seçici de temalarda geçiyor, yani yol gerçekten çalıştırılıyor.
fn known_selectors() -> Vec<String> {
    vec![
        ".anime-card, .slider-card, .grid-view-item".to_string(),
        ".slider.orientation-horizontal".to_string(),
        "body".to_string(),
    ]
}

/// İthal kuralları okunabilir CSS'e çevirir — `emit_css`in sahiplik elemesine
/// takılmadan, kuralın modele girip girmediğini sınamak için.
fn imported_text(rules: &[ImportedRule]) -> String {
    rules
        .iter()
        .map(|rule| {
            let at = rule.at.join(" ");
            let note = rule.note.clone().unwrap_or_default();
            format!("{note}\n{at} {} {{ {} }}", rule.selector, rule.body)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse(css: &str) -> ThemeDoc {
    parse_foreign_css(css, &known_selectors(), &ThemeDoc::default())
}

// --- Bağımsız CSS okuyucusu ------------------------------------------------

/// `i` konumunda yorum ya da tırnaklı dizge başlıyorsa bitişini döndürür.
/// İkinci dönen değer, bölümün düzgün KAPANDIĞINI söyler.
fn skip(bytes: &[u8], i: usize) -> Option<(usize, bool)> {
    let len = bytes.len();
    if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
        let mut j = i + 2;
        while j + 1 < len && !(bytes[j] == b'*' && bytes[j + 1] == b'/') {
            j += 1;
        }
        let closed = j + 1 < len;
        return Some(((j + 2).min(len), closed));
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
        let closed = j < len;
        return Some(((j + 1).min(len), closed));
    }
    None
}

/// CSS'te tanımlanan özel özellik (`--ad`) adlarının kümesi.
fn custom_property_names(css: &str) -> BTreeSet<String> {
    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut out = BTreeSet::new();
    let mut i = 0usize;
    let mut braces = 0usize;
    let mut parens = 0usize;
    let mut decl_start = 0usize;

    while i < len {
        if let Some((next, _)) = skip(bytes, i) {
            i = next;
            continue;
        }
        match bytes[i] {
            b'{' => {
                braces += 1;
                decl_start = i + 1;
            }
            b'}' => {
                braces = braces.saturating_sub(1);
                decl_start = i + 1;
            }
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b';' if parens == 0 => decl_start = i + 1,
            b':' if parens == 0 && braces > 0 => {
                let name = css[decl_start..i].trim();
                if name.starts_with("--") && !name.contains(char::is_whitespace) {
                    out.insert(name.to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

/// Üst düzey kuralların seçicileri (at-kuralları dahil, normalize boşluklu).
fn selectors(css: &str) -> Vec<String> {
    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut out = Vec::new();
    let mut i = 0usize;
    let mut braces = 0usize;
    let mut start = 0usize;

    while i < len {
        if let Some((next, _)) = skip(bytes, i) {
            i = next;
            continue;
        }
        match bytes[i] {
            b'{' => {
                if braces == 0 {
                    let raw = strip_comments_naive(&css[start..i]);
                    // Seçicinin önünde bildirim biçimli bir at-kuralı
                    // (`@import …;`) durabilir. Seçici üst düzey `;`
                    // içeremeyeceğine göre gerçek seçici sonuncu `;`'den
                    // sonrası. Bu kırpma olmadan `@import`i başa taşıyan her
                    // üretim "seçici kayboldu" gibi görünürdü.
                    let tail = raw.rsplit(';').next().unwrap_or(&raw);
                    let sel = tail.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !sel.is_empty() {
                        out.push(sel);
                    }
                }
                braces += 1;
            }
            b'}' => {
                braces = braces.saturating_sub(1);
                if braces == 0 {
                    start = i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    out
}

fn strip_comments_naive(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(s) = rest.find("/*") {
        out.push_str(&rest[..s]);
        match rest[s + 2..].find("*/") {
            Some(e) => rest = &rest[s + 2 + e + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

#[derive(Default)]
struct Balance {
    /// Bir bildirimin ortasında açık kalan parantezler — kesilmiş değerin
    /// imzası. Yeniden üretilen CSS'te SIFIR olmak zorunda.
    open_paren_at_rule_end: Vec<String>,
    /// Dosya sonunda kapanmamış süslü parantez sayısı.
    ///
    /// Sıfır beklenmiyor, GİRDİYLE karşılaştırılıyor: gerçek temalarda
    /// kapanmamış parantez görülüyor ve ayrıştırıcının işi kullanıcının
    /// CSS'ini sessizce "düzeltmek" değil, olduğu gibi taşımak.
    unclosed_braces: usize,
    /// Kapanmamış yorum ya da tırnak.
    unclosed_noise: usize,
}

fn balance(css: &str) -> Balance {
    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut b = Balance::default();
    let mut i = 0usize;
    let mut braces = 0usize;
    let mut parens = 0usize;

    while i < len {
        if let Some((next, closed)) = skip(bytes, i) {
            if !closed {
                b.unclosed_noise += 1;
            }
            i = next;
            continue;
        }
        match bytes[i] {
            b'(' => parens += 1,
            b')' => parens = parens.saturating_sub(1),
            b'{' => braces += 1,
            b'}' => {
                if parens != 0 {
                    let from = i.saturating_sub(120);
                    b.open_paren_at_rule_end.push(format!(
                        "{} bayttaki `}}`'a {} açık parantezle gelindi: …{}",
                        i,
                        parens,
                        &css[from..i]
                    ));
                    parens = 0;
                }
                braces = braces.saturating_sub(1);
            }
            _ => {}
        }
        i += 1;
    }

    if parens != 0 {
        b.open_paren_at_rule_end
            .push(format!("dosya sonunda {parens} açık parantez"));
    }
    b.unclosed_braces = braces;
    b
}

// --- Testler ----------------------------------------------------------------

/// Yeniden üretilen CSS'te kesilmiş değer olmamalı.
///
/// Somut karşılığı: `--fds-acrylic-noise-asset` değeri
/// `url(data:image/png;base64,…)`. Bildirimleri naif `split(';')` ile bölen
/// eski ayrıştırıcı değeri `url(data:image/png` diye kesiyor, üretilen blokta
/// parantez açık kalıyor ve tarayıcı sonraki bildirimleri de yutuyordu.
#[test]
fn emitted_css_has_no_truncated_values() {
    for (name, css) in themes() {
        let before = balance(css);
        let after = balance(&parse(css).emit_css());

        assert!(
            after.open_paren_at_rule_end.is_empty(),
            "{name}: üretilen CSS'te kesilmiş değer:\n{}",
            after.open_paren_at_rule_end.join("\n")
        );
        assert_eq!(
            after.unclosed_braces, before.unclosed_braces,
            "{name}: süslü parantez dengesi girdiden ayrıldı"
        );
        assert_eq!(
            after.unclosed_noise, before.unclosed_noise,
            "{name}: kapanmamış yorum/tırnak sayısı girdiden ayrıldı"
        );
    }
}

/// Girdideki her `--değişken` TANIMI üretilen CSS'te de bulunmalı.
///
/// Değişken ya `token_overrides`'a taşınmış (yönetilen bloğa yazılır) ya da ham
/// metinde bırakılmış olmalı; ikisi de olmuyorsa tema o değeri kaybetmiş
/// demektir ve kullanıcı "hiçbir şey uygulanmadı" görür.
#[test]
fn every_custom_property_definition_survives() {
    // Vurgu rampasının basamakları bu denetimin dışında: değeri tabandan
    // türetilenle AYNI olan bir basamak bilerek ikinci kez yazılmıyor (yoksa
    // her tur blok biraz daha şişerdi) ve yazılmadığında da site aynı rengi
    // hesaplıyor. Türetilenden AYRILAN basamağın korunması ayrı ve keskin bir
    // testte: `tests.rs -> hand_written_accent_step_survives`.
    let regenerated: BTreeSet<String> = crate::theme::parse::DERIVED_TOKEN_NAMES
        .iter()
        .map(|n| format!("--fds-{n}"))
        .collect();

    for (name, css) in themes() {
        let before = custom_property_names(css);
        let after = custom_property_names(&parse(css).emit_css());

        let lost: Vec<_> = before
            .difference(&after)
            .filter(|n| !regenerated.contains(*n))
            .cloned()
            .collect();
        assert!(
            lost.is_empty(),
            "{name}: {} değişken tanımı kayboldu: {:?}",
            lost.len(),
            lost
        );
    }
}

/// Girdideki her seçici üretilen CSS'te de bulunmalı.
#[test]
fn every_selector_survives() {
    for (name, css) in themes() {
        let after: BTreeSet<String> = selectors(&parse(css).emit_css()).into_iter().collect();

        let mut lost = Vec::new();
        for sel in selectors(css) {
            // Tüketilen küresel token blokları (`:root { yalnızca --x }`)
            // yönetilen bloğun `:root, .fds-theme-light, .fds-theme-dark`
            // seçicisinde birleşiyor — birebir eşleşme değil, kapsamın
            // korunması aranıyor.
            if sel.contains(":root") && after.iter().any(|s| s.contains(":root")) {
                continue;
            }
            // Kontrol kuralları özgüllük önekiyle yazılıyor (bkz.
            // `control_rules_outweigh_the_imported_theme`); karşılaştırma
            // öneksiz hâl üzerinden yapılmalı.
            if !after.contains(&sel)
                && !after
                    .iter()
                    .any(|s| crate::theme::parse::strip_specificity_boost(s) == sel)
            {
                lost.push(sel);
            }
        }
        assert!(lost.is_empty(), "{name}: kaybolan seçiciler: {lost:?}");
    }
}

/// `@import` adresleri temiz çıkarılmalı.
///
/// `COMMENTED_THEME`, `@import url('…'); /* [Aciklama] … */` yazıyor. Satır
/// bazlı eski çıkarım satır sonundaki yorumu adresin parçası sanıyor ve
/// yeniden üretilen `@import url("https://…swap'); /* … */")` geçersiz oluyordu
/// — yazı tipi hiç yüklenmiyordu.
///
/// `;` denetimi bilerek YOK: Google Fonts adresleri ağırlıkları noktalı
/// virgülle ayırıyor (`…Inter:wght@300;400;700`) ve bu geçerli bir adres.
#[test]
fn import_urls_are_clean() {
    let mut seen = 0usize;

    for (name, css) in themes() {
        for url in parse(css).imports {
            seen += 1;
            assert!(
                !url.contains("/*") && !url.contains("*/"),
                "{name}: adrese yorum sızmış: {url}"
            );
            assert!(
                !url.contains(char::is_whitespace),
                "{name}: adreste boşluk var: {url}"
            );
            assert!(
                !url.contains('"') && !url.contains('\''),
                "{name}: adreste tırnak kalmış: {url}"
            );
            assert!(
                url.starts_with("http://") || url.starts_with("https://"),
                "{name}: adres bir URL'ye benzemiyor: {url}"
            );
        }
    }

    assert_eq!(seen, 2, "temalardaki @import sayısı değişmiş — testler artık farklı şeyi sınıyor");
}

/// Kontrollere gösterilecek token'lar toplanıyor ama CSS'e yazılmıyor.
///
/// İkisi birden önemli: toplanmasaydı kullanıcı temayı açtığında bütün görsel
/// kontroller boş görünürdü (bildirilen asıl şikâyet). Yazılsaydı kipe bağlı
/// değerler kapsamlarını kaybederdi.
#[test]
fn seed_tokens_feed_controls_without_entering_css() {
    let parsed = parse(TOKEN_THEME);

    assert!(
        parsed.seed_tokens.contains_key("--fds-text-primary"),
        "`.fds-theme-dark` altındaki metin rengi kontrole ulaşmıyor"
    );

    // Aynı anahtar iki haritada birden olmamalı — hangisinin okunduğu
    // belirsizleşirdi.
    for name in parsed.seed_tokens.keys() {
        assert!(
            !parsed.token_overrides.contains_key(name),
            "{name} hem yazılan hem yalnızca gösterilen haritada"
        );
    }

    // Üretilen CSS bu değerleri KENDİ yazmıyor; yalnızca temanın kendi
    // bloğundan (ham CSS) geliyorlar.
    let emitted = parsed.emit_css();
    let start = emitted.find("/* <oa:tokens> */").expect("yönetilen blok yok");
    let end = emitted.find("/* </oa:tokens> */").expect("yönetilen blok kapanmıyor");
    let managed = &emitted[start..end];
    for name in parsed.seed_tokens.keys() {
        assert!(
            !managed.contains(&format!("{name}:")),
            "{name} yönetilen bloğa sızmış"
        );
    }
}

/// Kipe bağlı token bloğu metinde, kapsamında kalmalı.
///
/// Taşınsaydı yeniden üretilen `:root, .fds-theme-light, .fds-theme-dark`
/// bloğu koyu kip renklerini açık kipe de yazardı.
#[test]
fn theme_scoped_tokens_stay_in_their_scope() {
    let parsed = parse(TOKEN_THEME);

    assert!(
        imported_text(&parsed.imported_rules).contains(".fds-theme-dark"),
        "`.fds-theme-dark` bloğu taban katmanında korunmalı"
    );
    assert!(
        !parsed.token_overrides.contains_key("--fds-solid-background-base"),
        "kipe bağlı değer küresel bloğa taşınmış"
    );
}

/// `@media` içindeki değişkenler küresel bloğa sızmamalı.
#[test]
fn media_query_values_do_not_become_global() {
    let parsed = parse(LAYERED_THEME);

    assert_eq!(
        parsed.token_overrides.get("--logo-size").map(String::as_str),
        Some("20px"),
        "koşulsuz değer yerine mobil kırılımdaki değer alınmış"
    );
    assert!(
        imported_text(&parsed.imported_rules).contains("@media"),
        "`@media` bloğu taban katmanında korunmalı"
    );
}

/// Tanınan bir seçicideki, kontrollerde karşılığı olmayan bildirimler kalmalı.
///
/// `.slider.orientation-horizontal` "Oynatıcı" bölümüne bağlı; kontroller
/// gövdedeki beş bildirimden yalnızca birkaçını biliyor. Gövdeyi komple
/// devralan eski yol, geri kalanını kullanıcı hiçbir şeye dokunmadan
/// siliyordu. (Birleştirmenin kendisi ön yüzde: `src/lib/cssDecls.ts`.)
#[test]
fn known_selector_keeps_its_extra_declarations() {
    let parsed = parse(COMMENTED_THEME);
    let body = parsed
        .rule_overrides
        .get(".slider.orientation-horizontal")
        .expect("tanınan kural kontrollere alınmamış");

    for decl in ["block-size", "inline-size", "justify-content", "position", "color"] {
        assert!(body.contains(decl), "`{decl}` bildirimi kayıp");
    }
}

// --- Gelişmiş temalarla rekabet -------------------------------------------

/// Midnight biçimli tema: paletini `html.fds-theme-dark` altında tanımlıyor ve
/// kendi vurgu adlarını `!important` ile kullanıyor.
const RIVAL_THEME: &str = r#"
html.fds-theme-dark {
    --accent-primary: #5594ff;
    --accent-secondary: #b8d4ff;
    --accent-tertiary: #6b9ef5;
    --fds-solid-background-base: #0b0d12;
    --fds-text-primary: #e6e9ef;
}

.list-item.selected {
    border-left: 2px solid var(--accent-primary) !important;
}
"#;

/// Bir seçicinin özgüllüğü: (id, sınıf+sözde sınıf+öznitelik, eleman).
///
/// Kasten basit — testteki seçiciler bilindiği için tam bir CSS ayrıştırıcısına
/// gerek yok.
fn specificity(selector: &str) -> (usize, usize, usize) {
    let ids = selector.matches('#').count();
    let classes = selector.matches('.').count() + selector.matches(':').count();
    let elements = selector
        .split(['.', ':', '#', ' ', '>'])
        .filter(|p| !p.is_empty() && p.chars().next().is_some_and(|c| c.is_ascii_alphabetic()))
        .count();
    (ids, classes, elements)
}

/// Yönetilen blok, içe aktarılan temanın palet bloğunu YENMELİ.
///
/// Kullanıcının bildirdiği "renkler dışında hiçbir şeyi değiştiremiyorum"
/// bunun karşılığıydı: tema paletini `html.fds-theme-dark` (0,1,1) altında
/// tanımlıyor, bizim blok ise `:root, …` (0,1,0) yazıyordu. Blok temadan SONRA
/// gelse bile özgüllük yetmediği için yüzey/metin/kart gibi temanın tanımladığı
/// her token bizi eziyordu. Vurgunun çalışması yanıltıcıydı — temalar
/// `--fds-accent-base`'i genelde hiç tanımlamıyor, o yüzden orada rakip yoktu.
#[test]
fn managed_block_outranks_imported_palette() {
    let mut parsed = parse(RIVAL_THEME);
    parsed.token_overrides.insert("--fds-solid-background-base".into(), "#123456".into());
    let css = parsed.emit_css();

    let block = css
        .lines()
        .find(|l| l.contains(":root") && l.contains(","))
        .expect("yönetilen blok seçicisi bulunamadı");

    assert!(
        specificity(block) > specificity("html.fds-theme-dark"),
        "yönetilen blok ({block}) temanın palet bloğunu yenemiyor"
    );
}

/// İçe aktarılan tema yoksa özgüllük yükseltilmiyor.
///
/// Yükseltmenin bir bedeli var: kullanıcının "Ham CSS" kutusuna yazdığı sade
/// bir `:root` kuralı artık bloğu ezemez. O bedeli yalnızca gerçekten
/// gerektiği yerde ödüyoruz.
#[test]
fn plain_theme_keeps_the_simple_selector() {
    let doc = ThemeDoc {
        accent: [280.0, 70.0, 50.0],
        ..Default::default()
    };
    assert!(doc.emit_css().contains(":root,\n.fds-theme-light,\n.fds-theme-dark"));
}

/// Temanın KENDİ vurgu değişkenleri kaydırıcıyı izlemeli.
///
/// "Renk değişimleri tam değil" şikâyetinin karşılığı: tema her yeri
/// `var(--accent-primary) !important` ile boyuyor, biz ise yalnızca
/// `--fds-accent-*` yazıyorduk. Aile artık değere bakılarak bulunuyor ve
/// kaydırıcıyla birlikte kayıyor.
#[test]
fn theme_accent_family_follows_the_slider() {
    let mut parsed = parse(RIVAL_THEME);

    let names: Vec<&str> = parsed.accent_aliases.iter().map(|a| a.name.as_str()).collect();
    for expected in ["--accent-primary", "--accent-secondary", "--accent-tertiary"] {
        assert!(names.contains(&expected), "{expected} vurgu ailesine girmemiş: {names:?}");
    }
    assert!(
        !names.iter().any(|n| n.starts_with("--fds-")),
        "`--fds-*` ailesi rampadan yazılıyor, ikinci kez girmemeli: {names:?}"
    );

    // Kaydırıcıyı kırmızıya çek: üç değişken de yeni tona taşınmalı.
    parsed.accent = [0.0, 80.0, 55.0];
    let css = parsed.emit_css();
    for name in ["--accent-primary", "--accent-secondary", "--accent-tertiary"] {
        assert!(css.contains(&format!("{name}: hsl(")), "{name} yeniden yazılmamış");
    }
    assert!(
        !css.contains("#5594ff") || imported_text(&parsed.imported_rules).contains("#5594ff"),
        "eski vurgu değeri yönetilen bloğa sızmış"
    );
}

/// Aile ilişkisi korunuyor: açık ton açık, koyu ton koyu kalıyor.
///
/// Mutlak renkler saklansaydı aile ilk oynatmada tek renge çökerdi; sapma
/// saklandığı için sıralama duruyor.
#[test]
fn accent_family_keeps_its_shades() {
    let mut parsed = parse(RIVAL_THEME);
    parsed.accent = [0.0, 80.0, 55.0];

    let by_name = |n: &str| {
        parsed
            .accent_aliases
            .iter()
            .find(|a| a.name == n)
            .map(|a| (parsed.accent[2] + a.delta[2]).clamp(0.0, 100.0))
            .expect(n)
    };

    // Kaynakta `--accent-secondary` (#b8d4ff) en açık ton.
    assert!(
        by_name("--accent-secondary") > by_name("--accent-primary"),
        "açık ton, ana tondan açık kalmalı"
    );
}



// --- İçe aktarılan tema modele giriyor -------------------------------------

/// Kullanıcının bildirdiği asıl hata: içe aktarılan dosya, önizlemede sitenin
/// kendi tema yuvasına konan YABANCI bir stil sayfası gibi davranıyordu.
///
/// Ölçüm netti: 103 seçicilik gerçek bir temanın 101 seçicisi modele hiç
/// girmiyor, 101 KB'lık tek parça metin olarak sayfaya basılıyordu. Bu test o
/// yolu kapatıyor — dosyadan gelen her kural modelde, tek tek duruyor ve
/// üretilen CSS'in tamamı modelden çıkıyor.
#[test]
fn nothing_passes_through_as_raw_text() {
    for (name, css) in themes() {
        let parsed = parse(css);
        assert!(
            parsed.imported_css.is_empty(),
            "{name}: ham geçiş bloğu üretilmemeli"
        );
        assert!(
            !parsed.imported_rules.is_empty(),
            "{name}: kurallar modele girmemiş"
        );
        for rule in &parsed.imported_rules {
            assert!(
                !rule.selector.is_empty() && !rule.body.trim().is_empty(),
                "{name}: boş kural modele girmiş"
            );
        }
    }
}

/// Kontrolden değiştirilen bir token, temanın `!important` bildirimini DEVRALIR.
///
/// Bu, "içe aktarılan dosya özelleştirilemiyor" şikâyetinin çekirdeği. Tema
/// bildirimlerini `!important` ile yazıyor (örnek temada 255 tane) ve
/// `!important` bir bildirimi ne sonra gelmek ne de daha özgül olmak yener —
/// yani eski taban/kontrol katmanlaması ne yaparsa yapsın kaybediyordu.
/// Kullanıcı kaydırıcıyı oynatıyor, ekranda hiçbir şey değişmiyordu.
///
/// Çözüm katman değil sahiplik: çakışan bildirim ithal kuraldan ÇIKARILIYOR.
#[test]
fn changing_a_control_takes_over_an_important_declaration() {
    const THEME: &str = r#"
html.fds-theme-dark {
    --fds-text-primary: #111111 !important;
    --fds-solid-background-base: #222222 !important;
}
"#;

    let mut doc = parse(THEME);
    assert!(
        doc.emit_css().contains("#111111"),
        "dokunulmadan önce temanın kendi değeri durmalı"
    );

    // Kullanıcı metin rengini kontrolden değiştiriyor.
    doc.token_overrides
        .insert("--fds-text-primary".into(), "#ff0000".into());

    let css = doc.emit_css();
    assert!(
        !css.contains("#111111"),
        "kontrolün devraldığı bildirim ithal kuraldan çıkarılmalı:\n{css}"
    );
    assert!(css.contains("#ff0000"), "kontrolün değeri yazılmalı:\n{css}");

    // Dokunulmayan komşu bildirim yerinde kalmalı — sahiplik bildirim
    // bazında, blok bazında değil.
    assert!(
        css.contains("#222222"),
        "dokunulmayan bildirim silinmemeli:\n{css}"
    );
}

/// Ve tersi: hiçbir kontrol oynatılmadıysa temanın kipe bağlı değerleri durur.
///
/// Ölçüt "kontrolde karşılığı var mı" olsaydı, tema açılır açılmaz
/// `.fds-theme-dark` altındaki koyu renkleri silinirdi — kullanıcı daha hiçbir
/// şeye dokunmadan tema bozulmuş olurdu.
#[test]
fn untouched_theme_values_are_left_alone() {
    let parsed = parse(TOKEN_THEME);
    let css = parsed.emit_css();

    for value in ["hsl(0deg 0% 100%)", "hsl(0, 0%, 13%)"] {
        assert!(
            css.contains(value),
            "dokunulmamış tema değeri kaybolmuş: {value}\n{css}"
        );
    }
}

/// `@media` sarmalayıcısı yeniden ÜRETİLİYOR, sadece korunmuyor.
///
/// Kural modele girdiğine göre metinden kopyalanamaz; sarmalayıcı zinciriyle
/// birlikte saklanıp yeniden yazılması gerekiyor. Zincir kaybolsaydı yalnızca
/// dar ekranda geçerli bir kural her ekranda geçerli olurdu.
#[test]
fn media_wrapper_is_rebuilt_around_its_rules() {
    let parsed = parse(LAYERED_THEME);
    let css = parsed.emit_css();

    let media = css
        .find("@media (max-width: 768px)")
        .expect("`@media` sarmalayıcısı yazılmamış");
    let inner = css[media..]
        .find("border-radius: 4px")
        .expect("`@media` içindeki kural yazılmamış");
    let close = css[media..]
        .find("\n}")
        .expect("`@media` bloğu kapanmamış");

    assert!(
        inner < close + 2,
        "kural sarmalayıcının İÇİNDE kalmalı:\n{}",
        &css[media..media + 400.min(css.len() - media)]
    );
}

/// Kuraldan önceki açıklama yorumu seçicinin parçası DEĞİL.
///
/// Ayrılmadığında seçici `/* [Açıklama] … */ .slider…` oluyordu. CSS bunu
/// kabul ediyor, ama kontrole bağlanabilecek kurallar bulunamıyordu: tanınan
/// seçici eşleşmesi tam metin karşılaştırması yapıyor ve yorum yüzünden hiç
/// tutmuyordu. Elle yazılmış temaların hepsi açıklamalı, yani bu istisna değil
/// kuraldı.
#[test]
fn leading_comment_is_not_part_of_the_selector() {
    let parsed = parse(COMMENTED_THEME);

    for rule in &parsed.imported_rules {
        assert!(
            !rule.selector.starts_with("/*"),
            "seçiciye yorum karışmış: {:?}",
            rule.selector
        );
    }

    // Ve yorum kaybolmuyor: kuralın notu oluyor.
    const NOTED: &str = r#"
/* [Aciklama] Rozetin gorunumu. */
.ozel-rozet { color: red; }
"#;
    let noted = parse(NOTED);
    let rule = noted
        .imported_rules
        .iter()
        .find(|rule| rule.selector == ".ozel-rozet")
        .expect("kural modele girmemiş");
    assert_eq!(
        rule.note.as_deref(),
        Some("/* [Aciklama] Rozetin gorunumu. */")
    );
    assert!(
        noted.emit_css().contains("[Aciklama] Rozetin gorunumu."),
        "not yeniden üretilen CSS'e yazılmalı"
    );
}

/// Açıklamalı bir tema, tanınan seçicisini kontrollere GERÇEKTEN bağlıyor.
///
/// `known_selector_keeps_its_extra_declarations` bunun gövde tarafını
/// sınıyordu; buradaki soru daha önce gelen adım: yorumlu yazılmış bir dosyada
/// eşleşme kuruluyor mu.
#[test]
fn commented_theme_still_binds_to_controls() {
    let parsed = parse(COMMENTED_THEME);
    assert!(
        parsed.rule_overrides.contains_key(".slider.orientation-horizontal"),
        "açıklamalı dosyada tanınan seçici eşleşmemiş: {:?}",
        parsed.rule_overrides.keys().collect::<Vec<_>>()
    );
}

/// Kurallar ve notları, kod editörü turundan geçince yerinde kalmalı.
///
/// Kod editörü her tuş vuruşunda metni geri okuyor. İthal kurallar artık
/// metinden ayrıştırıldığı için bu turun kayıpsız olması şart — aksi hâlde
/// kullanıcı editöre tek harf yazdığında temasının bir kısmı erirdi.
#[test]
fn imported_rules_survive_the_editor_roundtrip() {
    for (name, css) in themes() {
        let parsed = parse(css);
        let round = crate::theme::parse::parse_css(
            &parsed.emit_css(),
            &known_selectors(),
            &parsed,
        );

        let before = selectors(&parsed.emit_css());
        let after = selectors(&round.emit_css());
        assert_eq!(before, after, "{name}: tur sonrası seçiciler değişti");
    }
}


/// Tek bir kontrolü oynatmak, DOKUNULMAYAN tema değerlerini silmemeli.
///
/// Ön yüz bir kontrol değiştiğinde token haritasının TAMAMINI geri yazıyor
/// (`+page.svelte` -> `tokenMap`). Sahiplik ölçütü "haritada var mı" olsaydı,
/// kullanıcı yuvarlaklığı değiştirdiğinde temanın kipe bağlı bütün renkleri
/// birden "değiştirildi" sayılır ve silinirdi. Ölçüt bu yüzden değer bazlı:
/// değer temanın verdiğiyle aynı kaldığı sürece sahibi tema.
#[test]
fn writing_back_unchanged_values_changes_nothing() {
    let mut doc = parse(TOKEN_THEME);

    // Ön yüzün yaptığı gibi: kontrollere beslenen her değeri geri yaz.
    let seeded = doc.seed_tokens.clone();
    for (name, value) in &seeded {
        doc.token_overrides.insert(name.clone(), value.clone());
    }

    let css = doc.emit_css();
    for value in ["hsl(0deg 0% 100%)", "hsl(0, 0%, 13%)"] {
        assert!(
            css.contains(value),
            "dokunulmamış tema değeri geri yazma sırasında silinmiş: {value}"
        );
    }

    // Ama gerçekten değiştirilen bir değer yine devralınmalı.
    doc.token_overrides
        .insert("--fds-solid-background-base".into(), "#abcdef".into());
    let css = doc.emit_css();
    assert!(css.contains("#abcdef"), "kontrolün değeri yazılmalı");
    assert!(
        !css.contains("hsl(0, 0%, 13%)"),
        "devralınan bildirim ithal kuraldan çıkarılmalı:\n{css}"
    );
}



// --- Kontroller temayı yenebiliyor mu ----------------------------------------

/// Kontrollerle çakışan, gerçek bir temadan alınmış üç desen.
///
/// Üçü de `ornek/REzero v10.2.css` denetiminde bulundu ve üçünde de kontrol
/// hiçbir şey yapamıyordu:
///
///   * `a.anime-card.svelte-…` — kontrolün seçicisinin NİTELENMİŞ hâli, yani
///     daha özgül; üstelik `!important`.
///   * `div.comment.svelte-…`  — aynısı, iki bildirimle.
///   * `:not(:has(#…))`        — `:has()` içindeki id, özgüllüğü ID düzeyine
///     çıkarıyor. Sınıf sütununu kaç kez artırırsak artıralım yenilemez;
///     önekin ID bileşeni bu yüzden var.
const HEAVY_THEME: &str = r#"
.calendar-card,
a.anime-card.svelte-1w17qyc,
.slider-card {
    border-radius: 24px !important;
}

div.comment.svelte-1snun1g {
    border-radius: 30px !important;
    padding: 20px !important;
}

.list-item:not(:has(#lottie-player)) svg path {
    fill: #ff00ff !important;
}
"#;

/// Bir seçicinin özgüllüğü: (id, sınıf/sözde-sınıf/öznitelik, eleman).
///
/// `::before` eleman, `:hover` sınıf düzeyinde; `:not()/:is()/:has()` kendisi
/// saymaz, yalnızca argümanı sayar. Bu ayrımlar olmadan karşılaştırma yanlış
/// taraf lehine sonuçlanıyor.
fn specificity_of(selector: &str) -> (usize, usize, usize) {
    let b = selector.as_bytes();
    let (mut id, mut cls, mut el) = (0usize, 0usize, 0usize);
    let mut i = 0usize;
    let ident = |c: u8| matches!(c, b'-' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9');

    while i < b.len() {
        match b[i] {
            b'#' => {
                id += 1;
                i += 1;
                while i < b.len() && ident(b[i]) {
                    i += 1;
                }
            }
            b'.' => {
                cls += 1;
                i += 1;
                while i < b.len() && ident(b[i]) {
                    i += 1;
                }
            }
            b'[' => {
                cls += 1;
                while i < b.len() && b[i] != b']' {
                    i += 1;
                }
                i += 1;
            }
            b':' => {
                let pseudo_element = i + 1 < b.len() && b[i + 1] == b':';
                let name_start = if pseudo_element { i + 2 } else { i + 1 };
                i = name_start;
                while i < b.len() && ident(b[i]) {
                    i += 1;
                }
                let name = selector[name_start..i].to_ascii_lowercase();
                let transparent = matches!(name.as_str(), "not" | "is" | "has" | "where");
                if pseudo_element {
                    el += 1;
                } else if !transparent {
                    cls += 1;
                }
                if i < b.len() && b[i] == b'(' {
                    let start = i + 1;
                    let mut depth = 1usize;
                    i += 1;
                    while i < b.len() && depth > 0 {
                        if b[i] == b'(' {
                            depth += 1;
                        }
                        if b[i] == b')' {
                            depth -= 1;
                        }
                        i += 1;
                    }
                    if name != "where" {
                        let inner = specificity_of(&selector[start..i.saturating_sub(1)]);
                        id += inner.0;
                        cls += inner.1;
                        el += inner.2;
                    }
                }
            }
            c if ident(c) => {
                el += 1;
                while i < b.len() && ident(b[i]) {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    (id, cls, el)
}

/// Kontrolün yazdığı kural, temanın `!important` bildirimini yenmeli.
///
/// Kullanıcının bildirdiği hata: "birçok ayarı değiştirsem de yansımıyor".
/// Ölçüldüğünde sebep tekti — kontrol kuralları `!important` taşımıyordu ve
/// tema kendi bildirimlerini hem `!important` ile hem daha özgül seçicilerle
/// yazıyordu. `!important` bir bildirimi ne sonra gelmek ne daha özgül olmak
/// yener; iki taraf da `!important` olduğunda ise özgüllük karar verir.
/// Bu yüzden ikisi birden gerekiyor.
#[test]
fn control_rules_outweigh_the_imported_theme() {
    let known = vec![
        ".anime-card, .slider-card, .grid-view-item".to_string(),
        ".comment".to_string(),
        ".bottom-controls button svg path".to_string(),
    ];
    let mut doc = parse_foreign_css(HEAVY_THEME, &known, &ThemeDoc::default());

    doc.rule_overrides.insert(
        ".anime-card, .slider-card, .grid-view-item".into(),
        "border-radius: 4px;".into(),
    );
    doc.rule_overrides
        .insert(".comment".into(), "border-radius: 2px; padding: 1px;".into());
    doc.rule_overrides
        .insert(".bottom-controls button svg path".into(), "fill: #000000;".into());

    let css = doc.emit_css();
    let managed_at = css.find(crate::theme::models::TOKENS_OPEN).expect("yönetilen blok yok");
    let managed = &css[managed_at..];

    // 1. Her kontrol bildirimi `!important` taşımalı.
    for decl in ["border-radius: 4px !important", "padding: 1px !important", "fill: #000000 !important"] {
        assert!(managed.contains(decl), "`{decl}` yazılmamış:\n{managed}");
    }

    // 2. Ve özgüllükte temayı yenmeli — her rakip için tek tek.
    let rivals = [
        "a.anime-card.svelte-1w17qyc",
        "div.comment.svelte-1snun1g",
        ".list-item:not(:has(#lottie-player)) svg path",
    ];
    for (control, rival) in known.iter().zip(rivals) {
        let first = control.split(',').next().unwrap().trim();
        let ours = specificity_of(&format!("{} {first}", crate::theme::emit::SPECIFICITY_BOOST));
        let theirs = specificity_of(rival);
        assert!(
            ours > theirs,
            "kontrol kuralı temayı yenemiyor: {ours:?} <= {theirs:?} ({first} ↔ {rival})"
        );
    }
}

/// Ağırlık YALNIZCA ithal tema varken uygulanmalı.
///
/// Bedeli var: `!important`, kullanıcının "Ham CSS" kutusunun kontrolleri sade
/// bir kuralla ezmesini engelliyor. Rakip olmadığında o bedel ödenmemeli ve
/// çıktı sade kalmalı.
#[test]
fn plain_document_keeps_clean_rules() {
    let mut doc = ThemeDoc::default();
    doc.rule_overrides
        .insert(".comment".into(), "border-radius: 2px;".into());

    let css = doc.emit_css();
    assert!(css.contains(".comment {"), "sade seçici beklenirdi:\n{css}");
    assert!(
        !css.contains(crate::theme::emit::SPECIFICITY_BOOST),
        "tema yokken özgüllük öneki yazılmamalı:\n{css}"
    );
    assert!(
        !css.contains("border-radius: 2px !important"),
        "tema yokken `!important` eklenmemeli:\n{css}"
    );
}

/// Önek emit'te eklenip parse'ta sökülmeli.
///
/// Sökülmezse iki şey birden bozulur: anahtar artık kontrolün tanıdığı seçici
/// olmaz (kontrol kendi kuralını bulamaz) ve her turda bir önek daha eklenip
/// seçici sonsuza kadar uzar.
#[test]
fn specificity_boost_does_not_accumulate() {
    let known = vec![".comment".to_string()];
    let mut doc = parse_foreign_css(HEAVY_THEME, &known, &ThemeDoc::default());
    doc.rule_overrides
        .insert(".comment".into(), "border-radius: 2px;".into());

    let mut css = doc.emit_css();
    for turn in 0..3 {
        let round = crate::theme::parse::parse_css(&css, &known, &doc);
        assert!(
            round.rule_overrides.contains_key(".comment"),
            "tur {turn}: anahtar bozuldu: {:?}",
            round.rule_overrides.keys().collect::<Vec<_>>()
        );
        css = round.emit_css();
        // Değişmez: hiçbir seçici öneki İKİ kez taşımamalı. Sayıya değil
        // tekrara bakıyoruz — kural sayısı turlar arasında değişebilir, ama
        // aynı seçicide iki önek her zaman hatadır.
        let doubled = format!("{0}{0}", crate::theme::emit::SPECIFICITY_BOOST);
        assert!(!css.contains(&doubled), "tur {turn}: önek birikiyor:\n{css}");
    }
}

// --- Vurgu ailesi doğru sınırlarda mı ----------------------------------------

/// Mavi vurgulu, alfa katmanları yoğun bir tema.
///
/// Değişkenler kullanıcının bildirdiği bozulmadan alındı: vurgu kaydırıcısı
/// bu temanın ince tint'lerini ve gri/beyaz tonlarını da "vurgu ailesi"
/// sayıyordu.
const TINTED_THEME: &str = r#"
:root {
    --fds-accent-base: 217.9, 86.9%, 76.1%;
    --accent-primary: #8db4f7;
    --accent-secondary: #b8d4ff;
    --accent-tertiary: #6b9ef5;
    --bg-control: rgba(141, 180, 247, 0.04);
    --bg-menu-selected: rgba(141, 180, 247, 0.12);
    --border-card: rgba(141, 180, 247, 0.1);
    --mn-white: #e8eaf2;
    --mn-muted: #4a4f68;
    --mn-sub1: #9da3bb;
    --text-primary: #e8eaf2;
}
"#;

/// Vurgu ailesine alınan bir değişkenin ALFASI korunmalı.
///
/// Kullanıcının bildirdiği bozulmanın tam karşılığı: `--bg-control` temada
/// `rgba(141, 180, 247, 0.04)` — bir girdi zemini için %4'lük tint. Alfa
/// düşürülünce opak `hsl(...)` olarak geri yazılıyor ve arama kutusu dolu bir
/// mavi bloğa dönüşüyordu. Aynı şey seçili menü öğesinde ve kart
/// kenarlıklarında da oluyordu.
#[test]
fn accent_aliases_keep_their_alpha() {
    let parsed = parse(TINTED_THEME);
    let css = parsed.emit_css();

    for (name, alpha) in [
        ("--bg-control", "4%"),
        ("--bg-menu-selected", "12%"),
        ("--border-card", "10%"),
    ] {
        let line = css
            .lines()
            .find(|l| l.trim_start().starts_with(&format!("{name}:")) && l.contains("hsl"))
            .unwrap_or_else(|| panic!("`{name}` vurgu ailesine alınmamış:\n{css}"));
        assert!(
            line.contains("hsla(") && line.contains(alpha),
            "`{name}` alfasını kaybetmiş: {line}"
        );
    }
}

/// Gri ve beyaz tonlar vurgu ailesine ALINMAMALI.
///
/// Ölçüt ham doygunlukken (%12) `#e8eaf2` (beyaz metin, %27.8 doygunluk) ve
/// `#4a4f68` (gri, %16.9) eşiği geçiyordu. Sonuç: kullanıcı vurgu rengini
/// oynattığında temanın metin renkleri de kayıyordu.
#[test]
fn greys_and_whites_are_not_accent_family() {
    let parsed = parse(TINTED_THEME);
    let aliases: Vec<&str> = parsed
        .accent_aliases
        .iter()
        .map(|a| a.name.as_str())
        .collect();

    for name in ["--mn-white", "--mn-muted", "--mn-sub1", "--text-primary"] {
        assert!(
            !aliases.contains(&name),
            "gri/beyaz ton vurguya bağlanmış: {name} ({aliases:?})"
        );
    }
    // Gerçek vurgu tonları ise bağlanmalı — eşik fazla sıkı olmamalı.
    for name in ["--accent-primary", "--accent-secondary", "--accent-tertiary"] {
        assert!(
            aliases.contains(&name),
            "vurgu tonu bağlanmamış: {name} ({aliases:?})"
        );
    }
}

/// Gövdesi bloğun ALTINDA kalmış, eski bir sürümden çıkma dosya.
///
/// Kullanıcının iki dosyası bunu ortaya çıkardı: aynı editörden çıkmış iki
/// `.css`ten biri (tamamı blok içi) sorunsuz açılıyor, diğeri (gövdesi bloğun
/// altında) bozuk geliyordu. İkincisi, ithal temayı `raw_css`e yazan eski bir
/// sürümle üretilmişti.
const LEGACY_LAYOUT: &str = r#"
@import url("https://example.com/font.css");

/* <oa:tokens> */
:root:root,
:root:root.fds-theme-dark {
	--fds-accent-base: 217, 87%, 76%;
	--fds-solid-background-base: #111118;
}
:root:root:not(#oa-theme) .anime-card,
:root:root:not(#oa-theme) .slider-card,
:root:root:not(#oa-theme) .grid-view-item {
	border-radius: 10px !important;
}
/* </oa:tokens> */

.calendar-card {
	background: #16161f !important;
}

.comment {
	border-radius: 10px !important;
}
"#;

/// İçe aktarmada bloğun ALTINDAKİ gövde de modele girmeli.
///
/// `raw_css` kullanıcının kendi kaçış kapısı: tek parça saklanıyor, yönetilen
/// bloktan SONRA yazılıyor ve kontrolleri eziyor. Başka birinin teması oraya
/// düştüğünde dosya düzenlenemez hâle geliyordu — kontroller hiçbir şeye
/// dokunmuyor, kurallar tek tek ele alınamıyordu.
#[test]
fn importing_folds_content_below_the_marker_into_the_model() {
    let doc = parse(LEGACY_LAYOUT);

    assert!(
        doc.raw_css.trim().is_empty(),
        "bloğun altındaki gövde `raw_css`te kalmış: {:?}",
        doc.raw_css
    );

    let selectors: Vec<&str> = doc
        .imported_rules
        .iter()
        .map(|rule| rule.selector.as_str())
        .collect();
    assert!(
        selectors.contains(&".calendar-card"),
        "blok altındaki kural modele girmemiş: {selectors:?}"
    );

    // Blok İÇİ bilgi de kaybolmamalı: kontroller geri okunmalı.
    assert_eq!(doc.control_corner_radius, Some(10.0), "yuvarlaklık okunmadı");
    assert!(
        doc.token_overrides.contains_key("--fds-solid-background-base"),
        "blok içi token kontrollere bağlanmamış: {:?}",
        doc.token_overrides.keys().collect::<Vec<_>>()
    );
    assert!(!doc.imports.is_empty(), "@import kaybolmuş");

    // Ve tek blok üretilmeli.
    let css = doc.emit_css();
    assert_eq!(css.matches(crate::theme::models::TOKENS_OPEN).count(), 1);
}

/// Yönetilen bloktaki kurallar, özgüllük öneki yüzünden kontrollerden
/// KOPMAMALI.
///
/// Önek bizim kendi ürettiğimiz bir ek. Sökülmediğinde, bu editörden çıkmış
/// bir dosya yeniden açıldığında blok içi kurallar tanınan seçicilerle
/// eşleşmiyor ve kontrollere geri bağlanamıyorlardı.
#[test]
fn boosted_rules_bind_back_to_controls_on_import() {
    let doc = crate::theme::parse::parse_foreign_css(
        LEGACY_LAYOUT,
        &[".anime-card, .slider-card, .grid-view-item".to_string()],
        &ThemeDoc::default(),
    );

    assert!(
        doc.rule_overrides
            .contains_key(".anime-card, .slider-card, .grid-view-item"),
        "önekli kural kontrole bağlanmamış: {:?}",
        doc.rule_overrides.keys().collect::<Vec<_>>()
    );
}

/// Kendi işaretleyici bloğumuzu taşıyan bir dosya İKİNCİ blok üretmemeli.
///
/// Bu editörden çıkmış bir `.css` yeniden içe aktarıldığında blok "temanın bir
/// kuralı" sayılıyordu: çıktıda iki `<oa:tokens>` bloğu oluşuyor, eskisi her
/// yeniden üretimde taşınıyor ve içindeki token'lar kontrollere bağlanmadığı
/// için düzenlenemiyordu.
#[test]
fn reimporting_our_own_output_does_not_duplicate_the_block() {
    let once = parse(TINTED_THEME).emit_css();
    assert_eq!(once.matches(crate::theme::models::TOKENS_OPEN).count(), 1);

    let twice = parse(&once).emit_css();
    assert_eq!(
        twice.matches(crate::theme::models::TOKENS_OPEN).count(),
        1,
        "ikinci içe aktarmada blok çoğaldı:\n{twice}"
    );
    assert_eq!(
        twice.matches(crate::theme::models::TOKENS_CLOSE).count(),
        1,
        "kapanış işaretleyicisi çoğaldı:\n{twice}"
    );
}

// --- Yuvarlaklık kaydırıcısı temaya ulaşıyor mu ------------------------------

/// Yuvarlaklığını KENDİ değişkenlerinden veren tema.
///
/// Desen `ornek/REzero v10.2.css`ten birebir alındı: tema kendi
/// `--ayar-kose-*` adlarını tanımlıyor, `--fds-*-corner-radius`'u onlardan
/// besliyor ve bütün kurallarında `var(--ayar-kose-…) !important` kullanıyor.
/// Yani hiçbir kuralı bizim yazdığımız token'ı okumuyor.
const RADIUS_THEME: &str = r#"
:root {
    --ayar-kose-genel: 20px;
    --ayar-kose-kart: 12px;
    --ayar-kose-tam-daire: 50%;
    --ayar-kose-banner: 0 0 12px 12px;
    --fds-control-corner-radius: var(--ayar-kose-genel) !important;
}

.anime-card, .slider-card, .grid-view-item {
    border-radius: var(--ayar-kose-kart) !important;
}

.comment {
    border-radius: var(--ayar-kose-genel) !important;
}

.avatar {
    border-radius: var(--ayar-kose-tam-daire) !important;
}

#banner {
    border-radius: var(--ayar-kose-banner) !important;
}
"#;

/// Kaydırıcı, temanın kendi yuvarlaklık değişkenlerini de yeniden yazmalı.
///
/// Kullanıcının bildirdiği "köşe yumuşatma asla çalışmıyor" hatası: kaydırıcı
/// yalnızca `--fds-control-corner-radius` yazıyordu, temanın hiçbir kuralı onu
/// okumuyordu, dolayısıyla ekranda hiçbir şey değişmiyordu.
#[test]
fn radius_slider_reaches_the_themes_own_variables() {
    let mut doc = parse(RADIUS_THEME);
    assert_eq!(doc.control_corner_radius, Some(20.0), "taban yuvarlaklık okunmalı");

    let names: Vec<&str> = doc.radius_aliases.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"--ayar-kose-genel"), "aile eksik: {names:?}");
    assert!(names.contains(&"--ayar-kose-kart"), "aile eksik: {names:?}");

    // Kullanıcı kaydırıcıyı 20 -> 4 çekiyor.
    doc.control_corner_radius = Some(4.0);
    let css = doc.emit_css();

    assert!(
        css.contains("--ayar-kose-genel: 4px;"),
        "temanın genel yuvarlaklığı kaydırıcıyı izlemiyor:\n{css}"
    );
    // Aile ilişkisi korunuyor: kart temada 8px daha küçüktü, öyle kalmalı.
    assert!(
        css.contains("--ayar-kose-kart: 0px;") || css.contains("--ayar-kose-kart: -"),
        "kart yuvarlaklığı sapmasını korumuyor:\n{css}"
    );
}

/// Uzunluk OLMAYAN değerler kaydırıcıya bağlanmamalı.
///
/// `50%` bir daire (avatar), `0 0 12px 12px` dört köşesi ayrı bir banner.
/// İkisi de tek bir px değerine indirgenemez; bağlansalardı avatar kare olur,
/// banner'ın köşe düzeni bozulurdu.
#[test]
fn non_length_radii_are_left_alone() {
    let doc = parse(RADIUS_THEME);
    let names: Vec<&str> = doc.radius_aliases.iter().map(|a| a.name.as_str()).collect();

    assert!(!names.contains(&"--ayar-kose-tam-daire"), "yüzde bağlanmış: {names:?}");
    assert!(!names.contains(&"--ayar-kose-banner"), "çok değerli yarıçap bağlanmış: {names:?}");

    // Ve olduğu gibi korunuyorlar.
    let css = doc.emit_css();
    assert!(css.contains("50%"), "daire değeri kaybolmuş:\n{css}");
}

// --- Kontroller temanın kendi değişkenlerine ulaşıyor mu ---------------------

/// Site token'larını kendi adlarıyla GÖLGELEYEN tema.
///
/// Desen kullanıcının Midnight temasından birebir: her site token'ının yanında
/// aynı değeri taşıyan bir tema değişkeni var ve kurallar site token'ını değil
/// o adı okuyor.
const SHADOWED_THEME: &str = r#"
:root {
    --bg-page: #111118;
    --text-primary: #e8eaf2;
    --font-primary: 'Inter', sans-serif;
    --kullanilmayan: #111118;
    --fds-solid-background-base: #111118;
    --fds-text-primary: #e8eaf2;
    --fds-font-family-text: 'Inter', sans-serif;
}

html {
    background-color: var(--bg-page);
}

body {
    color: var(--text-primary);
    font-family: var(--font-primary);
}
"#;

/// Bir kontrol değiştiğinde, onu gölgeleyen tema değişkeni de değişmeli.
///
/// Kullanıcının "çoğu ayar çalışmıyor" dediği durumun genel hâli: kontrol
/// `--fds-solid-background-base`'i yazıyor, sayfa zeminini ise `--bg-page`
/// boyuyor. İkisi bağlanmazsa kontrol ekranda hiçbir şeye dokunmuyor.
#[test]
fn changing_a_control_reaches_the_shadowing_variable() {
    let mut doc = parse(SHADOWED_THEME);

    let pairs: Vec<(&str, &str)> = doc
        .token_aliases
        .iter()
        .map(|a| (a.name.as_str(), a.source.as_str()))
        .collect();
    assert!(
        pairs.contains(&("--bg-page", "--fds-solid-background-base")),
        "zemin değişkeni bağlanmamış: {pairs:?}"
    );
    assert!(
        pairs.contains(&("--text-primary", "--fds-text-primary")),
        "metin değişkeni bağlanmamış: {pairs:?}"
    );
    assert!(
        pairs.contains(&("--font-primary", "--fds-font-family-text")),
        "yazı tipi değişkeni bağlanmamış: {pairs:?}"
    );

    // Tema hiç kullanmadığı bir adı yeniden yazmanın etkisi olmaz; blok
    // gereksiz yere şişmesin diye alınmıyor.
    assert!(
        !pairs.iter().any(|(name, _)| *name == "--kullanilmayan"),
        "kullanılmayan değişken bağlanmış: {pairs:?}"
    );

    // Kullanıcı zemin rengini değiştiriyor.
    doc.token_overrides
        .insert("--fds-solid-background-base".into(), "#ff0000".into());
    let css = doc.emit_css();

    assert!(
        css.contains("--bg-page: #ff0000;"),
        "kontrol temanın kendi değişkenine ulaşmıyor:\n{css}"
    );
    // Dokunulmayanlar yerinde kalmalı.
    assert!(
        !css.contains("--text-primary: #ff0000"),
        "ilgisiz değişken de değişmiş:\n{css}"
    );
}

/// Hiçbir kontrol oynatılmadıysa gölgelenen değişkenler yeniden yazılmamalı.
///
/// Yazılsaydı tema açılır açılmaz kendi değişkenlerinin üstüne aynı değerin
/// küresel bir kopyası binerdi: görünürde bir şey değişmez ama temanın kipe
/// bağlı tanımları o kopyaya ezilir ve blok gereksiz yere şişerdi.
#[test]
fn untouched_controls_do_not_rewrite_theme_variables() {
    let doc = parse(SHADOWED_THEME);
    let css = doc.emit_css();

    // Değer temanın verdiği gibi kalmalı; ikinci bir küresel kopya olmamalı.
    assert_eq!(
        css.matches("--bg-page:").count(),
        1,
        "dokunulmadan ikinci bir kopya yazılmış:\n{css}"
    );
}

/// Negatife düşen yuvarlaklık sıfırlanmalı.
///
/// CSS negatif yarıçapı geçersiz sayıp bildirimin TAMAMINI atıyor; kaydırıcı
/// dibe çekildiğinde o kural sessizce yok olurdu.
#[test]
fn radius_never_goes_negative() {
    let mut doc = parse(RADIUS_THEME);
    doc.control_corner_radius = Some(0.0);
    let css = doc.emit_css();

    assert!(!css.contains("--ayar-kose-kart: -"), "negatif yarıçap yazılmış:\n{css}");
    assert!(css.contains("--ayar-kose-kart: 0px;"), "sıfırlanmamış:\n{css}");
}

