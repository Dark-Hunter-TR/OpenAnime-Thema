use std::collections::BTreeMap;

use crate::theme::models::{ImportedRule, ThemeDoc, TOKENS_OPEN, TOKENS_CLOSE};
use crate::theme::parse::{
    normalize_selector, split_top_level, DERIVED_TOKEN_NAMES, STRUCTURAL_TOKENS,
};
use crate::theme::color::{DEFAULT_ACCENT, RAMP_NAMES, derive_ramp, fmt_triplet, num};

impl ThemeDoc {
    fn accent_is_default(&self) -> bool {
        self.accent
            .iter()
            .zip(DEFAULT_ACCENT.iter())
            .all(|(a, b)| (a - b).abs() < 0.001)
    }

    /// Kullanıcı vurgu kaydırıcısına GERÇEKTEN dokundu mu?
    ///
    /// İçe aktarılmış bir temada `accent`, temanın kendi vurgusudur —
    /// kullanıcının seçimi değil. Ölçüt "kütüphane varsayılanından farklı mı"
    /// olduğunda, tema açılır açılmaz "kullanıcı vurguyu değiştirdi" sanılıyor
    /// ve aşağıdaki seçili-öğe kuralı yazılıyordu; o kural da temanın kendi
    /// ikon rengiyle çakışıyordu (gerekçe: `ThemeDoc::imported_accent`).
    fn accent_was_changed(&self) -> bool {
        match self.imported_accent {
            // Karşılaştırma YAZILDIĞI biçim üzerinden yapılıyor, ham sayılar
            // üzerinden değil. Vurgu CSS'e yuvarlanarak yazılıyor
            // (`fmt_triplet`); kod editörü turunda geri okunan değer ham
            // sayıyla birebir tutmuyor ve sıkı bir karşılaştırma "kullanıcı
            // vurguyu değiştirdi" sanıp seçili-öğe kuralını yeniden
            // yazıyordu.
            Some(imported) => fmt_triplet(self.accent) != fmt_triplet(imported),
            None => !self.accent_is_default(),
        }
    }
}

/// Bir vurgu token'ının, verilen tabandan TÜRETİLEN değeri.
///
/// Tek kaynak olması şart: `parse` tarafı, bir dosyadaki açık değerin
/// türetilenden farklı olup olmadığına bakarak onu koruyup korumayacağına
/// karar veriyor (bkz. `parse::explicit_derived_overrides`). İki taraf ayrı
/// listeler tutsaydı, buradaki bir değişiklik sessizce "her tema bu token'ı
/// elle yazmış" gibi görünür ve blok gereksiz tekrarlarla şişerdi.
pub fn derived_token_value(name: &str, accent: crate::theme::color::Hsl) -> Option<String> {
    if let Some(idx) = RAMP_NAMES.iter().position(|n| *n == name) {
        return Some(fmt_triplet(derive_ramp(accent)[idx]));
    }

    // Rampa DIŞINDAKİ vurgu token'ları (`accent-default`, `accent-secondary`,
    // `accent-tertiary`, `accent-text-*`) BİLEREK yazılmıyor.
    //
    // Onları kütüphanenin kendisi rampadan türetiyor ve türetme KİPE BAĞLI.
    // Sitenin canlı CSS'inden okundu:
    //
    //     .fds-theme-light { --fds-accent-default: hsl(var(--fds-accent-dark-1)) }
    //     .fds-theme-dark  { --fds-accent-default: hsla(var(--fds-accent-light-2)) }
    //
    // Burada bir zamanlar hepsi `hsl(var(--fds-accent-base))` olarak
    // yazılıyordu. İki sorun birden vardı: değer yanlıştı (taban, doğru
    // basamak değil) ve yönetilen blok her iki kipi birden kapsadığı için
    // kütüphanenin DOĞRU kipe bağlı kurallarını da eziyordu. Sonucu koyu
    // kipte gözle görülür bir sapmaydı — bu token'dan türeyen her şey (rozet
    // gradyanının ucu, banner çerçevesi, oynatıcı rayı, bağlantı renkleri)
    // açık mavi olması gerekirken koyu mavi çıkıyordu.
    //
    // Yalnızca yedi rampa basamağını yazıp gerisini kütüphaneye bırakmak hem
    // doğru hem daha az kod: kip mantığını burada ikinci kez kurmuyoruz.
    None
}

/// Yönetilen token bloğunun seçicisi.
///
/// İçe aktarılan bir tema varsa özgüllük BİLEREK yükseltiliyor. Sebebi ölçülmüş
/// bir davranış: gelişmiş temalar renk paletlerini `html.fds-theme-dark`
/// altında tanımlıyor — özgüllüğü (0,1,1). Bizim eski seçicimiz
/// `:root, .fds-theme-light, .fds-theme-dark` ise (0,1,0), yani blok temadan
/// SONRA gelse bile kaybediyordu. Kullanıcının gördüğü "renkler dışında hiçbir
/// şeyi değiştiremiyorum" bu yüzdendi: vurgu çalışıyordu (temalar
/// `--fds-accent-base`'i genelde tanımlamıyor), ama yüzey, metin, kart gibi
/// temanın kendi tanımladığı her token bizi eziyordu.
///
/// `:root:root` (0,2,0) ikinci bileşende üstün olduğu için (0,1,1)'i ve
/// (0,1,2)'yi yeniyor.
///
/// İçe aktarılan tema YOKSA eski, sade seçici kullanılıyor. Bu bilinçli:
/// yükseltme kullanıcının "Ham CSS" kaçış kapısını da zayıflatıyor (oraya
/// yazılan sade bir `:root` kuralı artık bloğu ezemezdi) ve o bedeli yalnızca
/// gerçekten gerektiği yerde ödüyoruz.
fn token_block_selector(doc: &ThemeDoc) -> &'static str {
    if !doc.has_imported() {
        ":root,\n.fds-theme-light,\n.fds-theme-dark {\n"
    } else {
        ":root:root,\n:root:root.fds-theme-light,\n:root:root.fds-theme-dark {\n"
    }
}

pub fn emit_css(doc: &ThemeDoc) -> String {
    let mut root: Vec<String> = Vec::new();

    if !doc.accent_is_default() {
        for name in crate::theme::parse::DERIVED_TOKEN_NAMES {
            if let Some(value) = derived_token_value(name, doc.accent) {
                root.push(format!("\t--fds-{name}: {value};"));
            }
        }
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

    // Temanın KENDİ yuvarlaklık değişkenlerini kaydırıcıya bağla
    // (gerekçe: `ThemeDoc::radius_aliases`). Bunlar yazılmadığında kaydırıcı
    // yalnızca `--fds-*-corner-radius`'u değiştiriyor, temanın hiçbir kuralı
    // o token'ı okumadığı için ekranda hiçbir şey olmuyordu.
    //
    // Sapma korunuyor: tema kartlara 12px, banner'a 4px veriyorsa fark aynı
    // kalıyor. Negatife düşen değerler sıfırlanıyor — CSS negatif yarıçapı
    // geçersiz sayıp bildirimin tamamını atıyor.
    if let Some(base) = doc.control_corner_radius {
        for alias in &doc.radius_aliases {
            let value = (base + alias.delta).max(0.0);
            root.push(format!("\t{}: {}px;", alias.name.trim(), num(value)));
        }
    }

    // Temanın, site token'larını gölgeleyen kendi değişkenleri
    // (gerekçe: `ThemeDoc::token_aliases`). Kontrol site token'ını yazıyor,
    // temanın kuralları kendi adını okuyor; ikisi bağlanmazsa kontrol
    // ekranda hiçbir şeye dokunmuyor.
    //
    // Yalnızca kullanıcı GERÇEKTEN değiştirdiyse yazılıyor. Aksi hâlde tema
    // açılır açılmaz kendi değişkenlerinin üstüne aynı değerin bir kopyası
    // binerdi: görünürde bir şey değişmez ama blok gereksiz yere şişer ve
    // temanın kipe bağlı tanımları küresel bir kopyaya ezilirdi.
    for alias in &doc.token_aliases {
        let Some(current) = doc.token_overrides.get(&alias.source) else {
            continue;
        };
        let unchanged = doc
            .imported_tokens
            .get(&alias.source)
            .is_some_and(|original| original.trim() == current.trim());
        if unchanged {
            continue;
        }
        root.push(format!("\t{}: {};", alias.name.trim(), current.trim()));
    }

    // Temanın kendi vurgu değişkenlerini, kaydırıcının yeni değerine göre
    // yeniden yaz (gerekçe: `ThemeDoc::accent_aliases`). Her biri kendi
    // sapmasıyla hesaplandığı için aile ilişkisi korunuyor; kaydırıcı içe
    // aktarma anındaki yere geri getirilirse değerler de aslına dönüyor.
    for alias in &doc.accent_aliases {
        let shifted = [
            (doc.accent[0] + alias.delta[0]).rem_euclid(360.0),
            (doc.accent[1] + alias.delta[1]).clamp(0.0, 100.0),
            (doc.accent[2] + alias.delta[2]).clamp(0.0, 100.0),
        ];
        // Alfa olduğu gibi geri yazılıyor. Düşürüldüğünde temanın ince
        // tint'leri (ör. `rgba(141, 180, 247, 0.04)` bir girdi zemini) opak
        // vurgu bloklarına dönüşüyordu (gerekçe: `AccentAlias::alpha`).
        let value = if alias.alpha >= 1.0 {
            format!("hsl({})", fmt_triplet(shifted))
        } else {
            format!("hsla({}, {}%)", fmt_triplet(shifted), num(alias.alpha * 100.0))
        };
        root.push(format!("\t{}: {value};", alias.name.trim()));
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

    // İçe aktarılan tema TABAN: yönetilen bloktan önce yazılıyor ki kullanıcı
    // bir kontrolü oynattığında editörün çıktısı onun üstüne binsin
    // (gerekçe: `ThemeDoc::imported_rules`).
    write_imported(&mut css, doc);

    css.push_str(TOKENS_OPEN);
    css.push_str(
        "\n/* Bu blok görsel kontrollerden üretilir; elle düzenlerseniz\n   \
         kontroller de güncellenir. Blok ÜSTÜNDEKİ CSS taban alınır, blok\n   \
         ALTINDAKİ CSS ise bu bloğu ezer; ikisi de korunur. */\n",
    );
    if !root.is_empty() {
        css.push_str(token_block_selector(doc));
        css.push_str(&root.join("\n"));
        css.push_str("\n}\n");
    }
    // Yalnızca kullanıcı vurguyu GERÇEKTEN değiştirdiyse
    // (gerekçe: `accent_was_changed`). İçe aktarılmış bir temada bu kural
    // istenmeden yazıldığında temanın kendi seçili-ikon rengini eziyordu.
    if doc.accent_was_changed() {
        css.push_str(&control_selector(
            doc,
            ".list-item.selected, .list-item.selected *, .list-item.selected svg, .list-item.selected path",
        ));
        css.push_str(
            " {\n\
            \tcolor: var(--fds-accent-default) !important;\n\
            \tfill: currentColor !important;\n\
            }\n",
        );
    }
    for (selector, body) in &doc.rule_overrides {
        css.push_str(&control_selector(doc, selector));
        css.push_str(" {\n\t");
        css.push_str(&control_body(doc, body));
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


// --- İçe aktarılan kurallar --------------------------------------------------

/// Bir bildirimin özellik adı. Yorum ya da boş parça ise `None`.
fn declaration_property(decl: &str) -> Option<String> {
    let cleaned = strip_comments(decl);
    // İlk `:` özellik sınırı — özellik adları iki nokta içeremez, dolayısıyla
    // `url(https://…)` gibi değerler bunu bozmuyor.
    let (key, value) = cleaned.split_once(':')?;
    let (key, value) = (key.trim(), value.trim());
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some(key.to_string())
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

/// Gövde bu özelliği tanımlıyor mu.
fn declares(body: &str, prop: &str) -> bool {
    split_top_level(body)
        .into_iter()
        .filter_map(declaration_property)
        .any(|p| p.eq_ignore_ascii_case(prop))
}

/// Bu bildirimin sahibi artık kontroller mi?
///
/// Bu dosyadaki en önemli karar: çakışan bildirim ithal kuraldan ÇIKARILIYOR,
/// üstüne yazılmıyor. Sıraya ya da özgüllüğe güvenmek yetmiyordu — gerçek
/// temalar bildirimlerini `!important` ile yazıyor (örnek temada 255 tane) ve
/// `!important` bir bildirimi ne sonra gelmek ne de daha özgül olmak yener.
/// Kullanıcı kaydırıcıyı oynatıyor, ekranda hiçbir şey değişmiyordu.
///
/// Ölçüt "kontrolde karşılığı var mı" DEĞİL, "kullanıcı gerçekten değiştirdi
/// mi". Aksi hâlde tema açılır açılmaz kendi kipe bağlı renklerini
/// kaybederdi: `.fds-theme-dark` altında yazılmış koyu renkler, kullanıcı
/// hiçbir şeye dokunmadan silinmiş olurdu.
fn control_owns(doc: &ThemeDoc, override_body: Option<&String>, prop: &str) -> bool {
    let prop = prop.trim();

    // Tanınan bir seçici: kontrol o kuralın gövdesini zaten yeniden yazıyor.
    if let Some(body) = override_body {
        if declares(body, prop) {
            return true;
        }
    }

    if !prop.starts_with("--") {
        return false;
    }

    // Yapısal token'ların `ThemeDoc`'ta kendi alanı var; bir değer varsa
    // yönetilen blok onu HER ZAMAN yazıyor, dolayısıyla sahibi kontroller.
    if STRUCTURAL_TOKENS.contains(&prop) {
        return match prop {
            "--fds-control-corner-radius" => doc.control_corner_radius.is_some(),
            _ => doc.overlay_corner_radius.is_some(),
        };
    }

    // Vurgu rampası yalnızca vurgu varsayılandan farklıyken yazılıyor; o
    // durumda basamakların sahibi kaydırıcı.
    if !doc.accent_is_default()
        && DERIVED_TOKEN_NAMES
            .iter()
            .any(|name| prop == format!("--fds-{name}"))
    {
        return true;
    }

    // Temanın KENDİ vurgu değişkenleri de kaydırıcıya bağlandı
    // (gerekçe: `ThemeDoc::accent_aliases`).
    if doc.accent_aliases.iter().any(|a| a.name.trim() == prop) {
        return true;
    }

    // Yuvarlaklık ailesi için aynısı (gerekçe: `ThemeDoc::radius_aliases`).
    // Yalnızca kaydırıcının bir değeri varken: değeri yokken yönetilen blok
    // bu adları hiç yazmıyor ve temanın kendi tanımını silmek onu ortadan
    // kaldırmak olurdu.
    if doc.control_corner_radius.is_some()
        && doc.radius_aliases.iter().any(|a| a.name.trim() == prop)
    {
        return true;
    }

    // Gölgelenen değişkenler (gerekçe: `ThemeDoc::token_aliases`). Yine
    // yalnızca kaynağı gerçekten değiştiyse: değişmediyse yönetilen blok o adı
    // hiç yazmıyor, silmek temanın değerini yok etmek olurdu.
    if doc.token_aliases.iter().any(|alias| {
        alias.name.trim() == prop
            && doc.token_overrides.get(&alias.source).is_some_and(|current| {
                doc.imported_tokens
                    .get(&alias.source)
                    .is_none_or(|original| original.trim() != current.trim())
            })
    }) {
        return true;
    }

    // Karşılaştırma tabanı: önce içe aktarma anındaki yazılan değer, o yoksa
    // yalnızca kontrolleri beslemek için okunan değer (`seed_tokens`).
    //
    // İkincisi şart. Kontroller kipe bağlı bloklardan da besleniyor ve ön yüz
    // bir kontrol değiştiğinde haritanın TAMAMINI geri yazıyor
    // (`+page.svelte` -> `tokenMap`). Yalnızca "haritada var mı" diye
    // sorulsaydı, kullanıcı tek bir kaydırıcıyı oynattığında tohumlanmış
    // bütün değerler "değiştirildi" sayılır ve temanın `.fds-theme-dark`
    // renkleri toptan silinirdi.
    let original = doc
        .imported_tokens
        .get(prop)
        .or_else(|| doc.seed_tokens.get(prop));

    match (doc.token_overrides.get(prop), original) {
        // İçe aktarma anındaki değerden ayrıldıysa kullanıcı değiştirmiştir.
        (Some(current), Some(base)) => current.trim() != base.trim(),
        // Temada hiç yoktu, şimdi var: kontrolden eklenmiş.
        (Some(_), None) => true,
        _ => false,
    }
}

/// İthal kuralın, kontrollere devredilmemiş bildirimleri. Hiçbiri kalmazsa
/// `None` — kural bütünüyle atlanır, boş bir seçici yazılmaz.
fn imported_body(
    doc: &ThemeDoc,
    rule: &ImportedRule,
    overrides: &BTreeMap<String, String>,
) -> Option<String> {
    // `@keyframes` / `@font-face` gövdesi bildirim değil iç içe kural taşır;
    // bildirim bazında ele alınamaz ve kontrollerle de çakışmıyor.
    if rule.selector.trim_start().starts_with('@') {
        let body = rule.body.trim();
        return (!body.is_empty()).then(|| body.to_string());
    }

    // Koşullu bir bloğun (`@media`, `@supports`) içindeki kural, kontrolün
    // yazdığından DAHA DAR bir bağlamda geçerli — dolayısıyla kontrol onun
    // sahibi olamaz. `@media (max-width: 768px)` altındaki bir bildirimi
    // kontrolün küresel karşılığı yüzünden silmek, temanın mobil kırılımını
    // kullanıcı hiç istemeden yok etmek olurdu.
    if !rule.at.is_empty() {
        let body = rule.body.trim();
        return (!body.is_empty()).then(|| body.to_string());
    }

    let override_body = overrides.get(&normalize_selector(&rule.selector));

    let mut kept: Vec<String> = Vec::new();
    for decl in split_top_level(&rule.body) {
        let trimmed = decl.trim();
        if trimmed.is_empty() {
            continue;
        }
        match declaration_property(trimmed) {
            Some(prop) => {
                if !control_owns(doc, override_body, &prop) {
                    kept.push(format!("{trimmed};"));
                }
            }
            // Bildirim değil (ör. bildirimler arasına düşmüş bir yorum):
            // olduğu gibi korunuyor.
            None => kept.push(trimmed.to_string()),
        }
    }

    // Yalnızca yorum kaldıysa kuralın anlamı kalmamış — atlansın.
    if kept.iter().all(|line| declaration_property(line).is_none()) {
        return None;
    }
    Some(kept.join("\n"))
}

/// İçe aktarılan kuralları belge sırasıyla yazar.
///
/// Sıra korunuyor çünkü CSS'te eşit özgüllükte sonra gelen kazanır — belge
/// sırası anlamın bir parçası. Ardışık kurallar aynı at-kuralı zincirini
/// paylaşıyorsa tek bir `@media` bloğunda toplanıyor; aksi hâlde her kural
/// kendi sarmalayıcısını açar ve dosya okunmaz hâle gelirdi.
fn write_imported(css: &mut String, doc: &ThemeDoc) {
    if doc.imported_rules.is_empty() {
        return;
    }

    // Seçici -> kontrolün yazdığı gövde. Döngüden önce bir kez kuruluyor:
    // her kaydırıcı hareketinde yeniden üretim yapılıyor ve kural sayısı üç
    // haneli olabiliyor.
    let overrides: BTreeMap<String, String> = doc
        .rule_overrides
        .iter()
        .map(|(selector, body)| (normalize_selector(selector), body.clone()))
        .collect();

    let mut open: Vec<String> = Vec::new();

    for rule in &doc.imported_rules {
        let Some(body) = imported_body(doc, rule, &overrides) else {
            continue;
        };

        // Açık zincirin ortak önekini koru; farkı kapatıp yenisini aç.
        let common = open
            .iter()
            .zip(rule.at.iter())
            .take_while(|(a, b)| a == b)
            .count();
        while open.len() > common {
            open.pop();
            css.push_str(&"\t".repeat(open.len()));
            css.push_str("}\n");
        }
        for at in &rule.at[common..] {
            css.push_str(&"\t".repeat(open.len()));
            css.push_str(at.trim());
            css.push_str(" {\n");
            open.push(at.clone());
        }

        let indent = "\t".repeat(open.len());
        if let Some(note) = &rule.note {
            for line in note.lines() {
                css.push_str(&indent);
                css.push_str(line.trim());
                css.push('\n');
            }
        }
        css.push_str(&indent);
        css.push_str(rule.selector.trim());
        css.push_str(" {\n");
        for line in body.lines() {
            css.push_str(&indent);
            css.push('\t');
            css.push_str(line.trim_start());
            css.push('\n');
        }
        css.push_str(&indent);
        css.push_str("}\n");
    }

    while !open.is_empty() {
        open.pop();
        css.push_str(&"\t".repeat(open.len()));
        css.push_str("}\n");
    }

    css.push('\n');
}


// --- Kontrol kurallarının ağırlığı -------------------------------------------

/// Kontrol kurallarına eklenen özgüllük öneki.
///
/// `:root` = `html`, yani her öğe onun altında; iki kez yazmak seçiciyi
/// daraltmadan sınıf sütununu 2 artırıyor. Aynı hile yönetilen token bloğu
/// için de kullanılıyor (bkz. `token_block_selector`).
///
/// `:not(#oa-theme)` ise ID sütunu için. Ölçülen bir ihtiyaç: bir tema
/// `.list-item:not(:has(#lottie-player)) svg path` yazıyor ve `:has()`
/// içindeki id, özgüllüğü ID düzeyine çıkarıyor — ID sütunu sınıf sütununu
/// her zaman yendiği için kaç `:root` eklenirse eklensin o kural
/// yenilemiyordu. `:not()` argümanının özgüllüğünü alır (1,0,0) ama
/// eşleşmeyi daraltmaz: böyle bir id hiçbir sayfada yok, dolayısıyla
/// `html` dâhil her öğe koşulu sağlıyor.
///
/// Virgül İÇERMEMESİ şart: `:is(#x, :root)` aynı işi görürdü ama seçiciyi
/// virgülden parçalayan taraflar (`control_selector`,
/// `strip_specificity_boost`) onu ortadan bölerdi.
///
/// `parse_css` bunu geri söküyor (`strip_specificity_boost`); yoksa her turda
/// bir tane daha eklenir ve seçici sonsuza kadar uzardı.
pub(crate) const SPECIFICITY_BOOST: &str = ":root:root:not(#oa-theme)";

/// Kontrol kuralı, ithal temanın karşısında kazanacak biçimde yazılmalı mı?
///
/// Ölçülmüş bir davranış: gerçek bir temanın kontrollerle çakışan 25 bildirimi
/// vardı ve HİÇBİRİ kontrolden değiştirilemiyordu. Sebep iki katmanlıydı —
/// tema bildirimlerini `!important` ile yazıyor (o dosyada 255 tane) ve
/// kendi seçicileri bizimkinden daha özgül (`a.anime-card` ↔ `.anime-card`,
/// `div.comment.svelte-1snun1g` ↔ `.comment`). `!important` bir bildirimi
/// ne sonra gelmek ne daha özgül olmak yener; iki taraf da `!important`
/// olduğunda ise özgüllük karar verir. Bu yüzden ikisi birden gerekiyor.
///
/// Yalnızca ithal tema VARKEN uygulanıyor. Bedeli bilinçli: `!important`,
/// kullanıcının "Ham CSS" kutusunun kontrolleri sade bir kuralla ezmesini de
/// engelliyor (orada da `!important` gerekiyor). Tema yokken böyle bir rakip
/// olmadığı için o bedel ödenmiyor ve çıktı sade kalıyor.
fn needs_weight(doc: &ThemeDoc) -> bool {
    doc.has_imported()
}

/// Seçiciyi, gerekiyorsa özgüllük önekiyle yazar. Önek her virgül parçasına
/// ayrı ayrı geliyor — `:root:root:root .a, .b` yazmak yalnızca ilk parçayı
/// güçlendirirdi.
fn control_selector(doc: &ThemeDoc, selector: &str) -> String {
    let selector = selector.trim();
    if !needs_weight(doc) {
        return selector.to_string();
    }
    selector
        .split(',')
        .map(|part| format!("{SPECIFICITY_BOOST} {}", part.trim()))
        .collect::<Vec<_>>()
        .join(",\n")
}

/// Gövdeyi, gerekiyorsa her bildirime `!important` ekleyerek yazar.
///
/// Zaten `!important` taşıyan bildirime dokunulmuyor: kontrollerin bir kısmı
/// (avatar, banner, yorumlar…) onu kendisi yazıyor ve iki kez eklemek geçersiz
/// CSS üretirdi.
fn control_body(doc: &ThemeDoc, body: &str) -> String {
    let body = body.trim();
    if !needs_weight(doc) {
        return body.to_string();
    }

    split_top_level(body)
        .into_iter()
        .filter_map(|decl| {
            let trimmed = decl.trim();
            if trimmed.is_empty() {
                return None;
            }
            if declaration_property(trimmed).is_none() {
                // Bildirim değil (ör. yorum) — olduğu gibi kalsın.
                return Some(trimmed.to_string());
            }
            if trimmed.to_ascii_lowercase().contains("!important") {
                Some(format!("{trimmed};"))
            } else {
                Some(format!("{trimmed} !important;"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n\t")
}
