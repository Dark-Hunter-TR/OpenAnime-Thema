//! Discord Rich Presence.
//!
//! Kullanıcı Discord'da "OpenAnime Theme oynuyor" şeklinde görünür; alt
//! satırlarda hangi bölümde olduğu ve düzenlediği temanın adı yazar.
//!
//! ## Neden ayrı bir thread
//!
//! `discord-rich-presence` tamamen SENKRON: `connect()` ve `set_activity()`
//! adlandırılmış boruya (Windows'ta `\\.\pipe\discord-ipc-0`) bloke edici
//! yazar. Discord açık değilken `connect()` hemen hata döndürür ama Discord
//! açılırken bu çağrı yüz milisaniyeler sürebiliyor. Tauri komutundan
//! doğrudan çağrılsaydı, arayüzdeki her sekme değişimi o süre boyunca IPC
//! kuyruğunu bekletirdi. Bu yüzden komutlar yalnızca paylaşılan duruma yazıp
//! thread'i dürtüyor; ağır iş orada oluyor.
//!
//! ## Neden anket (polling) döngüsü
//!
//! Discord'un RPC uç noktası aktivite güncellemelerini kısıtlıyor (20 saniyede
//! ~5 istek). Arayüz ise tek bir işlemde birkaç durum değişikliği üretebiliyor
//! (ör. proje açılırken `view`, `themeName` ve `editMode` arka arkaya
//! değişiyor). Her değişimi anında göndermek kısıtlamayı tetiklerdi. Döngü
//! saniyede bir uyanıp SON durumu okuyor ve yalnızca gerçekten değiştiyse,
//! üstelik `MIN_SEND_INTERVAL`'dan sık olmamak kaydıyla gönderiyor. Ara
//! durumlar böyle kendiliğinden yutuluyor.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::{Deserialize, Serialize};

/// Discord Developer Portal'daki uygulamanın **Application ID**'si.
///
/// DİKKAT: Rich Presence'ın en üst satırında görünen ad bu ID'ye bağlı
/// application'ın Portal'daki adıdır ve buradaki koddan DEĞİŞTİRİLEMEZ —
/// `activity::Activity`'nin öyle bir alanı yok. Yani "OpenAnime Theme"
/// yazması için Portal'daki application'ın adının birebir "OpenAnime Theme"
/// olması gerekiyor. Başka bir projenin (ör. OpenAnime bot'unun) ID'si
/// kullanılırsa orada ne yazıyorsa o görünür.
///
/// Bot, OAuth ya da doğrulama GEREKMEZ; boş bir application yeterli.
const CLIENT_ID: &str = "";

/// Portal → Rich Presence → Art Assets'e yüklenen görselin adı.
///
/// Asset yüklenmemişse Discord büyük görseli hiç çizmez; presence yine
/// çalışır, sadece görselsiz görünür. Buraya asset adı yerine tam bir
/// `https://` adresi de yazılabilir (Discord onu kendi CDN'ine kopyalar),
/// ama asset yolu daha hızlı ve bağlantı çürümesine bağışık.
const LARGE_IMAGE: &str = "logo";

/// Presence'ın altındaki tıklanabilir buton.
const REPO_URL: &str = "https://github.com/Dark-Hunter-TR/OpenAnime-Thema";
const REPO_LABEL: &str = "GitHub'da Görüntüle";

/// İki `set_activity` arasındaki en kısa süre.
///
/// Discord'un kendi sınırı 20 saniyede 5 istek. 4 saniye o sınırın rahatça
/// altında kalırken sekme değişiminin gözle fark edilir bir gecikme
/// yaratmayacağı en büyük değer.
const MIN_SEND_INTERVAL: Duration = Duration::from_secs(4);

/// Başarısız bağlantı denemeleri arasındaki bekleme.
///
/// Discord kapalıyken `connect()` her seferinde hata döndürür. Saniyede bir
/// denemek boru yolunu boşuna dövmek olurdu; kullanıcı Discord'u açtığında
/// en fazla bu kadar sonra bağlanılır.
const RECONNECT_INTERVAL: Duration = Duration::from_secs(15);

/// Arayüzün üst düzey görünümü. `src/lib/nav.ts` içindeki `NavId` ile eş.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum View {
    Home,
    Editor,
    Settings,
    About,
}

/// Editördeki düzenleme kipi. `AppSettings["defaultEditMode"]` ile eş.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase")]
pub enum EditMode {
    Visual,
    Code,
}

/// Discord'a yansıtılacak durumun tamamı.
///
/// `PartialEq` kasıtlı: döngü "değişti mi" kontrolünü alan alan yapmak yerine
/// tek bir karşılaştırmayla yapıyor, böylece ileride alan eklendiğinde
/// karşılaştırmayı güncellemeyi unutmak mümkün olmuyor.
#[derive(Clone, PartialEq, Debug)]
struct Presence {
    view: View,
    theme_name: String,
    edit_mode: EditMode,
}

impl Default for Presence {
    fn default() -> Self {
        Self {
            view: View::Home,
            theme_name: String::new(),
            edit_mode: EditMode::Visual,
        }
    }
}

/// Komutlarla thread arasında paylaşılan durum.
struct Shared {
    presence: Presence,
    enabled: bool,
}

enum Signal {
    Wake,
    Shutdown,
}

/// Tauri'nin `manage` ettiği tutamak.
pub struct DiscordState {
    shared: Arc<Mutex<Shared>>,
    tx: Sender<Signal>,
}

impl DiscordState {
    pub fn new() -> Self {
        let shared = Arc::new(Mutex::new(Shared {
            presence: Presence::default(),
            // Açılışta kapalı: frontend ayarları `localStorage`'dan okuyup
            // `discord_set_enabled` ile durumu bildirene kadar hiçbir şey
            // gönderilmemeli. Varsayılan açık olsaydı, RPC'yi kapatmış bir
            // kullanıcı her açılışta bir anlığına Discord'da görünürdü.
            enabled: false,
        }));

        let (tx, rx) = mpsc::channel();
        let worker_shared = shared.clone();

        thread::spawn(move || run_worker(worker_shared, rx));

        Self { shared, tx }
    }

    /// Görünen durumu günceller. Frontend her ilgili değişimde çağırır.
    pub fn update(&self, view: View, theme_name: String, edit_mode: EditMode) {
        if let Ok(mut s) = self.shared.lock() {
            s.presence = Presence {
                view,
                theme_name,
                edit_mode,
            };
        }
        let _ = self.tx.send(Signal::Wake);
    }

    /// Özelliği açar/kapatır. Kapatıldığında mevcut aktivite temizlenir.
    pub fn set_enabled(&self, enabled: bool) {
        if let Ok(mut s) = self.shared.lock() {
            s.enabled = enabled;
        }
        let _ = self.tx.send(Signal::Wake);
    }

    /// Uygulama kapanırken aktiviteyi temizlemesi için thread'e haber verir.
    ///
    /// Bu olmadan Discord, süreç öldükten sonra da presence'ı bir süre (kendi
    /// zaman aşımına kadar) göstermeye devam ediyor.
    pub fn shutdown(&self) {
        let _ = self.tx.send(Signal::Shutdown);
    }
}

impl Default for DiscordState {
    fn default() -> Self {
        Self::new()
    }
}

/// Arka plan döngüsü.
fn run_worker(shared: Arc<Mutex<Shared>>, rx: mpsc::Receiver<Signal>) {
    // CLIENT_ID doldurulmadıysa hiç uğraşma. Boş ID ile `connect()` Discord
    // tarafından reddedilir ve döngü sonsuza kadar boşuna dener.
    if CLIENT_ID.is_empty() {
        return;
    }

    let mut client: Option<DiscordIpcClient> = None;
    let mut last_connect_attempt: Option<Instant> = None;
    let mut last_send: Option<Instant> = None;

    // Discord'a EN SON gönderilmiş durum. `None` = aktivite temiz.
    //
    // Paylaşılan durumdan ayrı tutuluyor: "kullanıcının şu anki durumu" ile
    // "Discord'un bildiği durum" farklı şeyler ve gereksiz gönderimi
    // engelleyen tam olarak bu fark.
    let mut sent: Option<Presence> = None;

    loop {
        // Anket aralığı ile sinyal beklemeyi tek çağrıda birleştiriyoruz:
        // değişiklik olduğunda hemen uyanıyoruz, olmadığında da saniyede bir
        // uyanıp bağlantıyı yeniden kurmayı deneyebiliyoruz.
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Signal::Shutdown) | Err(RecvTimeoutError::Disconnected) => {
                if let Some(mut c) = client.take() {
                    let _ = c.clear_activity();
                    let _ = c.close();
                }
                return;
            }
            Ok(Signal::Wake) | Err(RecvTimeoutError::Timeout) => {}
        }

        let (presence, enabled) = match shared.lock() {
            Ok(s) => (s.presence.clone(), s.enabled),
            // Kilit zehirlendiyse (başka bir thread panikledi) presence'ı
            // sessizce bırak. Burada paniklemek uygulamayı öldürürdü ve
            // Rich Presence bunu hak edecek kadar önemli değil.
            Err(_) => return,
        };

        if !enabled {
            if sent.is_some() {
                if let Some(c) = &mut client {
                    if c.clear_activity().is_err() {
                        let _ = c.close();
                        client = None;
                    }
                }
                sent = None;
            }
            continue;
        }

        if sent.as_ref() == Some(&presence) {
            continue;
        }

        let now = Instant::now();
        if let Some(last) = last_send {
            if now.duration_since(last) < MIN_SEND_INTERVAL {
                // Henüz erken. Değişiklik `shared` içinde duruyor, bir sonraki
                // turda tekrar bakılacak — kaybolmuyor.
                continue;
            }
        }

        if client.is_none() {
            let due = last_connect_attempt
                .map(|last| now.duration_since(last) >= RECONNECT_INTERVAL)
                .unwrap_or(true);
            if !due {
                continue;
            }
            last_connect_attempt = Some(now);

            // Discord kurulu/açık değilse buraya normal işleyişte sürekli
            // düşülür; sessizce geçiyoruz.
            match DiscordIpcClient::new(CLIENT_ID) {
                Ok(mut c) => {
                    if c.connect().is_err() {
                        continue;
                    }
                    client = Some(c);
                }
                Err(_) => continue,
            }
        }

        let Some(c) = &mut client else { continue };

        let (details, state) = describe(&presence);

        let assets = activity::Assets::new()
            .large_image(LARGE_IMAGE)
            .large_text("OpenAnime Tema Editörü");

        let activity = activity::Activity::new()
            .details(&details)
            .state(&state)
            .assets(assets)
            .buttons(vec![activity::Button::new(REPO_LABEL, REPO_URL)]);

        if c.set_activity(activity).is_ok() {
            sent = Some(presence);
            last_send = Some(now);
        } else {
            // Boru koptu (Discord kapandı ya da yeniden başladı). İstemciyi
            // düşür ki bir sonraki turda yeniden bağlanma yoluna girsin.
            let _ = c.close();
            client = None;
            // `sent`'i de sıfırlıyoruz: yeni bağlantıda Discord hiçbir şey
            // bilmiyor olacak, aynı durumu tekrar göndermek gerekecek.
            sent = None;
        }
    }
}

/// Duruma karşılık gelen iki satırı üretir.
///
/// Dönüş `String`, `&'static str` değil: `Activity` metinleri ödünç aldığı
/// için tema adının çağıran kapsamda yaşaması gerekiyor ve iki satırın
/// ömrünü aynı yerde tutmak karışıklığı azaltıyor.
fn describe(p: &Presence) -> (String, String) {
    match p.view {
        View::Home => (
            "Ana ekranda".to_string(),
            "Tema seçiyor".to_string(),
        ),
        View::Editor => {
            let mode = match p.edit_mode {
                EditMode::Visual => "Görsel düzenleyici",
                EditMode::Code => "Kod düzenleyici",
            };
            let name = p.theme_name.trim();
            // Discord `state` için 2-128 karakter istiyor ve sınır dışındaki
            // değerde aktivitenin TAMAMINI reddediyor. Proje adı uzunluğu
            // uygulamada kısıtlı olmadığı için iki ucu da burada kapatıyoruz.
            let name = if name.chars().count() < 2 {
                "İsimsiz tema".to_string()
            } else {
                truncate(name, 128)
            };
            (format!("{mode} · tema düzenliyor"), name)
        }
        View::Settings => (
            "Ayarlarda".to_string(),
            "Editörü yapılandırıyor".to_string(),
        ),
        View::About => (
            "Hakkında ekranında".to_string(),
            "OpenAnime Tema Editörü".to_string(),
        ),
    }
}

/// Metni en fazla `max` KARAKTERE kısaltır.
///
/// `chars()` üzerinden gidiyor, bayt dilimlemiyor: tema adları Türkçe karakter
/// içerebiliyor ve bayt sınırından kesmek UTF-8 dizisini ortadan bölerek
/// panikletirdi.
fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    // Son karakteri üç noktaya bırakıyoruz ki kırpıldığı belli olsun.
    s.chars().take(max - 1).chain(std::iter::once('…')).collect()
}

#[tauri::command]
pub fn discord_update(
    state: tauri::State<'_, DiscordState>,
    view: View,
    theme_name: String,
    edit_mode: EditMode,
) {
    state.update(view, theme_name, edit_mode);
}

#[tauri::command]
pub fn discord_set_enabled(state: tauri::State<'_, DiscordState>, enabled: bool) {
    state.set_enabled(enabled);
}
