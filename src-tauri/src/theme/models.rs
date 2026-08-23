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

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::System
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
    /// Yalnızca KONTROLLERİ BESLEMEK için okunan token'lar — üretilen CSS'e
    /// hiç yazılmazlar.
    ///
    /// Harici temaların çoğu renklerini `.fds-theme-dark` gibi kipe bağlı bir
    /// blokta tanımlıyor. Bu değerleri `token_overrides`'a koymak yanlış
    /// olurdu: oradan `:root, .fds-theme-light, .fds-theme-dark` bloğuna
    /// yazılırlar ve koyu kip için yazılmış renkler açık kipe de sızardı.
    /// Ama görmezden gelmek de yanlış: o zaman kullanıcı temayı açtığında
    /// görsel kontroller boş görünür ("hiçbir şey uygulanmamış" hissi).
    ///
    /// Ayrım bu alanla yapılıyor: değerler kapsamlarında (ham CSS'te) duruyor,
    /// kontroller ise onları buradan okuyup gösteriyor.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub seed_tokens: BTreeMap<String, String>,
    /// İçe aktarılan temanın KURALLARI — çözümlenmiş, sıralı ve tek tek
    /// düzenlenebilir hâlde.
    ///
    /// Burası eskiden `imported_css` adında tek parça bir metindi: dosyadan
    /// gelen ne varsa olduğu gibi saklanıp önizlemeye olduğu gibi basılıyordu.
    /// Ölçülen sonucu şuydu — 103 seçicilik gerçek bir temanın 101 seçicisi
    /// modele hiç girmiyor, 101 KB'lık yabancı bir stil sayfası olarak
    /// sayfaya ekleniyordu. İki somut arıza doğuruyordu:
    ///
    ///   1. O metin bizim değil, sayfanın bir parçasıydı. İçindeki 255 adet
    ///      `!important` bildirimini kontroller ezemiyordu — kullanıcı bir
    ///      kaydırıcıyı oynatıyor, ekranda hiçbir şey değişmiyordu.
    ///   2. Tek parça olduğu için kural bazında ele alınamıyordu: ne
    ///      listelenebiliyor, ne kapatılabiliyor, ne de bir bildirimi
    ///      değiştirilebiliyordu.
    ///
    /// Şimdi dosyadaki her kural ayrı bir kayıt. Önizlemeye giden CSS'in
    /// tamamı bu modelden üretiliyor; sayfaya dışarıdan hiçbir şey
    /// eklenmiyor. Bir kontrol bir bildirimin sahibi olduğunda `emit_css` o
    /// bildirimi ithal kuraldan ÇIKARIYOR — yani kontrol, sıraya ya da
    /// özgüllüğe güvenmek yerine çakışan bildirimi ortadan kaldırarak
    /// kazanıyor (`!important` da dahil).
    ///
    /// Sıra korunuyor: CSS'te eşit özgüllükte sonra gelen kazanır, dolayısıyla
    /// belge sırası anlamın bir parçası.
    ///
    ///   imported_rules → yönetilen blok → raw_css
    ///
    /// İthal tema TABAN; `raw_css` ise kullanıcının kendi yazdığı kaçış kapısı
    /// ve en sonda kalıyor — oraya yazılan bir kural kontrolleri ezebilmeli.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub imported_rules: Vec<ImportedRule>,
    /// İçe aktarma ANINDAKİ özel değişken değerleri.
    ///
    /// Sahiplik kararı bunun üzerinden veriliyor: `token_overrides`'taki bir
    /// değer buradakiyle AYNIYSA kullanıcı o token'a hiç dokunmamıştır ve
    /// temanın kendi (kipe ya da ekran boyutuna bağlı) tanımları olduğu gibi
    /// korunur. Farklıysa kullanıcı kontrolden değiştirmiştir; o bildirim
    /// artık bizimdir ve ithal kurallardan çıkarılır.
    ///
    /// Bu ayrım olmadan iki uçtan birine düşülüyordu: ya ithal tanımlar hep
    /// kalıyor (kontrol hiç çalışmıyor) ya da hep siliniyordu (`.fds-theme-dark`
    /// için yazılmış koyu renkler daha kullanıcı hiçbir şeye dokunmadan
    /// kayboluyordu).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub imported_tokens: BTreeMap<String, String>,
    /// İçe aktarma ANINDAKİ vurgu rengi.
    ///
    /// `imported_tokens` ile aynı işi vurgu ekseninde yapıyor: kullanıcı
    /// kaydırıcıya gerçekten dokundu mu, buna bakarak anlaşılıyor.
    ///
    /// Somut hata: `emit_css`, vurgu KÜTÜPHANE VARSAYILANINDAN farklıysa
    /// seçili kenar çubuğu öğesini boyayan bir kural yazıyor
    /// (`.list-item.selected * { color: var(--fds-accent-default) }`). İçe
    /// aktarmada `accent` temanın kendi vurgusu oluyor, yani kullanıcı hiçbir
    /// şeye dokunmasa bile o kural yazılıyor ve temanın kendi ikon rengiyle
    /// çakışıyordu. Koyu kipte `--fds-accent-default` rampanın `light-2`
    /// basamağından türüyor; o basamak beyaza kırpıldığında seçili menü ikonu
    /// düpedüz beyaz çıkıyordu.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported_accent: Option<Hsl>,
    /// ESKİ projelerden gelen tek parça ithal gövde.
    ///
    /// Yalnızca geriye dönük uyumluluk için duruyor; yeni bir ayrıştırma bunu
    /// asla doldurmuyor. Dolu bulunduğunda `migrate_imported` onu
    /// `imported_rules`'a çeviriyor — yani eski bir proje açıldığında da tema
    /// modele giriyor, sayfaya ham blok olarak basılmıyor.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub imported_css: String,
    /// İçe aktarılan temanın KENDİ vurgu değişkenleri.
    ///
    /// Gelişmiş temalar `--fds-accent-*`'ı kullanmıyor; kendi adlarını
    /// tanımlayıp (`--accent-primary`, `--accent-secondary` …) her yerde
    /// `var(--accent-primary) !important` ile boyuyorlar. Vurgu kaydırıcısı
    /// yalnızca `--fds-accent-*`'ı yazdığı için tema o değişkenlere hiç
    /// dokunulmuyor ve renk değişikliği "yarım" kalıyordu.
    ///
    /// Burada her değişkenin, içe aktarma anındaki vurgudan SAPMASI tutuluyor.
    /// Böylece kaydırıcı oynadığında aile ilişkisi korunuyor: açık ton açık,
    /// koyu ton koyu kalıyor — mutlak renkler saklansaydı hepsi tek renge
    /// çökerdi.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accent_aliases: Vec<AccentAlias>,
    /// İçe aktarılan temanın KENDİ yuvarlaklık değişkenleri.
    ///
    /// `accent_aliases` ile aynı gerekçe, yuvarlaklık ekseninde
    /// (bkz. `RadiusAlias`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub radius_aliases: Vec<RadiusAlias>,
    /// İçe aktarılan temanın, site token'larını gölgeleyen kendi değişkenleri.
    ///
    /// `accent_aliases` ve `radius_aliases` iki ekseni çözüyordu; bu, geri
    /// kalan her kontrolü çözüyor (gerekçe: `TokenAlias`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub token_aliases: Vec<TokenAlias>,
    pub raw_css: String,
}

/// Temanın kendi vurgu değişkenlerinden biri ve vurgudan sapması.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccentAlias {
    pub name: String,
    /// Ton farkı (derece), doygunluk ve ışıklılık farkı (yüzde puanı).
    pub delta: [f64; 3],
    /// Değişkenin KENDİ alfası (0.0–1.0).
    ///
    /// Sapmaya dahil değil, olduğu gibi korunuyor: kaydırıcı rengi değiştirir,
    /// saydamlığı değiştirmez. Taşınmadığında temanın `rgba(…, 0.04)` gibi
    /// ince tint'leri opak vurgu bloklarına dönüşüyordu.
    ///
    /// Eski projelerde alan yok; varsayılanı 1.0 (opak).
    #[serde(default = "one")]
    pub alpha: f64,
}

fn one() -> f64 {
    1.0
}

/// Temanın KENDİ değişkeni ve gölgelediği site token'ı.
///
/// Genel hâli `AccentAlias`/`RadiusAlias`ın: onlar renk ve yuvarlaklık
/// eksenlerini çözüyordu, bu ise GERİ KALAN HER ŞEYİ. Sorun her eksende aynı
/// biçimde tekrarlıyordu — kontrol site token'ını yazıyor, temanın kuralları
/// kendi adını okuyor, ekranda hiçbir şey değişmiyor:
///
/// ```text
/// :root { --bg-page: #111118; --fds-solid-background-base: #111118; }
/// html  { background-color: var(--bg-page); }
/// ```
///
/// "Sayfa zemini" kontrolü `--fds-solid-background-base`'i değiştiriyor, ama
/// sayfa zeminini `--bg-page` boyuyor. Aynısı metin renginde, yazı tipinde,
/// gölgede, geçiş süresinde…
///
/// ## Eşleştirme neden DEĞERE bakıyor
///
/// Ada bakmak işe yaramaz: adlar temaya ve dile göre değişiyor (`--bg-page`,
/// `--mn-bg1`, `--zemin`). Site token'ının içe aktarma anındaki değeriyle
/// BİREBİR aynı değeri taşıyan bir tema değişkeni, pratikte aynı düğmedir —
/// temayı yazan kişi ikisini elle eşitlemiştir. Ek güvence olarak yalnızca
/// temanın gerçekten `var()` ile KULLANDIĞI adlar alınıyor; kullanılmayan bir
/// değişkeni yeniden yazmanın hiçbir etkisi olmazdı zaten.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TokenAlias {
    /// Temanın kendi değişkeni (`--bg-page`).
    pub name: String,
    /// Gölgelediği site token'ı (`--fds-solid-background-base`).
    pub source: String,
}

/// Temanın KENDİ yuvarlaklık değişkenlerinden biri ve kaydırıcıdan sapması.
///
/// `AccentAlias` ile birebir aynı gerekçe, farklı eksen. Yuvarlaklık kaydırıcısı
/// yalnızca `--fds-control-corner-radius` / `--fds-overlay-corner-radius`
/// yazıyor; gelişmiş temalar ise kendi adlarını tanımlayıp her yerde
/// `border-radius: var(--ayar-kose-yuvarlakligi-genel) !important` ile
/// kullanıyor. Temanın hiçbir kuralı bizim yazdığımız token'ı OKUMADIĞI için
/// kaydırıcı hiçbir şeye dokunmuyordu — kullanıcının "köşe yumuşatma asla
/// çalışmıyor" dediği durum tam olarak buydu.
///
/// Sapma (px) saklanıyor, mutlak değer değil: tema kartlara 12px, banner'a 4px
/// veriyorsa aradaki fark korunuyor. Mutlak değerler yazılsaydı kaydırıcı
/// oynatıldığında hepsi tek bir yuvarlaklığa çökerdi.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RadiusAlias {
    pub name: String,
    /// İçe aktarma anındaki kontrol yuvarlaklığından fark (px).
    pub delta: f64,
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
            seed_tokens: BTreeMap::new(),
            imported_rules: Vec::new(),
            imported_tokens: BTreeMap::new(),
            imported_accent: None,
            imported_css: String::new(),
            accent_aliases: Vec::new(),
            radius_aliases: Vec::new(),
            token_aliases: Vec::new(),
            raw_css: String::new(),
        }
    }
}

/// İçe aktarılan bir temanın tek bir kuralı.
///
/// `at`, kuralı saran koşul at-kurallarının zinciri — en dıştan içe
/// (`["@supports (x)", "@media (max-width: 768px)"]`). Sarmalayıcılar
/// ATILMIYOR: atılsaydı yalnızca dar ekranda geçerli bir kural her ekranda
/// geçerli olurdu ve gerçek temalar mobil kırılımlarını tam olarak böyle
/// yazıyor. Zincir olarak tutulmasının sebebi iç içe geçebilmeleri.
///
/// `selector` bir seçici (`.anime-card`) ya da kendi başına bir at-kuralı
/// (`@keyframes parlama`, `@font-face`) olabilir. İkinci durumda `body`
/// bildirim değil iç içe kural taşır ve olduğu gibi yazılır.
///
/// `note`, kuralın hemen üstündeki açıklama yorumu. Saklanmasının sebebi
/// pratik: elle yazılmış temalarda her bölümün ne yaptığı bu yorumlarda
/// anlatılıyor (örnek temada 100'den fazla var). Yorumlar modele girmeseydi
/// kullanıcı temasını `.css` olarak geri kaydettiğinde dosyası sessizce
/// açıklamalarından arınmış olurdu.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportedRule {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub at: Vec<String>,
    pub selector: String,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl ThemeDoc {
    pub fn emit_css(&self) -> String {
        crate::theme::emit::emit_css(self)
    }

    /// Eski projelerin tek parça `imported_css`ini kurallara çevirir.
    ///
    /// Çağrılma yeri `apply_theme`: ön yüzden gelen HER belge buradan geçiyor,
    /// dolayısıyla proje açılışı da dahil tek bir kapı yetiyor. Yeni
    /// ayrıştırma zaten `imported_rules` üretiyor; bu yüzden koşul
    /// "kurallar boş ama metin dolu" ile sınırlı ve normal akışta hiç
    /// çalışmıyor.
    pub fn migrate_imported(&mut self) {
        if !self.imported_rules.is_empty() || self.imported_css.trim().is_empty() {
            self.imported_css = String::new();
            return;
        }
        self.imported_rules = crate::theme::parse::decompose_imported(&self.imported_css);
        self.imported_css = String::new();
    }

    /// Modelde ithal bir tema var mı.
    pub fn has_imported(&self) -> bool {
        !self.imported_rules.is_empty() || !self.imported_css.trim().is_empty()
    }
}
