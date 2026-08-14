use std::collections::BTreeMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::theme::color::{DEFAULT_ACCENT, Hsl};

pub const TOKENS_OPEN: &str = "/* <oa:tokens> */";
pub const TOKENS_CLOSE: &str = "/* </oa:tokens> */";

#[derive(Default)]
pub struct ThemeState(pub Mutex<ThemeDoc>);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
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
    pub control_corner_radius: Option<f64>,
    pub overlay_corner_radius: Option<f64>,
    #[serde(default)]
    pub imports: Vec<String>,
    #[serde(default)]
    pub token_overrides: BTreeMap<String, String>,
    #[serde(default)]
    pub rule_overrides: BTreeMap<String, String>,
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

impl ThemeDoc {
    pub fn emit_css(&self) -> String {
        crate::theme::emit::emit_css(self)
    }
}
