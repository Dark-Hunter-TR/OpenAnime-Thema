//! Önizleme child webview'i: oluşturma, konumlandırma ve tema enjeksiyonu.
//!
//! openani.me `x-frame-options: SAMEORIGIN` gönderdiği için iframe kullanılamaz
//! (PLAN.md §0.5). Bu yüzden site, ana pencereye `Window::add_child` ile eklenen
//! gerçek bir webview içinde gösteriliyor (`unstable` feature gerekir).

use std::sync::{Mutex, OnceLock};

use tauri::{
    webview::{PageLoadEvent, Webview, WebviewBuilder},
    AppHandle, LogicalPosition, LogicalSize, Manager, Rect, Runtime, WebviewUrl,
};

use crate::theme::{ThemeDoc, ThemeMode, ThemeState};

pub const PREVIEW_LABEL: &str = "preview";
pub const MAIN_LABEL: &str = "main";
pub const SITE_URL: &str = "https://openani.me/";

/// openani.me'nin resmi API sunucusu.
///
/// Sitenin kendi (herkese açık) istemci paketinden çıkarıldı — bundle'daki
/// `env.PUBLIC_API_LINK` değeri bu. `fetch_account_info` (lib.rs) buraya
/// sitenin kendi web istemcisinin attığı BİREBİR aynı isteği atıyor:
/// `GET {API_BASE}/user` + `Authorization: <token>` başlığı.
pub const API_BASE: &str = "https://api.openani.me";

/// Frontend henüz ölçüm göndermeden önce kullanılacak sol boşluk.
/// Gezinme şeridi (`NavRail`, 4.5rem = 72px) + editör paneli
/// (`.shell { grid-template-columns: 420px 1fr }`) ile aynı tutulmalı —
/// yoksa editöre ilk geçişte webview yanlış yerde bir kare görünüp yerine
/// oturuyor.
const FALLBACK_PANEL_WIDTH: f64 = 72.0 + 420.0;

/// Aynı gerekçeyle dikey karşılığı: başlık çubuğu (40px) + önizlemenin
/// üstündeki floating görünüm seçici şeridi (~48px). Bu değer olmadan webview
/// ilk karede ikisinin de üstüne biniyor ve `syncBounds` çalışana kadar
/// seçici görünmüyordu.
const FALLBACK_TOP_OFFSET: f64 = 88.0;

const INIT_JS: &str = include_str!("preview_init.js");

fn mode_str(mode: ThemeMode) -> &'static str {
    match mode {
        ThemeMode::System => "system",
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    }
}

/// Yer tutucuları güvenli JS string literal'leriyle değiştirir.
///
/// `serde_json::to_string` kullanmak şart: CSS içindeki tırnak, ters bölü ve
/// satır sonları elle kaçırılmaya çalışılırsa kod enjeksiyonuna dönüşür.
fn init_script(doc: &ThemeDoc) -> String {
    let css = serde_json::to_string(&doc.emit_css()).unwrap_or_else(|_| "\"\"".into());
    let mode = serde_json::to_string(mode_str(doc.mode)).unwrap_or_else(|_| "\"dark\"".into());

    let api_base = serde_json::to_string(API_BASE).unwrap_or_else(|_| "\"\"".into());

    INIT_JS
        .replace("\"__OA_INITIAL_CSS__\"", &css)
        .replace("\"__OA_INITIAL_MODE__\"", &mode)
        .replace("\"__OA_API_BASE__\"", &api_base)
}

/// Sayfa içi API köprüsünü tetikler (bkz. `preview_init.js`).
///
/// Sonuç buradan DÖNMÜYOR: `webview.eval` değer döndürmez. Sayfa işi bitince
/// `oa://account-result` olayını yayınlıyor, `lib.rs` -> `bridge_get` o olayı
/// `request_id` ile eşleştirerek bekliyor.
///
/// `path` çağıranın doğruladığı bir API yolu (`/user`, `/user/<id>/followers`
/// gibi). Serbest metin geçirilmemeli — köprü onu olduğu gibi `API_BASE`'in
/// sonuna ekliyor.
pub fn request_api(app: &AppHandle, request_id: &str, path: &str) -> Result<(), String> {
    let Some(webview) = app.get_webview(PREVIEW_LABEL) else {
        return Err("önizleme webview'i bulunamadı".into());
    };
    let id = serde_json::to_string(request_id).unwrap_or_else(|_| "\"\"".into());
    let path = serde_json::to_string(path).unwrap_or_else(|_| "\"\"".into());
    webview
        .eval(format!(
            "window.__OA_API_FETCH__ && window.__OA_API_FETCH__({id}, {path})"
        ))
        .map_err(|e| format!("hesap köprüsü çağrılamadı: {e}"))
}

/// Giriş köprüsünü tetikler (bkz. `preview_init.js` -> `__OA_API_LOGIN__`).
///
/// `request_api`'den ayrı bir fonksiyon çünkü yükü farklı: bu, sayfaya kimlik
/// bilgisi geçiriyor. `serde_json::to_string` ile kaçışlamak burada isteğe
/// bağlı değil — parola tırnak, ters bölü ya da satır sonu içerebiliyor ve
/// dize birleştirmeyle üretilen bir `eval` gövdesi o karakterlerle bozulurdu.
pub fn request_login(
    app: &AppHandle,
    request_id: &str,
    email: &str,
    password: &str,
) -> Result<(), String> {
    let Some(webview) = app.get_webview(PREVIEW_LABEL) else {
        return Err("önizleme webview'i bulunamadı".into());
    };
    let id = serde_json::to_string(request_id).unwrap_or_else(|_| "\"\"".into());
    let email = serde_json::to_string(email).unwrap_or_else(|_| "\"\"".into());
    let password = serde_json::to_string(password).unwrap_or_else(|_| "\"\"".into());
    webview
        .eval(format!(
            "window.__OA_API_LOGIN__ && window.__OA_API_LOGIN__({id}, {email}, {password})"
        ))
        .map_err(|e| format!("giriş köprüsü çağrılamadı: {e}"))
}

/// Çıkış köprüsünü tetikler (bkz. `preview_init.js` -> `__OA_API_LOGOUT__`).
pub fn request_logout(app: &AppHandle, request_id: &str) -> Result<(), String> {
    let Some(webview) = app.get_webview(PREVIEW_LABEL) else {
        return Err("önizleme webview'i bulunamadı".into());
    };
    let id = serde_json::to_string(request_id).unwrap_or_else(|_| "\"\"".into());
    webview
        .eval(format!(
            "window.__OA_API_LOGOUT__ && window.__OA_API_LOGOUT__({id})"
        ))
        .map_err(|e| format!("çıkış köprüsü çağrılamadı: {e}"))
}

/// QR (DAG) akışından bir sonraki olayı ister
/// (bkz. `preview_init.js` -> `__OA_API_DAG_NEXT__`).
///
/// Akış ilk çağrıda kendiliğinden başlıyor; ayrı bir "başlat" komutu yok.
pub fn request_qr_next(app: &AppHandle, request_id: &str) -> Result<(), String> {
    let Some(webview) = app.get_webview(PREVIEW_LABEL) else {
        return Err("önizleme webview'i bulunamadı".into());
    };
    let id = serde_json::to_string(request_id).unwrap_or_else(|_| "\"\"".into());
    webview
        .eval(format!(
            "window.__OA_API_DAG_NEXT__ && window.__OA_API_DAG_NEXT__({id})"
        ))
        .map_err(|e| format!("QR köprüsü çağrılamadı: {e}"))
}

/// QR akışını kapatır. Yanıt beklemiyor — diyalog kapanırken çağrılıyor ve
/// akışın kapanmasını beklemenin arayüze bir faydası yok.
pub fn request_qr_stop(app: &AppHandle) -> Result<(), String> {
    let Some(webview) = app.get_webview(PREVIEW_LABEL) else {
        return Ok(());
    };
    webview
        .eval("window.__OA_API_DAG_STOP__ && window.__OA_API_DAG_STOP__()")
        .map_err(|e| format!("QR köprüsü kapatılamadı: {e}"))
}
                                                
/// `add_child` başarısız olduğunda hata mesajına eklenen platform ipucu.
///
/// Linux'ta bu çağrının tek gerçek başarısızlık sebebi X11'in olmaması: child
/// webview, ana pencerenin X penceresi altına açılan ayrı bir X penceresi ve
/// wry Wayland tutamacıyla çağrıldığında `UnsupportedWindowHandle` dönüyor.
/// Ham hata metni bunu söylemiyor; kullanıcıya çözümü doğrudan veriyoruz.
#[cfg(target_os = "linux")]
const ADD_CHILD_HINT: &str = "\nLinux'ta önizleme X11 gerektiriyor. \
Oturum Wayland ise uygulamayı `GDK_BACKEND=x11` ile başlatın — normalde bunu \
uygulama kendisi ayarlıyor, ama değişkeni elle verdiyseniz üzerine yazılmıyor.";

#[cfg(not(target_os = "linux"))]
const ADD_CHILD_HINT: &str = "";

/// Ana pencereye önizleme webview'ini ekler.
pub fn create(app: &AppHandle, doc: &ThemeDoc) -> Result<(), Box<dyn std::error::Error>> {
    let window = app
        .get_window(MAIN_LABEL)
        .ok_or("ana pencere ('main') bulunamadı")?;

    let scale = window.scale_factor()?;
    let inner: LogicalSize<f64> = window.inner_size()?.to_logical(scale);

    let x = FALLBACK_PANEL_WIDTH.min(inner.width);
    let w = (inner.width - x).max(0.0);
    let y = FALLBACK_TOP_OFFSET.min(inner.height);
    let h = (inner.height - y).max(0.0);

    let url: url::Url = SITE_URL.parse()?;

    let builder = WebviewBuilder::new(PREVIEW_LABEL, WebviewUrl::External(url))
        .initialization_script(init_script(doc))
        // Sayfa geçişlerinde (ana sayfa -> anime detay -> oynatıcı) temayı
        // yeniden basar. initialization_script her navigasyonda çalışsa da
        // içindeki taslak webview kurulduğu andaki hâli taşır; burada Rust'ın
        // güncel state'i yeniden uygulanarak tema kaybı önleniyor.
        .on_page_load(|webview, payload| {
            if !matches!(payload.event(), PageLoadEvent::Finished) {
                return;
            }
            let state = webview.app_handle().state::<ThemeState>();
            let Ok(doc) = state.0.lock() else { return };
            eval_apply_css(&webview, &doc.emit_css());
            eval_apply_mode(&webview, doc.mode);
        });

    let webview = window
        .add_child(builder, LogicalPosition::new(x, y), LogicalSize::new(w, h))
        .map_err(|e| format!("önizleme webview'i oluşturulamadı: {e}{ADD_CHILD_HINT}"))?;

    // Uygulama artık ana ekranla açılıyor, editörle değil. Önizleme baştan
    // gizli olmasa openani.me bir an ana ekranın üstünde görünürdü — native
    // child webview her zaman host içeriğinin üstüne çizildiği için bunu CSS
    // ile örtmek mümkün değil. Sayfa gizliyken de yüklenmeye devam ediyor,
    // dolayısıyla editöre geçildiğinde site hazır oluyor.
    webview.hide()?;

    Ok(())
}

/// Önizlemedeki temayı anında günceller. Reload yok, re-render yok —
/// yalnızca `<style themeStyle>` etiketinin textContent'i değişir.
fn eval_apply_css<R: Runtime>(webview: &Webview<R>, css: &str) {
    let payload = serde_json::to_string(css).unwrap_or_else(|_| "\"\"".into());
    let _ = webview.eval(format!(
        "window.__OA_THEME_APPLY__ && window.__OA_THEME_APPLY__({payload})"
    ));
}

fn eval_apply_mode<R: Runtime>(webview: &Webview<R>, mode: ThemeMode) {
    let payload = serde_json::to_string(mode_str(mode)).unwrap_or_else(|_| "\"dark\"".into());
    let _ = webview.eval(format!(
        "window.__OA_THEME_MODE__ && window.__OA_THEME_MODE__({payload})"
    ));
}

pub fn apply_css(app: &AppHandle, css: &str) {
    if let Some(webview) = app.get_webview(PREVIEW_LABEL) {
        eval_apply_css(&webview, css);
    }
}

/// Açık/koyu/sistem modunu önizlemeye uygular.
pub fn apply_mode(app: &AppHandle, mode: ThemeMode) {
    if let Some(webview) = app.get_webview(PREVIEW_LABEL) {
        eval_apply_mode(&webview, mode);
    }
}

/// Editör panelinin ölçtüğü boşluğa webview'i oturtur.
///
/// `add_child` ile eklenen webview pencere ile birlikte otomatik yeniden
/// boyutlanmaz; bu yüzden frontend bir ResizeObserver ile gerçek boşluğu ölçüp
/// buraya bildiriyor. Splitter sürüklemesi de aynı yolu kullanır.
pub fn set_bounds(app: &AppHandle, x: f64, y: f64, width: f64, height: f64) -> tauri::Result<()> {
    let Some(webview) = app.get_webview(PREVIEW_LABEL) else {
        return Ok(());
    };

    // Tam sayıya yuvarla. Aksi hâlde konum ve boyut mantıksal->fiziksel piksele
    // BAĞIMSIZ olarak yuvarlanıyor ve kesirli değerlerde (ör. genişlik
    // 1063.328) sağ kenar bir piksel şaşabiliyor.
    let x = x.round();
    let y = y.round();
    let width = width.max(1.0).round();
    let height = height.max(1.0).round();

    let previous = *last_bounds().lock().unwrap_or_else(|e| e.into_inner());

    // Konum ve boyut TEK çağrıda gidiyor. Bu bir mikro-optimizasyon değil,
    // Linux'ta doğruluk şartı:
    //
    // `set_position` ve `set_size` ayrı ayrı çağrıldığında Tauri ikisini de
    // oku-değiştir-yaz olarak işliyor — webview'in O ANKİ dikdörtgenini geri
    // okuyup yalnızca bir alanını değiştiriyor
    // (tauri-runtime-wry -> `WebviewMessage::SetSize` / `SetPosition`).
    // WebKitGTK tarafında o geri okuma `XGetWindowAttributes` ile yapılıyor ve
    // sonucu FİZİKSEL piksel olduğu hâlde `LogicalPosition`/`LogicalSize`
    // olarak etiketleniyor (wry -> `webkitgtk::InnerWebView::bounds`). Sonuç:
    //   * DPI ölçeği 1'den farklıysa geri okunan alan her turda ölçekle
    //     çarpılıyor — webview büyüyerek editörün üstünü kaplıyor,
    //   * ölçek 1 olsa bile X'in geometri değişiklikleri asenkron olduğu için
    //     ilk çağrının etkisi ikincinin geri okumasına yetişemiyor ve webview
    //     eski konumuna geri sıçrıyor.
    //
    // `set_bounds` iki alanı da bizim değerlerimizle yazıyor, hiçbir şey geri
    // okumuyor. Yan faydası: konum ile boyut arasında webview'in bir an
    // geçersiz bir dikdörtgene sahip olduğu ara durum da ortadan kalkıyor —
    // eskiden burada bunun için sıralama hilesi vardı.
    webview.set_bounds(Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    })?;

    let size_changed = previous.map(|(_, _, pw, ph)| pw != width || ph != height).unwrap_or(true);

    // Eşiğin hangi yakasında olduğumuz değişti mi? Karşılaştırmayı bounds'u
    // kaydetmeden önce yapıyoruz; `previous` bir sonraki satırda eziliyor.
    let crossed_breakpoint = previous
        .map(|(_, _, pw, _)| is_mobile_width(pw) != is_mobile_width(width))
        .unwrap_or(false);

    *last_bounds().lock().unwrap_or_else(|e| e.into_inner()) = Some((x, y, width, height));

    if size_changed {
        // Site mobil/masaüstü ayrımını `window.innerWidth` + bir `resize`
        // dinleyicisiyle yapıyor (bkz. PLAN.md §0.4 civarındaki bundle analizi).
        //
        // Buradaki incelik: olayı `set_size`'dan hemen sonra göndermek işe yaramaz,
        // çünkü WebView2 yeni bounds'u henüz uygulamamış olabiliyor; site eski
        // genişliği okuyup mobil düzende takılı kalıyor ve öğeler geniş çerçevede
        // üst üste biniyor. Bu yüzden olayı sayfanın kendi içinde iki kare
        // sonrasına erteliyoruz — o noktada yerleşim kesinlikle tazelenmiş olur.
        soft_resize(&webview, crossed_breakpoint);
    }

    Ok(())
}

/// Sitenin mobil/masaüstü ayrımını yaptığı eşik.
///
/// Bundle'daki mobil algılama fonksiyonu tam olarak şunu yapıyor:
/// `store.set(window.innerWidth < 768)` + bir `resize` dinleyicisi. Yani 768,
/// sitenin *kendi* sabiti; bizim seçtiğimiz bir değer değil. Viewport
/// genişliklerimiz (Mobil 390 / Tablet 834 / Masaüstü = kalan alan) bu eşiğin
/// iki yakasına düşüyor.
const SITE_MOBILE_BREAKPOINT: f64 = 768.0;

fn is_mobile_width(width: f64) -> bool {
    width < SITE_MOBILE_BREAKPOINT
}

/// Boyut değişimini sayfaya bildirir; mobil eşiği geçildiyse yeniden yükletir.
///
/// ## Eşik geçilmediyse — kesintisiz
///
/// Tablet <-> Masaüstü geçişi ve splitter sürüklemesi sitenin mobil dalını
/// değiştirmiyor; yalnızca `resize` + `orientationchange` gönderiliyor.
///
/// ## Eşik geçildiyse — yeniden yükleme
///
/// Sitenin masaüstü yerleşimi açılışta kurulan bir yükseklik zincirine bağlı:
///
/// ```text
/// html[data-overlayscrollbars~=body] > body { height: 100% }
///   #page        { display:flex; height:100%; overflow:hidden }
///     #page > div { display:flex; height:100% }
///       .sidebar  { min-width:4.5rem; height:100% }
/// ```
///
/// Kaydırmanın sahibi OverlayScrollbars ve o, sayfa açılırken o anki genişliğe
/// göre kuruluyor. Mobil eşiği geçilince site `body.mobile-patches`
/// (`height:100vh; overflow-y:auto`) ile native kaydırmaya dönüyor; geri
/// dönüldüğünde zincirin tepesi yeniden kurulmuyor. Body'nin kesin yüksekliği
/// kalmayınca `#page{height:100%}` `auto`ya düşüyor, `.sidebar` belge boyu
/// uzuyor ve `justify-content:space-between` yüzünden ortası boşalıyor —
/// kullanıcının gördüğü "sol menü kayboldu" tam olarak bu.
///
/// ### Neden CSS enjeksiyonu değil
///
/// Burada bir zamanlar `html,body{overflow-y:auto!important;height:auto!important}`
/// vardı ve `height:auto` zinciri onarmak yerine KIRIYORDU; hatanın bir kısmı
/// doğrudan o satırdan geliyordu. Doğru yükseklikleri (`height:100%`) zorla
/// geri yazmak da yetmiyor: OverlayScrollbars viewport elemanı yokken
/// `#page{overflow:hidden}` ile sayfada kaydırılabilir hiçbir şey kalmıyor.
/// Yani yerleşim ve kaydırma dışarıdan tek bir stille tutarlı biçimde
/// kurulamıyor; sitenin kendi önyükleme zinciri gerekiyor ve onu tetiklemenin
/// tek yolu yeniden yükleme.
///
/// Bedeli yalnızca Mobil <-> (Tablet|Masaüstü) geçişinde görünen kısa bir
/// splash ekranı. `preview_init.js` her navigasyonda yeniden enjekte edildiği,
/// `on_page_load` da temayı geri bastığı için tema ve oturum korunuyor.
fn soft_resize<R: Runtime>(webview: &Webview<R>, crossed_breakpoint: bool) {
    // Yer tutucuyu `init_script`'teki kalıpla değiştiriyoruz. JS'i `format!`
    // içine gömmek her süslü parantezi ikilemeyi gerektirirdi — bu betikte
    // onlarca tane var ve tek bir hata sessizce bozuk JS üretirdi.
    let script = RESIZE_JS.replace(
        "\"__OA_REPAIR__\"",
        if crossed_breakpoint { "true" } else { "false" },
    );
    let _ = webview.eval(script);
}

const RESIZE_JS: &str = include_str!("preview_resize.js");

/// Son uygulanan bounds — yalnızca yukarıdaki sıralama kararı için tutuluyor.
fn last_bounds() -> &'static Mutex<Option<(f64, f64, f64, f64)>> {
    static LAST: OnceLock<Mutex<Option<(f64, f64, f64, f64)>>> = OnceLock::new();
    LAST.get_or_init(|| Mutex::new(None))
}

/// Önizlemeyi gizler / geri gösterir.
///
/// Ana ekran ve ayarlar görünümünde önizlemeye yer yok. Native child webview
/// HER ZAMAN host sayfanın içeriğinin üstüne çizildiği için (bkz. `+page.svelte`
/// içindeki not) onu sadece CSS ile örtmek imkânsız — gerçekten gizlemek
/// gerekiyor.
pub fn set_visible(app: &AppHandle, visible: bool) -> tauri::Result<()> {
    let Some(webview) = app.get_webview(PREVIEW_LABEL) else {
        return Ok(());
    };
    if visible {
        webview.show()
    } else {
        webview.hide()
    }
}

pub fn navigate(app: &AppHandle, url: &str) -> Result<(), String> {
    if let Some(webview) = app.get_webview(PREVIEW_LABEL) {
        let parsed: url::Url = url.parse().map_err(|e| format!("geçersiz URL: {e}"))?;
        webview.navigate(parsed).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Viewport genişliklerimizin eşiğin doğru yakalarına düştüğünü sabitliyoruz.
    /// Mobil <-> (Tablet|Masaüstü) geçişi eşiği geçer; Tablet <-> Masaüstü
    /// geçmez. Eşik geçişi artık reload değil, yalnızca kaydırma onarımının
    /// denenip denenmeyeceğini belirliyor.
    #[test]
    fn breakpoint_crossing_only_on_mobile_side() {
        const MOBILE: f64 = 390.0;
        const TABLET: f64 = 834.0;
        const DESKTOP: f64 = 1116.0;

        assert!(is_mobile_width(MOBILE));
        assert!(!is_mobile_width(TABLET));
        assert!(!is_mobile_width(DESKTOP));

        let crossed = |a: f64, b: f64| is_mobile_width(a) != is_mobile_width(b);

        // Bozulan yönler (her iki yönde de).
        assert!(crossed(MOBILE, TABLET));
        assert!(crossed(TABLET, MOBILE));
        assert!(crossed(MOBILE, DESKTOP));
        assert!(crossed(DESKTOP, MOBILE));

        // Sorunsuz çalışan yön — gereksiz reload olmamalı.
        assert!(!crossed(TABLET, DESKTOP));
        assert!(!crossed(DESKTOP, TABLET));

        // Eşiğin tam kendisi masaüstü sayılır (site `< 768` diyor).
        assert!(is_mobile_width(767.0));
        assert!(!is_mobile_width(768.0));
    }

    /// Eşik GEÇİLMEDİYSE sayfa yeniden yüklenmemeli.
    ///
    /// Tablet <-> Masaüstü geçişi, splitter sürüklemesi ve pencere boyutlandırma
    /// hep bu yoldan geçiyor. Oradan bir yeniden yükleme çıksaydı önizleme
    /// kullanılamaz hâle gelirdi — sürüklemenin her karesi splash ekranı olurdu.
    #[test]
    fn no_reload_when_threshold_not_crossed() {
        let same = RESIZE_JS.replace("\"__OA_REPAIR__\"", "false");
        assert!(same.contains("if (!CROSSED) return;"));

        // Reload o erken çıkışın ARDINDA kalmalı; önüne geçerse koşul anlamsızlaşır.
        let before = same
            .split("window.location.reload")
            .next()
            .expect("reload çağrısı bulunamadı");
        assert!(
            before.contains("if (!CROSSED) return;"),
            "yeniden yükleme eşik kontrolünün arkasında olmalı"
        );
    }

    /// Eşik geçildiyse sitenin kendi önyükleme zinciri yeniden çalışmalı.
    ///
    /// Site masaüstü yerleşimini açılışta kurduğu kaydırma altyapısına
    /// dayandırıyor (`html[data-overlayscrollbars~=body] > body { height:100% }`
    /// -> `#page` -> `.sidebar`). Eşik geçişinde o zincirin tepesi yeniden
    /// kurulmuyor ve yan menü belge boyu uzayıp görünmez oluyor. Zinciri
    /// dışarıdan tetiklemenin tek yolu yeniden yükleme.
    #[test]
    fn page_reloads_when_threshold_crossed() {
        let crossed = RESIZE_JS.replace("\"__OA_REPAIR__\"", "true");
        assert_eq!(crossed.matches("window.location.reload").count(), 1);
        // Sitenin kendi mekanizması da tetiklenmeli — yeniden yükleme
        // beklenirken düzenin bir kare doğru kalması buna bağlı.
        assert!(crossed.contains("new Event(\"resize\")"));
    }

    /// Sayfaya ARTIK stil enjekte edilmiyor.
    ///
    /// Bu bir gerileme koruması. Betik bir zamanlar
    /// `html,body{overflow-y:auto!important;height:auto!important}` basıyordu;
    /// `height:auto`, sitenin yüzdelik yükseklik zincirini onarmak yerine
    /// kırıyor ve `.sidebar` çöküyordu. Sayfanın yerleşimine dışarıdan
    /// karışmanın doğru yolu yok — o yüzden hiç karışmıyoruz.
    #[test]
    fn no_style_injected_into_page() {
        for crossed in [true, false] {
            let script = RESIZE_JS.replace(
                "\"__OA_REPAIR__\"",
                if crossed { "true" } else { "false" },
            );
            for forbidden in ["createElement(\"style\")", "textContent", "appendChild"] {
                assert!(
                    !script.contains(forbidden),
                    "önizleme betiği sayfaya stil enjekte etmemeli (`{forbidden}` bulundu)"
                );
            }
        }
    }

    #[test]
    fn resize_script_replaces_placeholder() {
        let crossed = RESIZE_JS.replace("\"__OA_REPAIR__\"", "true");
        let same = RESIZE_JS.replace("\"__OA_REPAIR__\"", "false");

        assert!(!crossed.contains("__OA_REPAIR__"), "yer tutucu kalmamalı");
        assert!(!same.contains("__OA_REPAIR__"), "yer tutucu kalmamalı");
        assert!(crossed.contains("var CROSSED = true;"));
        assert!(same.contains("var CROSSED = false;"));
    }

    #[test]
    fn init_script_replaces_placeholders() {
        let doc = ThemeDoc {
            accent: [280.0, 80.0, 50.0],
            ..Default::default()
        };
        let script = init_script(&doc);
        assert!(!script.contains("__OA_INITIAL_CSS__"));
        assert!(!script.contains("__OA_INITIAL_MODE__"));
        assert!(script.contains("--fds-accent-base"));
        assert!(script.contains("\"dark\""));
    }

    #[test]
    fn init_script_survives_quoted_css() {
        // Ham CSS içine tırnak ve ters bölü koyup kaçışın doğru olduğunu sınıyoruz.
        let doc = ThemeDoc {
            raw_css: "body::after { content: \"\\\"tehlike\\\"\"; }".into(),
            ..Default::default()
        };
        let script = init_script(&doc);
        // Yer tutucu satırı hâlâ tek bir geçerli JS string ataması olmalı.
        let line = script
            .lines()
            .find(|l| l.contains("var INITIAL_CSS ="))
            .expect("INITIAL_CSS satırı bulunamadı");
        assert!(line.ends_with(';'));
        // Kaçırılmamış ham tırnak sızmamalı.
        assert!(!line.contains("content: \""));
    }
}
