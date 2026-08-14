#[cfg(test)]
mod tests {
    use crate::theme::*;
    use crate::theme::color::*;
    use crate::theme::emit::*;
    use crate::theme::parse::*;
    use crate::theme::models::*;
    use std::collections::BTreeMap;

    #[test]
    fn varsayilan_taban_kutuphane_rampasini_uretir() {
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
        assert_eq!(num(ramp[6][0]), "15");
        assert_eq!(num(ramp[0][0]), "340");
    }

    #[test]
    fn doygunluk_ve_isik_kirpilir() {
        let ramp = derive_ramp([200.0, 1.0, 95.0]);
        assert!(ramp.iter().all(|c| c[1] >= 0.0 && c[1] <= 100.0));
        assert!(ramp.iter().all(|c| c[2] >= 0.0 && c[2] <= 100.0));
    }

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

        let round = parse_css(&original.emit_css(), &[], &original);

        assert_eq!(round.accent, original.accent);
        assert_eq!(round.control_corner_radius, Some(10.0));
        assert_eq!(round.overlay_corner_radius, Some(18.0));
        assert_eq!(round.raw_css, "body { letter-spacing: .2px; }");
        assert_eq!(round.mode, ThemeMode::Light);
    }

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

        let round = parse_css(&original.emit_css(), &[], &original);

        assert_eq!(round.token_overrides, token_overrides);
        assert_eq!(round.rule_overrides, rule_overrides);
        assert_eq!(round.accent, [120.0, 60.0, 40.0]);
        assert_eq!(round.control_corner_radius, Some(6.0));
        assert_eq!(round.raw_css, "footer { opacity: .5; }");
    }

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

        let round = parse_css(&css, &[], &doc);
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
        let round = parse_css(&doc.emit_css(), &[], &doc);

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
        let round = parse_css(&doc.emit_css(), &[], &doc);
        assert!(round.token_overrides.is_empty(), "sızan: {:?}", round.token_overrides);
    }

    #[test]
    fn varsayilan_dokuman_da_tur_atlatir() {
        let original = ThemeDoc::default();
        let round = parse_css(&original.emit_css(), &[], &original);
        assert_eq!(round.accent, DEFAULT_ACCENT);
        assert_eq!(round.control_corner_radius, None);
        assert_eq!(round.raw_css, "");
    }

    #[test]
    fn elle_duzenlenen_accent_kontrole_yansir() {
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
    fn isaretleyici_yoksa_kontroller_korunur_metin_ham_sayilir() {
        let current = ThemeDoc {
            accent: [300.0, 50.0, 50.0],
            control_corner_radius: Some(9.0),
            ..Default::default()
        };
        let parsed = parse_css("body { color: red; }", &[], &current);
        assert_eq!(parsed.accent, [300.0, 50.0, 50.0], "accent sıfırlanmamalı");
        assert_eq!(parsed.control_corner_radius, Some(9.0));
        assert_eq!(parsed.raw_css, "body { color: red; }");
    }

    #[test]
    fn blok_disina_yazilan_css_korunur() {
        let doc = ThemeDoc::default();
        let text = format!("h1 {{ color: red; }}\n{TOKENS_OPEN}\n{TOKENS_CLOSE}\nh2 {{ color: blue; }}");
        let parsed = parse_css(&text, &[], &doc);
        assert!(parsed.raw_css.contains("h1"));
        assert!(parsed.raw_css.contains("h2"));
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
        assert!(parsed.raw_css.contains(".ozel-rozet"));
        assert!(parsed.raw_css.contains("linear-gradient"));
        assert!(parsed.raw_css.contains("tanımadığımız"));
        assert!(parsed.raw_css.contains("@media"));
        assert!(parsed.raw_css.contains("10, 10%, 10%"));
    }

    #[test]
    fn karisik_root_blogu_bolunmez() {
        let doc = ThemeDoc::default();
        let css = ":root { --fds-accent-base: 12, 88%, 55%; color-scheme: dark; }";
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert_eq!(parsed.accent, [12.0, 88.0, 55.0]);
        assert!(parsed.raw_css.contains("color-scheme: dark"));
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

    #[test]
    fn parse_css_davranisi_degismedi() {
        let doc = ThemeDoc::default();
        let text = ".ozel { color: red; }";
        let parsed = parse_css(text, &[], &doc);
        assert!(parsed.raw_css.contains(".ozel"));
        assert!(parsed.rule_overrides.is_empty());
        assert_eq!(parsed.accent, doc.accent);
    }

    #[test]
    fn esnek_renk_ve_yaricap_ayristirma_calisir() {
        let hex_color = parse_color_to_hsl("#3b82f6").expect("hex parsed");
        assert!((hex_color[0] - 217.0).abs() < 2.0);

        let rgb_color = parse_color_to_hsl("rgb(59, 130, 246)").expect("rgb parsed");
        assert!((rgb_color[0] - 217.0).abs() < 2.0);

        assert_eq!(parse_px("0.75rem"), Some(12.0));
        assert_eq!(parse_px("1em"), Some(16.0));
        assert_eq!(parse_px("14px"), Some(14.0));
    }

    #[test]
    fn jenerik_css_degiskenleri_kontrollere_eslenir() {
        let doc = ThemeDoc::default();
        let css = r#"
:root {
    --primary: #3b82f6;
    --border-radius: 0.75rem;
}
"#;
        let parsed = parse_foreign_css(css, &known(), &doc);

        assert!((parsed.accent[0] - 217.0).abs() < 2.0, "primary variable mapped to accent");
        assert_eq!(parsed.control_corner_radius, Some(12.0), "border-radius mapped to control_corner_radius");
    }

    #[test]
    fn harici_topluluk_temasi_midnight_ayristirilir() {
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

        assert!((parsed.accent[0] - 221.0).abs() < 2.0, "var() resolved accent mapped");
        assert_eq!(parsed.control_corner_radius, Some(8.0), "border-radius mapped");
        assert_eq!(
            parsed.token_overrides.get("--fds-solid-background-base").map(String::as_str),
            Some("#0f0f17")
        );
        assert_eq!(
            parsed.token_overrides.get("--fds-text-primary").map(String::as_str),
            Some("#c0caf5")
        );
        assert!(parsed.raw_css.contains("font-family"), "font-family preserved in raw_css");
    }

    #[test]
    fn url_degiskenleri_ve_gorsel_yollari_tokenlara_girer() {
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
