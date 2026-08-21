mod discord;
mod easter_egg;
mod preview;
mod projects;
mod theme;
mod updater;

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
    known_selectors: Option<Vec<String>>,
) -> Result<ThemeDoc, String> {
    let known = known_selectors.unwrap_or_default();
    let (doc, mode) = {
        let mut current = state.0.lock().map_err(|e| e.to_string())?;
        let parsed = parse_css(&text, &known, &current);
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

/// Önizlemedeki `oa://account-result` köprü olayının adı (bkz. `preview_init.js`).
const ACCOUNT_EVENT: &str = "oa://account-result";

/// Sayfadan dönen köprü yanıtı.
///
/// `stage` yanıtın hangi noktada üretildiğini söylüyor; `status`/`body` yalnızca
/// `"done"` aşamasında dolu.
#[derive(serde::Deserialize)]
struct AccountReply {
    id: String,
    stage: String,
    #[serde(default)]
    status: Option<u16>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    message: Option<String>,
    /// Yalnızca giriş köprüsü doldurur: istek başarılı mı?
    #[serde(default)]
    ok: Option<bool>,
    /// Yalnızca giriş köprüsü doldurur: hesabın e-postası doğrulanmış mı?
    #[serde(default)]
    verified: Option<bool>,
    /// Yalnızca QR köprüsü doldurur: "qr" | "success" | "error" | "idle".
    #[serde(default)]
    kind: Option<String>,
    /// Yalnızca QR köprüsü doldurur: gösterilecek QR görselinin kaynağı.
    #[serde(default)]
    image: Option<String>,
}

/// Köprüyü tetikler ve `request_id` ile eşleşen yanıtı bekler.
///
/// `bridge_get` ile `account_login` arasındaki TEK ortak parça bu: benzersiz
/// bir istek kimliği üretmek, dinleyiciyi kurmak, tetiklemek, zaman aşımıyla
/// beklemek. İkisinin ayrıştığı yer `stage` yorumlaması — o yüzden burası ham
/// `AccountReply` döndürüyor, karar çağırana ait.
///
/// `trigger` bir kapanış (closure) çünkü iki yolun sayfaya geçirdiği argümanlar
/// farklı (biri API yolu, diğeri kimlik bilgisi) ve o farkı bu fonksiyonun
/// bilmesine gerek yok.
///
/// `timeout_secs` çağrıya göre değişiyor: normal istekler için 30 sn yeter
/// (köprü geçidin kurulmasını ~10 sn, 401 sonrası yeniden denemeyi ~2.5 sn
/// bekliyor), QR akışı ise kullanıcının telefonuyla kod okutmasını beklediği
/// için daha uzun soluklu.
async fn bridge_await<F>(
    app: &AppHandle,
    timeout_secs: u64,
    trigger: F,
) -> Result<AccountReply, String>
where
    F: FnOnce(&AppHandle, &str) -> Result<(), String>,
{
    use tauri::Listener;

    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let request_id = format!(
        "acc-{}",
        NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    );

    // Dinleyici, köprü tetiklenmeden ÖNCE kurulmalı: sayfa çerezi ve geçit
    // token'ı hazırsa yanıt aynı tur içinde dönebiliyor.
    let (tx, rx) = tokio::sync::oneshot::channel::<AccountReply>();
    let slot = std::sync::Mutex::new(Some(tx));
    let wanted = request_id.clone();
    let listener = app.listen_any(ACCOUNT_EVENT, move |event| {
        let Ok(reply) = serde_json::from_str::<AccountReply>(event.payload()) else {
            return;
        };
        // Eşzamanlı iki istek olabilir (kart açılışı + "Yenile"); her dinleyici
        // yalnızca kendi isteğinin yanıtını alsın.
        if reply.id != wanted {
            return;
        }
        if let Ok(mut guard) = slot.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(reply);
            }
        }
    });

    if let Err(e) = trigger(app, &request_id) {
        app.unlisten(listener);
        return Err(e);
    }

    let outcome = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), rx).await;
    app.unlisten(listener);

    match outcome {
        Ok(Ok(reply)) => Ok(reply),
        // Kanal düştü: dinleyici kaldırıldı ama yanıt gelmedi.
        Ok(Err(_)) => Err("hesap köprüsünden yanıt alınamadı".into()),
        Err(_) => Err(
            "önizleme yanıt vermedi — openani.me sayfasının yüklenmesini bekleyip tekrar deneyin"
                .into(),
        ),
    }
}

/// Önizleme sayfası üzerinden openani.me API'sinden bir yol çeker.
///
/// **İstek buradan atılmıyor, önizleme sayfasının İÇİNDEN atılıyor.** Sebebi
/// `preview_init.js`'deki köprünün başındaki uzun yorumda: `api.openani.me`
/// "Vanguard" geçidinin arkasında ve `Gateway-Token` başlığı olmadan — kimlik
/// doğrulama gerektirmeyen uç noktalar dâhil — her istek 401 dönüyor. O başlık
/// sitenin kendi `window.fetch` yaması tarafından ekleniyor, değeri
/// `/osc.wasm` ile 35 saniyede bir yeniden imzalanıyor. Yani token ne
/// kopyalanabiliyor ne de yeniden üretilebiliyor; istek sayfanın yamalı
/// `fetch`'iyle atılmak zorunda.
///
/// Bu fonksiyonun tek işi: köprüyü tetiklemek, `request_id` ile eşleşen olayı
/// beklemek, gelen ham gövdeyi JSON'a çevirmek. `path`'i ÜRETEN ve doğrulayan
/// çağıran taraf — bu köprü genel amaçlı bir API istemcisine dönüşmesin diye
/// yalnızca aşağıdaki iki komut onu çağırıyor.
///
/// Zaman aşımı 30 sn: sayfa daha yeni açıldıysa köprü wasm geçidinin kurulmasını
/// ~10 sn, 401 gelirse yeniden denemeyi ~2.5 sn bekliyor; ikisi de bu bütçenin
/// içinde kalıyor.
async fn bridge_get(app: &AppHandle, path: &str) -> Result<serde_json::Value, String> {
    let reply = bridge_await(app, 30, |app, id| preview::request_api(app, id, path)).await?;

    match reply.stage.as_str() {
        "no-session" => Err("openani.me'de oturum açık değil".into()),
        "no-gateway" => Err(
            "openani.me'nin güvenlik geçidi (Vanguard) henüz kurulmadı — önizlemeyi açıp \
             sayfanın yüklenmesini bekleyin"
                .into(),
        ),
        "error" => Err(format!(
            "hesap bilgisi alınamadı: {}",
            reply.message.unwrap_or_else(|| "bilinmeyen hata".into())
        )),
        // Uygulama uzun süre arka planda kaldıysa (Chromium'un arka plan
        // zamanlayıcı kısıtlaması sitenin 35sn/5dk'lık geçit yenileme
        // döngülerini geciktirir) köprü iki denemeden sonra da 401/400
        // görürse sayfayı kendi kendine yeniliyor (bkz. `preview_init.js`).
        // O yenileme birkaç saniye sürüyor; bu arada net bir mesaj veriyoruz.
        "reloading" => Err(
            "oturum bayatlamış — önizleme kendini yeniliyor, birkaç saniye içinde tekrar deneyin"
                .into(),
        ),
        "done" => {
            let status = reply.status.unwrap_or(0);
            let body = reply.body.unwrap_or_default();
            match status {
                // 401/400 buraya artık ulaşmıyor: köprü ikisini de "reloading"
                // ile ele alıyor. Geriye yalnızca beklenmeyen durum kodları
                // kalıyor (ör. 5xx).
                200..=299 => serde_json::from_str::<serde_json::Value>(&body)
                    .map_err(|e| format!("yanıt ayrıştırılamadı: {e}")),
                _ => Err(format!("sunucu {status} döndürdü")),
            }
        }
        other => Err(format!("beklenmeyen köprü yanıtı: {other}")),
    }
}

/// Bir giriş denemesinin sonucu.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginOutcome {
    /// Hesabın e-postası doğrulanmış mı? `false` ise oturum açıldı ama
    /// kullanıcının siteden doğrulama adımını tamamlaması gerekiyor.
    verified: bool,
}

/// Kullanıcı adına openani.me'de oturum açar.
///
/// ## İstek neden yine önizlemeden çıkıyor
///
/// Vanguard geçidi burada da geçerli: `POST /user/auth` `Gateway-Token`
/// başlığı olmadan 401 döner ve o başlık yalnızca sayfanın `/osc.wasm` ile
/// imzalanan yamalı `fetch`'i tarafından eklenebiliyor (ayrıntı için
/// `preview_init.js`'in başındaki yorum). Yani "uygulama içi giriş formu",
/// isteği Rust'tan atmak anlamına GELMİYOR — form yereldeyken taşıma yine
/// sayfanın üzerinden gidiyor.
///
/// ## Kimlik bilgisi burada saklanmıyor
///
/// `email`/`password` yalnızca bu çağrının ömrü boyunca yaşıyor: köprüye
/// geçiriliyor, sayfa isteği atıyor, fonksiyon dönüyor. Hiçbir yere
/// yazılmıyor, tekrar kullanılmıyor. Dönen `token`/`refreshToken` ise Rust'a
/// hiç ULAŞMIYOR — sayfa onları doğrudan çereze yazıyor (sitenin kendi giriş
/// modalının yaptığının aynısı) ve uygulama oturumu yalnızca
/// `preview_login_state` ile, çerezin VARLIĞINA bakarak öğreniyor.
#[tauri::command]
async fn account_login(
    app: AppHandle,
    email: String,
    password: String,
) -> Result<LoginOutcome, String> {
    // Boş alanla sunucuya gitmenin anlamı yok; arayüz zaten engelliyor ama
    // komut kendi başına da savunulabilir olmalı.
    let email = email.trim().to_string();
    if email.is_empty() || password.is_empty() {
        return Err("e-posta ve parola gerekli".into());
    }

    let reply = bridge_await(&app, 30, |app, id| {
        preview::request_login(app, id, &email, &password)
    })
    .await?;

    match reply.stage.as_str() {
        "no-gateway" => Err(
            "openani.me'nin güvenlik geçidi (Vanguard) henüz kurulmadı — önizlemeyi açıp              sayfanın yüklenmesini bekleyin"
                .into(),
        ),
        // Köprü, geçit oturumu bayatladığı için sayfayı yeniliyor (bkz.
        // `preview_init.js` -> `gatewayRejected`). Parola YENİDEN GÖNDERİLMEDİ;
        // kullanıcı birkaç saniye sonra tekrar denemeli.
        "reloading" => Err(
            "openani.me oturumu tazeleniyor — birkaç saniye bekleyip tekrar deneyin".into(),
        ),
        "error" => Err(format!(
            "giriş yapılamadı: {}",
            reply.message.unwrap_or_else(|| "bilinmeyen hata".into())
        )),
        "done" => {
            if reply.ok == Some(true) {
                Ok(LoginOutcome {
                    verified: reply.verified.unwrap_or(true),
                })
            } else {
                // Sunucunun kendi mesajı (İngilizce) geliyor; Türkçeleştirmeyi
                // arayüz yapıyor (bkz. `$lib/account.ts` -> `loginErrorText`),
                // çünkü eşleştirme tablosu sitenin çeviri dosyasından geliyor
                // ve orada güncel tutmak daha kolay.
                Err(reply.message.unwrap_or_else(|| "giriş reddedildi".into()))
            }
        }
        other => Err(format!("beklenmeyen köprü yanıtı: {other}")),
    }
}

/// QR akışından bir sonraki olay.
///
/// `kind`:
///   - `"qr"`      -> `image` gösterilecek QR kaynağı (kod kısa aralıklarla
///                    yenilendiği için birden çok kez gelir)
///   - `"success"` -> oturum açıldı; `verified` e-posta doğrulanmış mı
///   - `"error"`   -> `message` gösterilecek hata
///   - `"idle"`    -> bu turda olay yok; arayüz tekrar sormalı
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct QrEvent {
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// QR ile giriş akışından bir sonraki olayı bekler.
///
/// Akış ilk çağrıda kendiliğinden açılıyor. Arayüz bunu bir DÖNGÜDE çağırıyor:
/// `"qr"` gelince görseli tazeliyor, `"idle"` gelince yeniden soruyor,
/// `"success"`/`"error"` ile döngüyü bitiriyor.
///
/// Zaman aşımı 40 sn, sayfa tarafındaki 25 sn'lik yoklamadan uzun — böylece
/// olay yokken bile yanıtı sayfa veriyor (`"idle"`) ve arayüz gerçek bir zaman
/// aşımı hatası görmüyor. Kullanıcının kodu okutması dakikalar sürebileceği
/// için akışın kendisinin bir süre sınırı YOK; `account_qr_stop` ile kapanıyor.
#[tauri::command]
async fn account_qr_next(app: AppHandle) -> Result<QrEvent, String> {
    let reply = bridge_await(&app, 40, preview::request_qr_next).await?;

    match reply.stage.as_str() {
        "done" => {
            let kind = reply.kind.unwrap_or_else(|| "idle".into());
            Ok(QrEvent {
                image: reply.image,
                verified: reply.verified,
                message: reply.message,
                kind,
            })
        }
        "no-gateway" => Err(
            "openani.me'nin güvenlik geçidi (Vanguard) henüz kurulmadı — birkaç saniye              bekleyip tekrar deneyin"
                .into(),
        ),
        "error" => Err(format!(
            "QR akışı başarısız: {}",
            reply.message.unwrap_or_else(|| "bilinmeyen hata".into())
        )),
        other => Err(format!("beklenmeyen köprü yanıtı: {other}")),
    }
}

/// QR akışını kapatır (diyalog kapanınca ya da başka adıma geçilince).
#[tauri::command]
fn account_qr_stop(app: AppHandle) -> Result<(), String> {
    preview::request_qr_stop(&app)
}

/// openani.me oturumunu kapatır.
///
/// İki adım: sunucudaki `refreshToken`'ı iptal etmek (`POST /user/logout`) ve
/// önizlemenin çerezlerini silmek. İkincisi belirleyici olan — uygulamanın
/// "giriş yapılmış mı" sorusuna verdiği yanıt `token` çerezinin varlığına
/// bakıyor (bkz. `preview_login_state`).
///
/// Sunucu isteği EN İYİ ÇABA: sayfa onu beklemiyor, geçit bayatsa atlanıyor.
/// Sitenin kendi `logout()`'u da `fetch`'i `await` etmeden çerezleri siliyor.
/// Tersi olsaydı, geçit toparlanmadığı sürece kullanıcı oturumunu hiç
/// kapatamazdı — çıkışın başarısız olabilmesi kabul edilebilir bir sonuç değil.
#[tauri::command]
async fn account_logout(app: AppHandle) -> Result<(), String> {
    let reply = bridge_await(&app, 30, preview::request_logout).await?;

    match reply.stage.as_str() {
        "done" if reply.ok == Some(true) => Ok(()),
        "done" => Err(reply
            .message
            .unwrap_or_else(|| "oturum kapatılamadı".into())),
        "error" => Err(format!(
            "oturum kapatılamadı: {}",
            reply.message.unwrap_or_else(|| "bilinmeyen hata".into())
        )),
        other => Err(format!("beklenmeyen köprü yanıtı: {other}")),
    }
}

/// Giriş yapmış kullanıcının kendi hesap nesnesi (`GET /user`).
#[tauri::command]
async fn fetch_account_info(app: AppHandle) -> Result<serde_json::Value, String> {
    bridge_get(&app, "/user").await
}

/// Takipçi / takip edilen listesini çeker.
///
/// Uç noktalar ve yanıt anahtarları sitenin kendi profil diyaloglarından
/// alındı: `GET /user/<id>/followers` -> `{ followers: [...] }`,
/// `GET /user/<id>/following` -> `{ following: [...] }`. Liste öğelerinde
/// arayüzün kullandığı alanlar `id`, `username`, `avatar`.
///
/// `user_id` ve `kind` burada DOĞRULANIYOR. Köprü aldığı yolu olduğu gibi
/// `API_BASE`'in sonuna ekliyor; doğrulama olmasa frontend'den gelen bir
/// dize onu istediği uç noktaya yönlendirebilirdi. Kimlikler openani.me'de
/// yalnızca rakamlardan oluşan dizeler (JSON'da string; sayı olsalardı
/// 2^53'ü aştıkları için tarayıcı tarafında hassasiyet kaybederlerdi).
#[tauri::command]
async fn fetch_account_follows(
    app: AppHandle,
    user_id: String,
    kind: String,
) -> Result<serde_json::Value, String> {
    if kind != "followers" && kind != "following" {
        return Err(format!("bilinmeyen liste türü: {kind}"));
    }
    if user_id.is_empty() || user_id.len() > 32 || !user_id.bytes().all(|b| b.is_ascii_digit()) {
        return Err("geçersiz kullanıcı kimliği".into());
    }

    let mut body = bridge_get(&app, &format!("/user/{user_id}/{kind}")).await?;

    // Yanıt sarmalı: {"followers": [...]} / {"following": [...]}. Sarmalın
    // içindekini döndürüyoruz ki arayüz iki listeyi aynı şekilde işlesin.
    match body.get_mut(&kind) {
        Some(list) => Ok(list.take()),
        None => Err(format!("yanıtta '{kind}' alanı yok")),
    }
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

/// Kurulan güncellemeyi devreye almak için uygulamayı yeniden başlatır.
///
/// `tauri::AppHandle::restart` çekirdek Tauri'nin bir parçası (bir eklenti
/// değil) ama frontend'e IPC ile açık değil — bu yüzden tek satırlık bu
/// komut var. Ayrı bir `tauri-plugin-process` bağımlılığı eklemeye değmezdi:
/// tek ihtiyacımız bu.
#[tauri::command]
fn restart_app(app: AppHandle) {
    app.restart();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(ThemeState::default())
        .manage(discord::DiscordState::default())
        .manage(updater::UpdaterState::default())
        .manage(easter_egg::Lock::default())
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
            account_login,
            account_logout,
            account_qr_next,
            account_qr_stop,
            fetch_account_info,
            fetch_account_follows,
            import_css_text,
            restart_app,
            projects::projects_dir_path,
            projects::list_projects,
            projects::load_project,
            projects::save_project,
            projects::delete_project,
            projects::rename_project,
            discord::discord_update,
            discord::discord_set_enabled,
            updater::updater_check,
            updater::updater_download,
            easter_egg::easter_egg_open,
            easter_egg::easter_egg_close
        ])
        .on_window_event(|window, event| {
            // Pencere kapanırken Discord aktivitesini temizle.
            //
            // Süreç öldüğünde Discord presence'ı kendi zaman aşımına kadar
            // (yaklaşık bir dakika) göstermeye devam ediyor; kullanıcı
            // uygulamayı kapattıktan sonra hâlâ "tema düzenliyor" görünmesi
            // yanlış olurdu. `CloseRequested` seçildi çünkü `Destroyed`
            // tetiklendiğinde thread'in temizliği bitirecek zamanı kalmıyor.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.state::<easter_egg::Lock>().get() {
                    api.prevent_close();
                    return;
                }
                window.state::<discord::DiscordState>().shutdown();
            }
        })
        .setup(|app| {
            // Pencere `tauri.conf.json`'da sabit 1400x900 açılıyor — bu, o
            // boyuttan küçük ekranlarda (ör. 1366x768 dizüstü) pencerenin
            // taşmasına ya da görev çubuğunun altına inmesine yol açıyordu.
            //
            // `preview::create`'DEN ÖNCE çalışmalı: o fonksiyon önizleme
            // webview'inin ilk boyutunu ana pencerenin O ANKİ `inner_size`'ına
            // göre hesaplıyor (bkz. preview.rs -> FALLBACK_PANEL_WIDTH). Sırayı
            // tersine çevirirsek önizleme bir kare eski (küçültülmeden önceki)
            // boyutla açılıp kendini frontend'in ResizeObserver'ı devreye
            // girene kadar yanlış yerde gösterirdi.
            if let Some(window) = app.get_webview_window("main") {
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let scale = monitor.scale_factor();
                    let work_area = monitor.work_area();
                    // `work_area` fiziksel piksel (görev çubuğu hariç); pencere
                    // boyutları mantıksal piksel — DPI ölçeğine bölerek eşitliyoruz.
                    let work_w = work_area.size.width as f64 / scale;
                    let work_h = work_area.size.height as f64 / scale;

                    const IDEAL_W: f64 = 1400.0;
                    const IDEAL_H: f64 = 900.0;
                    // Ekranı uç noktalarına kadar doldurmak yerine küçük bir
                    // pay bırakıyoruz — tam ekran boyutunda bir pencere
                    // "sığdırılmış" değil "taşmanın eşiğinde" hissettirir.
                    const MARGIN: f64 = 0.92;

                    let target_w = IDEAL_W.min(work_w * MARGIN);
                    let target_h = IDEAL_H.min(work_h * MARGIN);

                    // Alt sınır (900x600) elle uygulanmıyor: `tauri.conf.json`
                    // -> `minWidth`/`minHeight` zaten kalıcı bir kısıt olarak
                    // devrede, `set_size` onun altına inemez.
                    let _ = window.set_size(tauri::LogicalSize::new(target_w, target_h));
                }
                // Konfigürasyonda `"center": true` yok; her açılışta ortalıyoruz
                // ki küçültülmüş pencere de ekranın bir köşesinde kalmasın.
                let _ = window.center();
            }

            let doc = {
                let state: State<ThemeState> = app.state();
                let guard = state.0.lock().expect("tema kilidi");
                guard.clone()
            };
            preview::create(app.handle(), &doc)?;

            // Burada bir oturum tazeleme döngüsü YOK ve olmamalı.
            //
            // Önceden 58 saniyede bir `POST /user/refresh` çağıran bir kalp atışı
            // vardı; iki ayrı sebeple kaldırıldı. Birincisi: istek Rust'tan
            // atıldığı için `Gateway-Token` taşımıyordu, yani hiçbir zaman
            // başarılı olmuyordu (bkz. `fetch_account_info`). İkincisi — asıl
            // önemlisi — başarılı olsaydı zararlı olurdu: `/user/refresh` YENİ bir
            // `refreshToken` döndürüyor (rotasyon) ve sayfa bu çağrıyı
            // `navigator.locks.request("openanime_token_refresh_lock")` kilidi
            // ile serileştirip `oa_last_refresh_timestamp` üzerinden 5 dakikalık
            // bir pencereyle sınırlıyor. Paralel bir tazeleme, sayfanın elindeki
            // refresh token'ı geçersizleştirip oturumu düşürebilirdi.
            //
            // Token yönetimi tamamen sayfaya bırakıldı: sitenin kendi auth mutex'i
            // 8 dakikada bir ve sekme görünür olunca tazeliyor.

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
