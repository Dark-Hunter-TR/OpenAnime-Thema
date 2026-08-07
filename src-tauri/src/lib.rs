mod preview;
mod projects;
mod theme;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};

use theme::{derive_ramp, fmt_triplet, parse_css, parse_foreign_css, ThemeDoc, ThemeState};

/// `apply_theme` sonucu. CSS ve türetilmiş accent rampası tek turda döner ki
/// editör her slider hareketinde iki ayrı IPC çağrısı yapmak zorunda kalmasın.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplyResult {
    css: String,
    /// Yedi accent basamağı, `"206, 100%, 42%"` formatında.
    ramp: Vec<String>,
}

/// Editörden gelen taslağı kaydeder, CSS'i üretir ve önizlemeye uygular.
///
/// Frontend taslağın sahibi; Rust ise CSS'in tek üreticisi. Dönen string
/// hem önizlemeye enjekte edilen hem de dışa aktarılacak olan CSS'tir.
#[tauri::command]
fn apply_theme(
    app: AppHandle,
    state: State<ThemeState>,
    doc: ThemeDoc,
) -> Result<ApplyResult, String> {
    let (css, mode, ramp) = {
        let mut current = state.0.lock().map_err(|e| e.to_string())?;
        *current = doc;
        let ramp = derive_ramp(current.accent)
            .iter()
            .map(|c| fmt_triplet(*c))
            .collect();
        (current.emit_css(), current.mode, ramp)
    };

    preview::apply_css(&app, &css);
    preview::apply_mode(&app, mode);

    Ok(ApplyResult { css, ramp })
}

#[tauri::command]
fn get_theme(state: State<ThemeState>) -> Result<ThemeDoc, String> {
    Ok(state.0.lock().map_err(|e| e.to_string())?.clone())
}

/// Kod editöründeki metni `ThemeDoc`'a çevirir ve önizlemeye uygular.
///
/// `apply_theme`'in ters yönü: görsel kontroller ile kod editörü aynı state
/// üzerinde çalışsın diye. Dönen doküman ile editör kontrolleri güncellenir.
#[tauri::command]
fn apply_css_text(
    app: AppHandle,
    state: State<ThemeState>,
    text: String,
) -> Result<ThemeDoc, String> {
    let (doc, mode) = {
        let mut current = state.0.lock().map_err(|e| e.to_string())?;
        let parsed = parse_css(&text, &current);
        *current = parsed.clone();
        (parsed, current.mode)
    };

    // Kullanıcının yazdığı metni birebir uygula — emit edilmiş hâlini değil.
    // Aksi hâlde her tuş vuruşunda metin yeniden biçimlenip imleç zıplardı.
    preview::apply_css(&app, &text);
    preview::apply_mode(&app, mode);

    Ok(doc)
}

/// Harici bir .css dosyasını okur.
///
/// Yolu frontend, dialog eklentisinin dosya seçicisinden alıyor; yani
/// kullanıcının açıkça seçtiği dosya dışında bir şey okunmuyor.
#[tauri::command]
fn read_css_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("{path} okunamadı: {e}"))
}

/// Düzenlenen CSS'i aynı dosyaya geri yazar.
#[tauri::command]
fn write_css_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| format!("{path} yazılamadı: {e}"))
}

/// Bir görseli okuyup `data:` URI'sine çevirir.
///
/// Tema CSS'i tek bir metin olarak taşındığı (ve `localStorage.theme_content`
/// içine yazıldığı) için görselin dosya yolunu değil, gömülü hâlini saklamak
/// zorundayız — aksi hâlde tema başka bir makinede açıldığında görsel kaybolur.
#[tauri::command]
fn read_image_data_uri(path: String) -> Result<String, String> {
    use base64::Engine as _;

    let bytes = std::fs::read(&path).map_err(|e| format!("{path} okunamadı: {e}"))?;

    // ~5 MB üstü görseller localStorage kotasını (genelde 5 MB) zorlar.
    const LIMIT: usize = 3 * 1024 * 1024;
    if bytes.len() > LIMIT {
        return Err(format!(
            "Görsel çok büyük ({} KB). Tema tek bir metin olarak saklandığı için \
             en fazla {} KB olabilir.",
            bytes.len() / 1024,
            LIMIT / 1024
        ));
    }

    let mime = match std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("avif") => "image/avif",
        other => return Err(format!("desteklenmeyen görsel türü: {other:?}")),
    };

    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
}

#[tauri::command]
fn set_preview_bounds(
    app: AppHandle,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), String> {
    preview::set_bounds(&app, x, y, width, height).map_err(|e| e.to_string())
}

#[tauri::command]
fn preview_navigate(app: AppHandle, url: String) -> Result<(), String> {
    preview::navigate(&app, &url)
}

/// Ana ekran / ayarlar görünümünde önizlemeyi gizler.
#[tauri::command]
fn set_preview_visible(app: AppHandle, visible: bool) -> Result<(), String> {
    preview::set_visible(&app, visible).map_err(|e| e.to_string())
}

/// Önizlemede openani.me oturumu açık mı?
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginState {
    logged_in: bool,
}

/// Önizleme webview'inin çerez kavanozuna bakarak oturum durumunu söyler.
///
/// **Sinyalin kaynağı tahmin değil**, sitenin kendi kodu: dört giriş yolunun
/// (parola, QR/DAG, kayıt, e-posta doğrulama) dördü de
/// `window.setCookie("token", <jwt>, "7")` çağırıyor; çıkışta ise
/// `window.setCookie("token", "", -1)` ile süresi geçmişe çekiliyor. Dolayısıyla
/// `token` çerezinin VARLIĞI ve boş olmaması oturumun açık olduğu anlamına gelir.
///
/// Yalnızca çerezin var olup olmadığına bakıyoruz; değerini OKUYUP kullanmıyoruz.
/// Token'la siteye istek atmak, uygulamayı kullanıcı adına davranan yetkisiz bir
/// API istemcisine çevirirdi — bu projede bilinçle kaçınılan şey tam olarak bu.
///
/// `async` olması ZORUNLU: `cookies_for_url` Windows'ta senkron bir komuttan ya
/// da olay işleyicisinden çağrılırsa WebView2 kilitleniyor (tauri kaynağındaki
/// "Known issues" notu, wry#583). Async komut tokio üzerinde çalıştığı için
/// ana iş parçacığını bloklamıyor.
#[tauri::command]
async fn preview_login_state(app: AppHandle) -> Result<LoginState, String> {
    let Some(webview) = app.get_webview(preview::PREVIEW_LABEL) else {
        return Ok(LoginState { logged_in: false });
    };

    let url: url::Url = preview::SITE_URL
        .parse()
        .map_err(|e| format!("önizleme adresi çözümlenemedi: {e}"))?;

    let cookies = webview
        .cookies_for_url(url)
        .map_err(|e| format!("çerezler okunamadı: {e}"))?;

    let logged_in = cookies
        .iter()
        .any(|c| c.name() == "token" && !c.value().trim().is_empty());

    Ok(LoginState { logged_in })
}

/// Dışarıdan gelen bir temayı (GitHub, pano, dosya) kontrollere eşler.
///
/// `apply_css_text`'ten ayrı bir komut: o, editörün kendi işaretleyici
/// bloğuna güvenen gidiş-geliş yolu; bu ise işaretleyicisi hiç olmayan yabancı
/// bir CSS'i ilk kez içeri alır. İkisini ayırmak, kod editörünün senkron
/// davranışını değiştirmeden içe aktarmayı eklememizi sağlıyor.
///
/// `knownSelectors` frontend'den geliyor (`$lib/advanced.ts`) — selector
/// haritasının tek doğruluk kaynağı orada kalsın diye.
#[tauri::command]
fn import_css_text(
    app: AppHandle,
    state: State<ThemeState>,
    text: String,
    known_selectors: Vec<String>,
) -> Result<ThemeDoc, String> {
    let (doc, mode) = {
        let mut current = state.0.lock().map_err(|e| e.to_string())?;
        let parsed = parse_foreign_css(&text, &known_selectors, &current);
        *current = parsed.clone();
        (parsed, current.mode)
    };

    // Önizlemeye, kullanıcının getirdiği metni değil ÜRETİLEN CSS'i basıyoruz:
    // içe aktarma sonrası kontroller doğrudan bu belgeyi düzenlemeye başlıyor,
    // dolayısıyla önizleme de o belgenin çıktısını göstermeli.
    preview::apply_css(&app, &doc.emit_css());
    preview::apply_mode(&app, mode);

    Ok(doc)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(ThemeState::default())
        .invoke_handler(tauri::generate_handler![
            apply_theme,
            apply_css_text,
            get_theme,
            read_css_file,
            write_css_file,
            read_image_data_uri,
            set_preview_bounds,
            preview_navigate,
            set_preview_visible,
            preview_login_state,
            import_css_text,
            projects::projects_dir_path,
            projects::list_projects,
            projects::load_project,
            projects::save_project,
            projects::delete_project,
            projects::rename_project
        ])
        .setup(|app| {
            let doc = {
                let state: State<ThemeState> = app.state();
                let guard = state.0.lock().expect("tema kilidi");
                guard.clone()
            };
            preview::create(app.handle(), &doc)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
