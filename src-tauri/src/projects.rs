//! Yerel tema projeleri.
//!
//! Bir proje = `app_data_dir()/projects/<id>.json`. Tek dosya, tek proje.
//! Ana ekrandaki liste dizini tarayarak kuruluyor; ayrı bir indeks dosyası
//! bilerek YOK. İndeks olsaydı dosyalarla senkron kalmak zorunda olurdu ve
//! elle silinen bir proje listede hayalet olarak kalırdı.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::theme::{Hsl, ThemeDoc, ThemeMode};

/// Kaydedilen projenin tam gövdesi.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub created_at: u64,
    #[serde(default)]
    pub updated_at: u64,

    /// Temanın kendisi — CSS bundan üretilir.
    pub doc: ThemeDoc,

    /// Görsel kontrollerin durumu (gelişmiş bölümler, toggle'lar, renk
    /// alanları …).
    ///
    /// Rust bunu YORUMLAMAZ, olduğu gibi taşır. Bilerek opak: kontrol
    /// durumunun şekli frontend'e ait ve oraya yeni bir bölüm eklendiğinde
    /// burada karşılık bir tip güncellemesi gerekmemeli. Aksi hâlde her yeni
    /// slider iki yerde birden tanımlanırdı.
    #[serde(default)]
    pub ui: serde_json::Value,

    /// Kod editöründeki metin.
    ///
    /// `doc`'tan yeniden üretilebilir görünse de saklamak zorundayız:
    /// kullanıcı kod modunda işaretleyici bloğun DIŞINA yazdıysa o metin
    /// yalnızca burada yaşar.
    #[serde(default)]
    pub css_text: String,

    /// Proje bir harici `.css` dosyasına bağlıysa onun yolu.
    #[serde(default)]
    pub external_path: Option<String>,

    /// Projenin kökeni (ör. içe aktarıldığı GitHub bağlantısı). Yalnız bilgi.
    #[serde(default)]
    pub source: Option<String>,
}

/// Ana ekrandaki kart listesi için hafif özet.
///
/// Projeler gömülü `data:` URI'leri (logo, maskot, arkaplan) yüzünden
/// megabaytlarca olabiliyor. Liste ekranı yalnızca ad, tarih ve kart
/// önizlemesi için gereken accent/mod bilgisini istiyor; tam gövdeleri
/// belleğe almak açılışı gereksiz yere yavaşlatırdı.
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub created_at: u64,
    pub updated_at: u64,
    /// Kartın renk şeridini çizmek için.
    pub accent: Hsl,
    pub mode: ThemeMode,
    pub source: Option<String>,
}

/// Özet okurken kullanılan kısmi şema.
///
/// serde bilinmeyen alanları varsayılan olarak yok sayar, dolayısıyla dev
/// `ui` / `cssText` alanları buraya hiç materyalize edilmez.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMeta {
    id: String,
    name: String,
    #[serde(default)]
    created_at: u64,
    #[serde(default)]
    updated_at: u64,
    doc: MetaDoc,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Deserialize)]
struct MetaDoc {
    accent: Hsl,
    mode: ThemeMode,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Kimlik dosya adına dönüştüğü için dizin dışına çıkmaya izin verilmemeli.
///
/// `id` frontend'den geliyor; `../` ya da mutlak yol içeren bir değer
/// gönderilirse `projects` klasörünün dışındaki bir dosya okunabilir veya
/// silinebilirdi. Beyaz liste ile kapatıyoruz.
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("p-{nanos:x}")
}

fn projects_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("uygulama veri klasörü bulunamadı: {e}"))?
        .join("projects");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("{} oluşturulamadı: {e}", dir.display()))?;
    Ok(dir)
}

fn project_path(app: &AppHandle, id: &str) -> Result<PathBuf, String> {
    if !valid_id(id) {
        return Err(format!("geçersiz proje kimliği: {id}"));
    }
    Ok(projects_dir(app)?.join(format!("{id}.json")))
}

/// Projelerin tutulduğu klasörün tam yolu.
///
/// Ayarlar sayfasında gösteriliyor ve "klasörü aç" düğmesi bunu kullanıyor —
/// kullanıcı dosyalarının nerede olduğunu bilmeli, yedekleyebilmeli.
#[tauri::command]
pub fn projects_dir_path(app: AppHandle) -> Result<String, String> {
    Ok(projects_dir(&app)?.to_string_lossy().to_string())
}

/// Kayıtlı projeleri, en son güncellenen başta olacak şekilde listeler.
///
/// Bozuk / elle düzenlenmiş bir dosya tüm listeyi düşürmemeli; okunamayan
/// dosyalar sessizce atlanıyor.
#[tauri::command]
pub fn list_projects(app: AppHandle) -> Result<Vec<ProjectSummary>, String> {
    let dir = projects_dir(&app)?;
    let entries = std::fs::read_dir(&dir).map_err(|e| format!("{} okunamadı: {e}", dir.display()))?;

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(meta) = serde_json::from_str::<ProjectMeta>(&text) else {
            continue;
        };
        out.push(ProjectSummary {
            id: meta.id,
            name: meta.name,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
            accent: meta.doc.accent,
            mode: meta.doc.mode,
            source: meta.source,
        });
    }

    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(out)
}

#[tauri::command]
pub fn load_project(app: AppHandle, id: String) -> Result<Project, String> {
    let path = project_path(&app, &id)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("proje okunamadı ({}): {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("proje dosyası bozuk: {e}"))
}

/// Projeyi diske yazar ve kaydedilmiş hâlini geri döner.
///
/// `id` boşsa yeni bir proje oluşturulur. Dönen nesne frontend'in elindeki
/// taslağın yerine geçer — kimlik ve zaman damgaları böylece tek yerde
/// (burada) üretilmiş olur.
#[tauri::command]
pub fn save_project(app: AppHandle, mut project: Project) -> Result<Project, String> {
    let now = now_ms();

    if project.id.trim().is_empty() {
        project.id = new_id();
        project.created_at = now;
    } else if !valid_id(&project.id) {
        return Err(format!("geçersiz proje kimliği: {}", project.id));
    }
    if project.created_at == 0 {
        project.created_at = now;
    }
    project.updated_at = now;

    project.name = project.name.trim().to_string();
    if project.name.is_empty() {
        project.name = "Adsız tema".into();
    }

    let path = project_path(&app, &project.id)?;
    let text =
        serde_json::to_string_pretty(&project).map_err(|e| format!("proje yazılamadı: {e}"))?;
    std::fs::write(&path, text).map_err(|e| format!("{} yazılamadı: {e}", path.display()))?;

    Ok(project)
}

#[tauri::command]
pub fn delete_project(app: AppHandle, id: String) -> Result<(), String> {
    let path = project_path(&app, &id)?;
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        // Zaten yoksa bu bir hata değil: kullanıcı için sonuç aynı.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("proje silinemedi: {e}")),
    }
}

/// Yalnızca adı değiştirir; tema gövdesine dokunmaz.
#[tauri::command]
pub fn rename_project(app: AppHandle, id: String, name: String) -> Result<Project, String> {
    let mut project = load_project(app.clone(), id)?;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Proje adı boş olamaz.".into());
    }
    project.name = trimmed.to_string();
    save_project(app, project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kimlik_dogrulama_dizin_disina_cikmayi_engeller() {
        assert!(valid_id("p-1a2b3c"));
        assert!(valid_id("abc123"));

        // Yol ayracı ya da üst dizin içeren hiçbir şey kabul edilmemeli.
        assert!(!valid_id("../gizli"));
        assert!(!valid_id("a/b"));
        assert!(!valid_id("a\\b"));
        assert!(!valid_id("C:"));
        assert!(!valid_id(".."));
        assert!(!valid_id(""));
        assert!(!valid_id("nokta.li"));
        assert!(!valid_id(&"x".repeat(65)));
    }

    #[test]
    fn uretilen_kimlik_gecerlidir() {
        assert!(valid_id(&new_id()));
    }

    /// Özet şeması tam gövdeden okunabilmeli — `ui` / `cssText` gibi ağır
    /// alanlar yok sayılmalı.
    #[test]
    fn ozet_semasi_tam_projeyi_okuyabilir() {
        let full = serde_json::json!({
            "id": "p-1",
            "name": "Test",
            "createdAt": 5u64,
            "updatedAt": 9u64,
            "doc": {
                "accent": [200.0, 90.0, 40.0],
                "mode": "light",
                "controlCornerRadius": null,
                "overlayCornerRadius": null,
                "imports": [],
                "tokenOverrides": {},
                "ruleOverrides": {},
                "rawCss": ""
            },
            "ui": { "adv": { "cok": "buyuk" } },
            "cssText": "body{}",
            "source": "https://github.com/x/y"
        });

        let meta: ProjectMeta = serde_json::from_str(&full.to_string()).expect("özet okunmalı");
        assert_eq!(meta.id, "p-1");
        assert_eq!(meta.name, "Test");
        assert_eq!(meta.updated_at, 9);
        assert_eq!(meta.doc.mode, ThemeMode::Light);
        assert_eq!(meta.doc.accent, [200.0, 90.0, 40.0]);
        assert_eq!(meta.source.as_deref(), Some("https://github.com/x/y"));
    }
}
