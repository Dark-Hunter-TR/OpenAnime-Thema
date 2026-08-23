#[cfg(test)]
mod tests {
    use crate::theme::*;
    use crate::theme::color::*;
    use crate::theme::emit::*;
    use crate::theme::parse::*;
    use crate::theme::models::*;
    use std::collections::BTreeMap;

    #[test]
    fn default_base_produces_library_ramp() {
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
    fn default_document_writes_no_tokens() {
        let css = ThemeDoc::default().emit_css();
        assert!(!css.contains("--fds-"), "beklenmedik token: {css}");
    }

    #[test]
    fn changed_accent_writes_seven_steps() {
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

    /// Kendi ESKİ çıktımız yeniden içe aktarıldığında rampa tazelenmeli.
    ///
    /// Bu editörün eski bir sürümü rampayı sabit ışıklılık ekleriyle
    /// hesaplıyordu ve açık bir vurguda üst basamakları `%100`e (beyaz)
    /// kırpıyordu. Dosya yeniden açıldığında o değerler "tema yazarının elle
    /// seçtiği" sayılıp korunuyor ve düzeltilmiş türetmenin ÜSTÜNE biniyordu:
    /// üretilen blokta İKİ ayrı rampa oluyor, sonra gelen beyaz set kazanıyor
    /// ve seçili kenar çubuğu ikonu beyaz kalmaya devam ediyordu.
    #[test]
    fn stale_ramp_in_our_own_block_does_not_survive_reimport() {
        // Eski sürümün çıktısı: taban açık, üst basamaklar beyaza kırpılmış.
        let stale = format!(
            "{TOKENS_OPEN}\n:root:root {{\n\
             \t--fds-accent-light-3: 202.9, 84.9%, 100%;\n\
             \t--fds-accent-light-2: 210.9, 85.9%, 100%;\n\
             \t--fds-accent-light-1: 216.9, 86.9%, 83.1%;\n\
             \t--fds-accent-base: 217.9, 86.9%, 76.1%;\n\
             \t--fds-accent-dark-1: 220.9, 86.9%, 70.1%;\n\
             }}\n{TOKENS_CLOSE}\n"
        );

        let doc = parse_foreign_css(&stale, &[], &ThemeDoc::default());
        let css = doc.emit_css();

        // Taban korunmalı — vurgu oradan geri okunuyor.
        assert_eq!(doc.accent, [217.9, 86.9, 76.1]);

        // Ama beyaz basamaklar taşınmamalı.
        assert!(
            !css.contains("85.9%, 100%") && !css.contains("84.9%, 100%"),
            "eski beyaz basamaklar korunmuş:\n{css}"
        );

        // Ve her basamak tek kez yazılmalı; iki set çakışmamalı.
        for name in RAMP_NAMES {
            assert_eq!(
                css.matches(&format!("--fds-{name}:")).count(),
                1,
                "`{name}` iki kez yazılmış:\n{css}"
            );
        }
    }

    /// Kullanıcı vurguya DOKUNMADIYSA seçili-öğe kuralı yazılmamalı.
    ///
    /// Kullanıcının bildirdiği hata: içe aktarılan bir temada seçili kenar
    /// çubuğu ikonu beyaz çıkıyordu. Zincir şuydu — içe aktarmada `accent`
    /// temanın kendi vurgusu oluyor, ölçüt "kütüphane varsayılanından farklı
    /// mı" olduğu için "kullanıcı değiştirdi" sanılıyor ve
    /// `.list-item.selected * { color: var(--fds-accent-default) }` kuralı
    /// yazılıyordu. O kural temanın kendi ikon rengini eziyor, koyu kipte
    /// `--fds-accent-default` rampanın `light-2` basamağından türediği için
    /// (ve o basamak beyaza kırpıldığı için) ikon beyaz kalıyordu.
    #[test]
    fn untouched_accent_does_not_repaint_the_selected_item() {
        let theme = "\
:root { --fds-accent-base: 217.9, 86.9%, 76.1%; }
.list-item.selected svg { fill: var(--accent-primary) !important; }
";
        let mut doc = parse_foreign_css(theme, &[], &ThemeDoc::default());
        assert_eq!(doc.imported_accent, Some([217.9, 86.9, 76.1]));

        // Dokunulmadı: kural hiç yazılmamalı.
        assert!(
            !doc.emit_css().contains("var(--fds-accent-default)"),
            "kullanıcı vurguya dokunmadan seçili-öğe kuralı yazılmış:\n{}",
            doc.emit_css()
        );

        // Kaydırıcı oynatıldığında ise yazılmalı.
        doc.accent = [10.0, 90.0, 50.0];
        assert!(doc.emit_css().contains("var(--fds-accent-default)"));
    }

    /// Kendi ÜRETTİĞİMİZ seçili-öğe kuralı yeniden içe aktarmada taşınmamalı.
    ///
    /// Dosya bu editörden çıkmışsa o kuralı zaten içeriyor. İşaretleyiciler
    /// sıyrıldığı için "temanın bir kuralı" sayılıp taşınıyordu: kullanıcı
    /// vurguya hiç dokunmasa bile kural dosyada kalmaya devam ediyor ve
    /// temanın kendi ikon rengini ezmeye devam ediyordu.
    #[test]
    fn generated_accent_rule_does_not_survive_reimport() {
        let exported = format!(
            "{TOKENS_OPEN}\n\
             :root:root {{ --fds-accent-base: 217.9, 86.9%, 76.1%; }}\n\
             .list-item.selected, .list-item.selected * {{\n\
             \tcolor: var(--fds-accent-default) !important;\n\
             \tfill: currentColor !important;\n\
             }}\n{TOKENS_CLOSE}\n\
             .list-item.selected svg {{ fill: var(--accent-primary) !important; }}\n"
        );

        let doc = parse_foreign_css(&exported, &[], &ThemeDoc::default());
        let css = doc.emit_css();

        assert!(
            !css.contains("var(--fds-accent-default)"),
            "eski üretilmiş kural taşınmış:\n{css}"
        );
        // Temanın KENDİ kuralı korunmalı.
        assert!(css.contains("var(--accent-primary)"), "temanın kuralı kaybolmuş:\n{css}");
    }

    /// Temanın vurgu rengi, SİTENİN boyadığı basamakta çıkmalı.
    ///
    /// Kullanıcının bildirdiği hata: içe aktarılan temada seçili menü
    /// göstergesi ve ikon, temanın kendi renginden görünür biçimde AÇIK
    /// çıkıyordu — "renk tutmuyordu".
    ///
    /// Sebep eşlemeydi. Sitenin vurguyla boyadığı her şey
    /// `--fds-accent-default` kullanıyor; ölçüldü:
    ///
    /// ```text
    /// .list-item::before { background-color: var(--fds-accent-default);
    ///                      inline-size: 3px; block-size: 16px }
    /// ```
    ///
    /// Kütüphane o token'ı koyu kipte `light-2`den türetiyor. Temanın rengi
    /// doğrudan TABAN yapıldığında site onu bir basamak açık çiziyordu. Taban
    /// geriye çözülünce ikisi çakışıyor.
    #[test]
    fn theme_accent_lands_on_the_painted_step() {
        let theme = "\
:root { --accent-primary: #8db4f7; }
.list-item.selected { border-left: 2px solid var(--accent-primary); }
";
        let doc = parse_foreign_css(theme, &[], &ThemeDoc::default());

        let painted = derive_ramp(doc.accent)[accent_default_step(false)];
        let target = parse_color_to_hsl("#8db4f7").expect("renk okunmalı");

        for (i, (a, b)) in painted.iter().zip(target.iter()).enumerate() {
            assert!(
                (a - b).abs() < 0.5,
                "boyanan basamak temanın rengiyle çakışmıyor ({i}): {painted:?} != {target:?}"
            );
        }

        // Taban ise bilerek FARKLI: rampanın altında duruyor.
        assert!(doc.accent[2] < target[2], "taban boyanan renkten koyu olmalı");
    }

    /// `--fds-accent-base` doğrudan yazılmışsa geriye çözme YAPILMAZ.
    ///
    /// O ad zaten rampanın tabanını söylüyor; bir de geriye çözmek rengi
    /// kaydırırdı.
    #[test]
    fn explicit_fds_base_is_used_as_is() {
        let theme = ":root { --fds-accent-base: 206, 100%, 42%; }";
        let doc = parse_foreign_css(theme, &[], &ThemeDoc::default());
        assert_eq!(doc.accent, [206.0, 100.0, 42.0]);
    }

    /// AÇIK bir vurguda üst basamaklar beyaza çökmemeli.
    ///
    /// Kullanıcının bildirdiği hata: içe aktarılan bir temada seçili kenar
    /// çubuğu ikonu beyaz çıkıyordu. O temanın vurgusu `217.9, 86.9%, 76.1%`
    /// — zaten açık. Işıklılık farkları SABİT eklendiği için `light-2`
    /// 76.1 + 27 = 103.1 → %100'e kırpılıyor, yani düpedüz beyaz oluyordu.
    /// Koyu kipte `--fds-accent-default` tam olarak `light-2`den türediği için
    /// vurguyla boyanan her şey beyaza dönüyordu.
    #[test]
    fn light_accent_keeps_its_ramp_distinct() {
        let ramp = derive_ramp([217.9, 86.9, 76.1]);

        for (i, step) in ramp.iter().enumerate() {
            assert!(
                step[2] < 100.0,
                "basamak {i} beyaza çökmüş: {:?}",
                step
            );
        }

        // Basamaklar birbirinden ayrışmalı: açıktan koyuya kesintisiz azalan
        // bir ışıklılık dizisi. Kırpma olduğunda üstteki ikisi eşitleniyordu.
        for pair in ramp.windows(2) {
            assert!(
                pair[0][2] > pair[1][2],
                "basamaklar ayrışmıyor: {:?} -> {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    /// KOYU bir vurguda da aynısı — alt basamaklar siyaha çökmemeli.
    #[test]
    fn dark_accent_keeps_its_ramp_distinct() {
        let ramp = derive_ramp([260.0, 70.0, 12.0]);

        for (i, step) in ramp.iter().enumerate() {
            assert!(step[2] > 0.0, "basamak {i} siyaha çökmüş: {:?}", step);
        }
        for pair in ramp.windows(2) {
            assert!(pair[0][2] > pair[1][2], "basamaklar ayrışmıyor");
        }
    }

    /// Rampa DIŞINDAKİ vurgu token'ları yazılmamalı.
    ///
    /// Onları kütüphane rampadan türetiyor ve türetme KİPE BAĞLI. Sitenin
    /// canlı CSS'inden okundu:
    ///
    /// ```text
    /// .fds-theme-light { --fds-accent-default: hsl(var(--fds-accent-dark-1)) }
    /// .fds-theme-dark  { --fds-accent-default: hsla(var(--fds-accent-light-2)) }
    /// ```
    ///
    /// Bir zamanlar hepsi `hsl(var(--fds-accent-base))` olarak yazılıyordu.
    /// Değer yanlıştı ve yönetilen blok iki kipi birden kapsadığı için
    /// kütüphanenin doğru kurallarını da eziyordu: koyu kipte bu token'dan
    /// türeyen her şey (rozet gradyanının ucu, banner çerçevesi, oynatıcı
    /// rayı, bağlantı renkleri) açık mavi olması gerekirken koyu çıkıyordu.
    #[test]
    fn mode_dependent_accent_tokens_are_left_to_the_library() {
        let doc = ThemeDoc {
            accent: [280.0, 80.0, 50.0],
            ..Default::default()
        };
        let css = doc.emit_css();

        for name in [
            "--fds-accent-default:",
            "--fds-accent-secondary:",
            "--fds-accent-tertiary:",
            "--fds-accent-text-primary:",
            "--fds-accent-text-secondary:",
            "--fds-accent-text-tertiary:",
        ] {
            assert!(!css.contains(name), "kipe bağlı token yazılmış: {name}\n{css}");
        }

        // Referans olarak KULLANMAK serbest; sorun onu TANIMLAMAKtı.
        assert!(css.contains("var(--fds-accent-default)"));
    }

    #[test]
    fn hue_wraps_around_360() {
        let ramp = derive_ramp([355.0, 100.0, 42.0]);
        assert_eq!(num(ramp[6][0]), "15");
        assert_eq!(num(ramp[0][0]), "340");
    }

    #[test]
    fn saturation_and_lightness_are_clamped() {
        let ramp = derive_ramp([200.0, 1.0, 95.0]);
        assert!(ramp.iter().all(|c| c[1] >= 0.0 && c[1] <= 100.0));
        assert!(ramp.iter().all(|c| c[2] >= 0.0 && c[2] <= 100.0));
    }

    #[test]
    fn emit_parse_roundtrip_preserves_document() {
        let original = ThemeDoc {
            accent: [280.0, 72.5, 48.0],
            mode: ThemeMode::Light,
            control_corner_radius: Some(10.0),
            overlay_corner_radius: Some(18.0),
            raw_css: "body { letter-spacing: .2px; }".into(),
            ..Default::default()
        };

        let round = parse_css(&original.emit_css(), &[], &original);

        assert_eq!(round.accent, original.accent);
        assert_eq!(round.control_corner_radius, Some(10.0));
        assert_eq!(round.overlay_corner_radius, Some(18.0));
        assert_eq!(round.raw_css, "body { letter-spacing: .2px; }");
        assert_eq!(round.mode, ThemeMode::Light);
    }

    #[test]
    fn token_and_rule_overrides_roundtrip() {
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

        let round = parse_css(&original.emit_css(), &[], &original);

        assert_eq!(round.token_overrides, token_overrides);
        assert_eq!(round.rule_overrides, rule_overrides);
        assert_eq!(round.accent, [120.0, 60.0, 40.0]);
        assert_eq!(round.control_corner_radius, Some(6.0));
        assert_eq!(round.raw_css, "footer { opacity: .5; }");
    }

    #[test]
    fn import_lines_stay_at_top_and_roundtrip() {
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

        let round = parse_css(&css, &[], &doc);
        assert_eq!(round.imports, doc.imports);
        assert_eq!(round.raw_css, "body { color: red; }");
        assert!(!round.raw_css.contains("@import"), "ham CSS'e sızmamalı");
    }

    #[test]
    fn logo_data_uri_is_not_corrupted() {
        let uri = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciLz4=";
        let mut rule_overrides = BTreeMap::new();
        rule_overrides.insert(".topbar .logo img".into(), format!("content: url(\"{uri}\");"));

        let doc = ThemeDoc {
            rule_overrides,
            ..Default::default()
        };
        let round = parse_css(&doc.emit_css(), &[], &doc);

        assert_eq!(
            round.rule_overrides.get(".topbar .logo img").map(String::as_str),
            Some(format!("content: url(\"{uri}\");").as_str())
        );
    }

    #[test]
    fn derived_accent_steps_do_not_leak_into_map() {
        let doc = ThemeDoc {
            accent: [10.0, 50.0, 50.0],
            ..Default::default()
        };
        let round = parse_css(&doc.emit_css(), &[], &doc);
        assert!(round.token_overrides.is_empty(), "sızan: {:?}", round.token_overrides);
    }

    #[test]
    fn default_document_also_roundtrips() {
        let original = ThemeDoc::default();
        let round = parse_css(&original.emit_css(), &[], &original);
        assert_eq!(round.accent, DEFAULT_ACCENT);
        assert_eq!(round.control_corner_radius, None);
        assert_eq!(round.raw_css, "");
    }

    #[test]
    fn manually_edited_accent_reflects_in_control() {
        let doc = ThemeDoc {
            accent: [206.0, 100.0, 42.0],
            ..Default::default()
        };
        let edited = format!(
            "{TOKENS_OPEN}\n:root {{\n\t--fds-accent-base: 120, 60%, 40%;\n}}\n{TOKENS_CLOSE}\n"
        );
        let parsed = parse_css(&edited, &[], &doc);
        assert_eq!(parsed.accent, [120.0, 60.0, 40.0]);
    }

    #[test]
    fn without_marker_text_becomes_the_base_layer() {
        let current = ThemeDoc {
            accent: [300.0, 50.0, 50.0],
            control_corner_radius: Some(9.0),
            ..Default::default()
        };
        let parsed = parse_css("body { color: red; }", &[], &current);
        assert_eq!(parsed.accent, [300.0, 50.0, 50.0], "accent sıfırlanmamalı");
        assert_eq!(parsed.control_corner_radius, Some(9.0));
        // İşaretleyici yoksa metin yabancı bir tema sayılıyor ve TABAN
        // katmanına düşüyor; kullanıcının kendi ham CSS'i henüz boş.
        assert_eq!(parsed.imported_rules.len(), 1);
        assert_eq!(parsed.imported_rules[0].selector, "body");
        assert_eq!(parsed.imported_rules[0].body, "color: red;");
        assert!(parsed.imported_css.is_empty(), "ham blok artık üretilmiyor");
        assert_eq!(parsed.raw_css, "");
    }

    /// İçe aktarılan tema TABAN; kontroller onun ÜSTÜNE biner.
    ///
    /// Kullanıcının bildirdiği hatanın tam karşılığı: GitHub'dan ya da diskten
    /// gelen bir tema önizlemede görünüyor, ama sonrasında hiçbir kontrol
    /// değişikliği yansımıyordu. Sebebi yayılma sırasıydı — içe aktarılan gövde
    /// yönetilen bloktan SONRA yazıldığı için, eşit özgüllükte her zaman o
    /// kazanıyordu. Bu test sırayı sabitliyor.
    #[test]
    fn imported_rules_are_the_base_layer() {
        let doc = ThemeDoc {
            accent: [120.0, 60.0, 40.0],
            imported_rules: vec![crate::theme::models::ImportedRule {
                selector: ".ithal".into(),
                body: "color: red;".into(),
                ..Default::default()
            }],
            raw_css: ".kullanici { color: blue; }".into(),
            ..Default::default()
        };
        let css = doc.emit_css();

        let imported = css.find(".ithal").expect("içe aktarılan gövde yazılmamış");
        let block_open = css.find(TOKENS_OPEN).expect("yönetilen blok yok");
        let block_close = css.find(TOKENS_CLOSE).expect("yönetilen blok kapanmıyor");
        let user = css.find(".kullanici").expect("kullanıcının ham CSS'i yazılmamış");

        assert!(imported < block_open, "içe aktarılan tema bloktan ÖNCE gelmeli");
        assert!(block_close < user, "kullanıcının ham CSS'i bloktan SONRA gelmeli");

        // Ve tur: iki katman karışmadan geri okunmalı.
        let round = parse_css(&css, &[], &doc);
        assert_eq!(round.imported_rules.len(), 1);
        assert_eq!(round.imported_rules[0].selector, ".ithal");
        assert_eq!(round.imported_rules[0].body, "color: red;");
        assert_eq!(round.raw_css, ".kullanici { color: blue; }");
    }

    #[test]
    fn css_written_outside_block_is_preserved() {
        let doc = ThemeDoc::default();
        let text = format!("h1 {{ color: red; }}\n{TOKENS_OPEN}\n{TOKENS_CLOSE}\nh2 {{ color: blue; }}");
        let parsed = parse_css(&text, &[], &doc);
        assert!(preserved(&parsed).contains("h1"));
        assert!(preserved(&parsed).contains("h2"));
    }

    #[test]
    fn radius_and_raw_css_are_written() {
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

    /// Yönetilen bloğun DIŞINDA korunan her şey, tek metin hâlinde.
    ///
    /// Testlerin çoğu "şu kural kayboldu mu" diye soruyor; metnin hangi
    /// katmanda durduğu (taban mı, kullanıcının ham CSS'i mi) ayrı ve keskin
    /// bir testin konusu: `imported_rules_are_the_base_layer`.
    fn preserved(doc: &ThemeDoc) -> String {
        format!("{}\n{}", imported_text(&doc.imported_rules), doc.raw_css)
    }

    /// İthal kuralları okunabilir CSS'e çevirir.
    ///
    /// `emit_css`in kendi yazıcısı KULLANILMIYOR: o, kontrollerin sahiplendiği
    /// bildirimleri eliyor. Bir kuralın modele girip girmediğini sınarken bu
    /// eleme sonucu gizlerdi.
    fn imported_text(rules: &[crate::theme::models::ImportedRule]) -> String {
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

    fn known() -> Vec<String> {
        vec![
            ".anime-card, .slider-card, .grid-view-item".to_string(),
            ".topbar a.logo::before".to_string(),
        ]
    }

    #[test]
    fn foreign_css_maps_tokens_and_rules_to_controls() {
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
        assert_eq!(
            parsed
                .rule_overrides
                .get(".anime-card, .slider-card, .grid-view-item")
                .map(String::as_str),
            Some("border-radius: 14px;")
        );
        assert!(parsed.raw_css.trim().is_empty(), "kalan: {:?}", parsed.raw_css);
    }

    #[test]
    fn unmapped_rules_preserved_in_raw_css() {
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
        assert!(preserved(&parsed).contains(".ozel-rozet"));
        assert!(preserved(&parsed).contains("linear-gradient"));
        assert!(preserved(&parsed).contains("tanımadığımız"));
        assert!(preserved(&parsed).contains("@media"));
        assert!(preserved(&parsed).contains("10, 10%, 10%"));
    }

    #[test]
    fn mixed_root_block_is_not_split() {
        let doc = ThemeDoc::default();
        let css = ":root { --fds-accent-base: 12, 88%, 55%; color-scheme: dark; }";
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.accent, [12.0, 88.0, 55.0]);
        assert!(preserved(&parsed).contains("color-scheme: dark"));
    }

    /// Rampa basamağı yalnızca TÜRETİLENDEN FARKLIYSA korunuyor.
    ///
    /// Kural değere bakıyor, isme değil: türetilenle aynı değer gereksiz bir
    /// kopya olurdu (her turda blok şişerdi), farklı bir değer ise temanın
    /// bilinçli kararı. Eskiden ada bakılıp hepsi atılıyordu ve bu, dosyada
    /// açıkça yazan rengi sessizce değiştiriyordu.
    #[test]
    fn derived_accent_step_matching_base_is_not_duplicated() {
        let doc = ThemeDoc::default();
        // 280, 70%, 50% tabanının türettiği dark-1 basamağı.
        let derived_dark_1 = fmt_triplet(derive_ramp([280.0, 70.0, 50.0])[4]);
        let css = format!(
            ":root {{ --fds-accent-base: 280, 70%, 50%; --fds-accent-dark-1: {derived_dark_1}; }}"
        );
        let parsed = parse_foreign_css(&css, &known(), &doc);

        assert_eq!(parsed.accent, [280.0, 70.0, 50.0]);
        assert!(
            !parsed.token_overrides.contains_key("--fds-accent-dark-1"),
            "tabandan yeniden üretilecek basamak ikinci kez yazılmamalı"
        );
    }

    #[test]
    fn hand_written_accent_step_survives() {
        let doc = ThemeDoc::default();
        // ytanime teması tam olarak bunu yapıyor: tabanı sitenin
        // varsayılanında bırakıp tek tek basamakları beyaza çekiyor.
        let css = ":root { --fds-accent-base: 206, 100%, 42%; --fds-accent-light-3: 0, 0%, 100%; }";
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(
            parsed.token_overrides.get("--fds-accent-light-3").map(String::as_str),
            Some("0, 0%, 100%"),
            "dosyanın elle yazdığı basamak korunmalı"
        );
        assert!(
            parsed.emit_css().contains("--fds-accent-light-3: 0, 0%, 100%"),
            "korunan basamak üretilen CSS'e de yazılmalı"
        );
    }

    #[test]
    fn braces_in_comments_and_strings_do_not_shift_rule_boundary() {
        let doc = ThemeDoc::default();
        let css = r#"
/* eski kural: .x { color: red } */
.ozel::after { content: "}"; color: lime; }
:root { --fds-accent-base: 100, 40%, 40%; }
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.accent, [100.0, 40.0, 40.0]);
        assert!(preserved(&parsed).contains(".ozel::after"));
        assert!(preserved(&parsed).contains("color: lime"));
    }

    #[test]
    fn import_lines_are_separated() {
        let doc = ThemeDoc::default();
        let css = "@import url(\"https://fonts.googleapis.com/css2?family=Inter\");\n\
                   :root { --fds-accent-base: 5, 5%, 5%; }";
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.imports, vec!["https://fonts.googleapis.com/css2?family=Inter"]);
        assert!(!preserved(&parsed).contains("@import"));
    }

    #[test]
    fn imported_theme_remains_intact_in_emitted_css() {
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
        // Kontrole taşınan kural artık ağırlıklı yazılıyor: ithal tema
        // varken bildirimler `!important` alıyor, yoksa temanın kendi
        // `!important` bildirimlerini yenemiyorlardı (bkz.
        // `fidelity_tests::control_rules_outweigh_the_imported_theme`).
        assert!(emitted.contains("border-radius: 14px !important;"), "{emitted}");
        assert!(emitted.contains(".ozel-rozet"));
    }

    #[test]
    fn parse_css_behavior_unchanged() {
        let doc = ThemeDoc::default();
        let text = ".ozel { color: red; }";
        let parsed = parse_css(text, &[], &doc);
        assert!(preserved(&parsed).contains(".ozel"));
        assert!(parsed.rule_overrides.is_empty());
        assert_eq!(parsed.accent, doc.accent);
    }

    #[test]
    fn lenient_color_and_radius_parsing_works() {
        let hex_color = parse_color_to_hsl("#3b82f6").expect("hex parsed");
        assert!((hex_color[0] - 217.0).abs() < 2.0);

        let rgb_color = parse_color_to_hsl("rgb(59, 130, 246)").expect("rgb parsed");
        assert!((rgb_color[0] - 217.0).abs() < 2.0);

        assert_eq!(parse_px("0.75rem"), Some(12.0));
        assert_eq!(parse_px("1em"), Some(16.0));
        assert_eq!(parse_px("14px"), Some(14.0));
    }

    #[test]
    fn generic_css_variables_map_to_controls() {
        let doc = ThemeDoc::default();
        let css = r#"
:root {
    --primary: #3b82f6;
    --border-radius: 0.75rem;
}
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);

        // Saklanan değer rampanın TABANI; temanın rengi ise sitenin boyamada
        // kullandığı basamakta (`--fds-accent-default`) çıkmalı. Eskiden
        // tabanın kendisi karşılaştırılıyordu, ama site o basamağı değil
        // türetileni kullanıyor (gerekçe: `color::base_for_step`).
        let painted = derive_ramp(parsed.accent)[accent_default_step(false)];
        assert!(
            (painted[0] - 217.0).abs() < 2.0,
            "tema rengi boyanan basamakta çıkmalı: {painted:?}"
        );
        assert_eq!(parsed.control_corner_radius, Some(12.0), "border-radius mapped to control_corner_radius");
    }

    #[test]
    fn external_community_theme_midnight_is_parsed() {
        let doc = ThemeDoc::default();
        let css = r#"
/* Midnight OpenAnime Theme */
:root {
    --midnight-blue: #7aa2f7;
    --accent: var(--midnight-blue);
    --border-radius: 8px;
    --bg-primary: #0f0f17;
    --text-primary: #c0caf5;
    font-family: 'Inter', sans-serif;
    color-scheme: dark;
}

body {
    background-color: var(--bg-primary);
}
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);

        // `var()` zinciri çözülmeli ve temanın rengi sitenin boyadığı
        // basamakta çıkmalı (gerekçe: `color::base_for_step`).
        let painted = derive_ramp(parsed.accent)[accent_default_step(false)];
        assert!(
            (painted[0] - 221.0).abs() < 2.0,
            "var() ile gelen tema rengi boyanan basamakta çıkmalı: {painted:?}"
        );
        assert_eq!(parsed.control_corner_radius, Some(8.0), "border-radius mapped");
        assert_eq!(
            parsed.token_overrides.get("--fds-solid-background-base").map(String::as_str),
            Some("#0f0f17")
        );
        assert_eq!(
            parsed.token_overrides.get("--fds-text-primary").map(String::as_str),
            Some("#c0caf5")
        );
        assert!(preserved(&parsed).contains("font-family"), "font-family preserved in raw_css");
    }

    #[test]
    fn url_variables_and_image_paths_become_tokens() {
        let doc = ThemeDoc::default();
        let css = r#"
:root {
    --url-logo: url('data:image/svg+xml,<svg>test</svg>');
    --url-bg: url('https://i.imgur.com/2BdJVda.png');
}
.topbar a.logo::before {
    background: var(--url-logo) no-repeat center / contain !important;
}
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);
        assert_eq!(
            parsed.token_overrides.get("--url-logo").map(String::as_str),
            Some("url('data:image/svg+xml,<svg>test</svg>')")
        );
        assert_eq!(
            parsed.token_overrides.get("--url-bg").map(String::as_str),
            Some("url('https://i.imgur.com/2BdJVda.png')")
        );
    }


}
