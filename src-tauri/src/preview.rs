//! Önizleme child webview'i: oluşturma, konumlandırma ve tema enjeksiyonu.
//!
//! openani.me `x-frame-options: SAMEORIGIN` gönderdiği için iframe kullanılamaz
//! (PLAN.md §0.5). Bu yüzden site, ana pencereye `Window::add_child` ile eklenen
//! gerçek bir webview içinde gösteriliyor (`unstable` feature gerekir).

use std::sync::{Mutex, OnceLock};

use tauri::{
    webview::{PageLoadEvent, Webview, WebviewBuilder},
    AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewUrl,
};

use crate::theme::{ThemeDoc, ThemeMode, ThemeState};

pub const PREVIEW_LABEL: &str = "preview";
pub const MAIN_LABEL: &str = "main";
pub const SITE_URL: &str = "https://openani.me/";

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

    INIT_JS
        .replace("\"__OA_INITIAL_CSS__\"", &css)
        .replace("\"__OA_INITIAL_MODE__\"", &mode)
}

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

    let webview = window.add_child(builder, LogicalPosition::new(x, y), LogicalSize::new(w, h))?;

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

    // Konum ve boyut iki ayrı çağrı olduğu için aralarında webview bir an
    // geçersiz bir dikdörtgene sahip oluyor. Sırayı, ara durumun asla
    // pencerenin dışına taşmayacağı şekilde seçiyoruz:
    //   sağa kayıyorsa (viewport daralıyor) -> önce küçült, sonra taşı
    //   sola kayıyorsa (viewport genişliyor) -> önce taşı, sonra büyüt
    let moving_right = previous.map(|(px, ..)| x > px).unwrap_or(false);

    if moving_right {
        webview.set_size(LogicalSize::new(width, height))?;
        webview.set_position(LogicalPosition::new(x, y))?;
    } else {
        webview.set_position(LogicalPosition::new(x, y))?;
        webview.set_size(LogicalSize::new(width, height))?;
    }

    let size_changed = previous.map(|(_, _, pw, ph)| pw != width || ph != height).unwrap_or(true);

    // Eşiğin hangi yakasında olduğumuz değişti mi? Karşılaştırmayı bounds'u
    // kaydetmeden ÖNCE yapıyoruz; `previous` bir sonraki satırda eziliyor.
    let crossed_breakpoint = previous
        .map(|(_, _, pw, _)| is_mobile_width(pw) != is_mobile_width(width))
        .unwrap_or(false);

    *last_bounds().lock().unwrap_or_else(|e| e.into_inner()) = Some((x, y, width, height));

    if size_changed {
        // Site mobil/masaüstü ayrımını `window.innerWidth` + bir `resize`
        // dinleyicisiyle yapıyor (bkz. PLAN.md §0.4 civarındaki bundle analizi).
        //
        // Buradaki incelik: olayı `set_size`'dan HEMEN sonra göndermek işe yaramaz,
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

/// Boyut değişimini sayfaya YUMUŞAK biçimde bildirir — reload YOK.
///
/// Burada eskiden `location.reload()` vardı: mobil eşiği geçilince sitenin
/// kaydırma altyapısı yeniden kurulmadığı için sayfa kilitleniyordu ve reload
/// tek çare gibi görünmüştü. Ama reload, tam sayfa yeniden yükleme + splash
/// ekranı demek; görünüm değiştirmeyi ağır ve kesintili yapıyordu.
///
/// Yerine üç adımlı, kesintisiz bir yol:
///
/// 1. **`resize` + `orientationchange`** — sitenin kendi mekanizması. Mobil
///    algılama `window.innerWidth < 768` + bir `resize` dinleyicisi; bunu
///    tetiklemek DOM'un doğru dala geçmesi için yeterli. Olayı iki kare
///    sonrasına erteliyoruz, çünkü WebView2 yeni bounds'u hemen uygulamıyor.
/// 2. **`matchMedia` uyandırma** — bazı bileşenler genişliği `matchMedia` ile
///    izliyor. Sorgular otomatik değerlendirilir ama dinleyicilerin sırayla
///    çalışabilmesi için ek bir kare bırakıyoruz.
/// 3. **Kaydırma onarımı (yalnızca eşik geçildiyse)** — site `html, body`'ye
///    `overflow: hidden` verip kaydırmayı OverlayScrollbars'ın viewport
///    elemanına devrediyor. Eşik geçişinde o eleman yeniden kurulmazsa
///    kaydırılabilir hiçbir şey kalmıyor. Bunu ÖLÇEREK tespit ediyoruz
///    (içerik görünümden uzun mu, kaydırılabilir kap var mı) ve yalnızca
///    gerçekten bozuksa native kaydırmayı geri açan küçük bir stil enjekte
///    ediyoruz. Sorun yoksa hiçbir şey yapılmıyor ve stil geri alınıyor.
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
    fn eslik_gecisi_yalnizca_mobil_tarafinda() {
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

    /// Görünüm değişimi ARTIK sayfayı yeniden yüklememeli. Bu test, reload'un
    /// bir daha sessizce geri sızmamasını garanti altına alıyor.
    #[test]
    fn gorunum_gecisi_reload_tetiklemez() {
        for crossed in [true, false] {
            let script = RESIZE_JS.replace(
                "\"__OA_REPAIR__\"",
                if crossed { "true" } else { "false" },
            );
            assert!(
                !script.contains("location.reload"),
                "görünüm geçişinde tam sayfa yeniden yükleme olmamalı"
            );
            // Sitenin kendi mekanizması tetiklenmeli.
            assert!(script.contains("new Event(\"resize\")"));
        }
    }

    #[test]
    fn resize_betigi_yer_tutucuyu_degistirir() {
        let crossed = RESIZE_JS.replace("\"__OA_REPAIR__\"", "true");
        let same = RESIZE_JS.replace("\"__OA_REPAIR__\"", "false");

        assert!(!crossed.contains("__OA_REPAIR__"), "yer tutucu kalmamalı");
        assert!(!same.contains("__OA_REPAIR__"), "yer tutucu kalmamalı");
        assert!(crossed.contains("var REPAIR = true;"));
        assert!(same.contains("var REPAIR = false;"));
    }

    #[test]
    fn init_script_yer_tutuculari_degistirir() {
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
    fn init_script_tirnakli_css_ile_bozulmaz() {
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
