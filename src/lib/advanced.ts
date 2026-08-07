/**
 * Gelişmiş stil ve düzen incelemelerinden elde edilen özelleştirme noktaları.
 * Sitenin canlı CSS yapısı analiz edilerek çıkarılmıştır.
 *
 * Buradaki her selector openani.me'nin CANLI CSS'inde aranıp doğrulandı.
 * Sitede karşılığı bulunmayan selector'lar dışarıda bırakılmıştır.
 */

import type { ColorTokenSpec } from "$lib/customization";

// --- Metin ve odak (3 temada --fds-text-primary / -secondary eziliyor) ------

export const TEXT_TOKENS: ColorTokenSpec[] = [
	{
		token: "--fds-text-primary",
		label: "Ana metin",
		hint: "Sitede varsayılan metin rengi",
		alpha: true,
		defaultAlpha: 100
	},
	{
		token: "--fds-text-secondary",
		label: "İkincil metin",
		hint: "Açıklama, alt başlık",
		alpha: true,
		defaultAlpha: 79
	},
	{
		token: "--fds-focus-stroke-outer",
		label: "Odak halkası (dış)",
		hint: "Klavyeyle gezinirken görünen odak çerçevesi",
		alpha: true,
		defaultAlpha: 100
	},
	{
		token: "--fds-focus-stroke-inner",
		label: "Odak halkası (iç)",
		hint: "Dış halkanın içindeki kontrast çizgisi",
		alpha: true,
		defaultAlpha: 100
	}
];

// --- Kartlar (3 temada --fds-card-background-default eziliyor) --------------

export const CARD_TOKENS: ColorTokenSpec[] = [
	{
		token: "--fds-card-background-default",
		label: "Kart arkaplanı",
		hint: "Anime kartları ve listelenen kartların zemin rengi",
		alpha: true,
		defaultAlpha: 70
	},
	{
		token: "--fds-card-background-secondary",
		label: "Kart arkaplanı (ikincil)",
		hint: "İç içe kart yüzeyleri",
		alpha: true,
		defaultAlpha: 50
	},
	{
		token: "--fds-card-stroke-default",
		label: "Kart kenarlığı",
		hint: "Kart çerçevesi",
		alpha: true,
		defaultAlpha: 10
	},
	{
		token: "--fds-layer-background-default",
		label: "Katman arkaplanı",
		hint: "Kartların altındaki katman yüzeyi",
		alpha: true,
		defaultAlpha: 30
	}
];

/**
 * Kart gövdesi. `.anime-card`, `.slider-card` ve `.grid-view-item` kartlarını hedefler.
 */
export const CARD_SELECTOR = ".anime-card, .slider-card, .grid-view-item";
export const CARD_HOVER_SELECTOR = ".anime-card:hover, .slider-card:hover, .grid-view-item:hover";

/** Geriye dönük ad — mevcut çağrı yerleri bozulmasın diye. */
export const CARD_RADIUS_SELECTOR = CARD_SELECTOR;

/** Kart parıltısı. */
export const CARD_GLOW_SELECTOR = CARD_HOVER_SELECTOR;

/**
 * Kart görselinin alt kenarını silikleştiren maske.
 * Site `.anime-card #main` ve `.grid-view-item img` üzerinde görseli basıyor.
 */
export const CARD_IMAGE_SELECTOR = ".anime-card #main, .grid-view-item img, .slider-card img";

// --- Kenar çubuğu ve menü ---------------------------------------------------

/**
 * Sol kenar çubuğu. Sitenin kendi ölçüsü `min-width: 4.5rem; max-width: 4.5rem`.
 * Genişlik ve menü öğeleri için gelişmiş düzenleme.
 */
export const SIDEBAR_SELECTOR = ".sidebar";

/**
 * Seçili menü öğesinin sol kenarındaki vurgu çubuğu.
 * Sitenin kendi kuralı:
 *   .list-item::before { inline-size: 3px; block-size: 16px;
 *                        border-radius: 3px; background: var(--fds-accent-default) }
 */
export const SIDEBAR_INDICATOR_SELECTOR = ".list-item::before";

/** Menü öğesi metni. `#label` sitede doğrulandı (`.sidebar a #label`). */
export const SIDEBAR_LABEL_SELECTOR = ".list-item [id='label']";

// --- Kaydırma çubuğu --------------------------------------------------------

/**
 * Site OverlayScrollbars kullanıyor ve `::-webkit-scrollbar`'ı açıkça
 * kapatıyor (`display: none !important`). Webkit sözde elemanları yerine
 * sitenin `--os-*` değişkenleri kullanılır.
 *
 * Doğru giriş noktası sitenin kendi beslediği `--os-*` değişkenleri:
 *   .os-scrollbar { --os-handle-bg: var(--fds-control-strong-fill-default);
 *                   --os-track-bg-hover: var(--fds-layer-background-default);
 *                   --os-track-border-radius: 50px; ... }
 *   .os-theme-dark/.os-theme-light { --os-size: 10px }
 */
export const SCROLLBAR_SELECTOR = ".os-scrollbar";

// --- Rozetler ---------------------------------------------------------------

/**
 * Üst çubuktaki "NEXT-GEN" rozeti.
 * Site: `#badge { background: linear-gradient(135deg,
 *        hsl(var(--fds-accent-light-1)) 0%, var(--fds-accent-default) 100%) }`
 */
export const BADGE_SELECTOR = "#badge";

/**
 * Takvim/kart üzerindeki "yayınlandı" rozeti. İki temada da yeniden boyanmış.
 * Site: `linear-gradient(to right, #6371da, var(--fds-accent-tertiary))`
 */
export const RELEASED_BADGE_SELECTOR = ".released-badge";

/**
 * Kart üzerindeki "geliştirilmiş" şeridi. İki temada da hedefleniyor.
 * Site: `linear-gradient(96.58deg, #66faff -100%, #196a91)`
 */
export const ENHANCED_SELECTOR = ".enhanced-highlight";

// --- Profil fotoğrafı -------------------------------------------------------

/**
 * `--fds-person-picture-size` resmi bir fds token'ı ve site onu her
 * PersonPicture örneğinde ayrı ayrı veriyor (üst çubukta 32px). Bu yüzden
 * yalnızca üst çubuktaki avatara uyguluyoruz; kart içindeki avatarları etkilemez.
 */
export const AVATAR_SELECTOR = ".topbar #account .person-picture-container";

// --- Banner / kayan kartlar -------------------------------------------------

/** Ana sayfadaki kayan kartın seçili hâli. Site: `outline-color: var(--fds-accent-default)`. */
export const BANNER_SELECTED_SELECTOR = ".slider-card.selected";

/**
 * Kayan kartların otomatik geçiş ilerleme çubuğu.
 * Site: `#progress { height: .3rem; background: #fff; border-radius: 50px }`
 */
export const BANNER_PROGRESS_SELECTOR = ".slider-card #progress";

// --- Yorumlar ---------------------------------------------------------------

/**
 * Yorum bölümü stil tanımları.
 *
 * Not: temalar Svelte hash'li hâllerini yazmış (`.comment.svelte-1snun1g`) ve
 * o hash artık geçersiz (site bugün `svelte-1xqk1kh` üretiyor) — yani o
 * kurallar şu an çalışmıyor. Biz hash'siz yazıp `!important` ile
 * özgüllüğü telafi ediyoruz, böylece deploy'lar arasında ayakta kalıyor.
 */
export const COMMENT_SELECTOR = ".comment";
export const COMMENT_INPUT_SELECTOR = ".input-wrapper";

// --- Oynatıcı ---------------------------------------------------------------

/** Alt kontrol çubuğu. Sitede 29 kural */
export const PLAYER_BAR_SELECTOR = ".bottom-controls";

/** Sahne kontrolleri (oynatıcı üstündeki butonlar). Sitede 5 kural. */
export const PLAYER_SCENE_BUTTON_SELECTOR = ".openanime-scene-controller button";

/**
 * İlerleme/ses çubuğu. fluent Slider'ın anatomisi (sitede doğrulandı):
 *   .slider.orientation-horizontal  -> dokunma alanı (32px)
 *     .slider-rail                  -> zemin (4px)
 *     .slider-track                 -> dolgu
 *     .slider-thumb                 -> topuz
 */
export const PLAYER_SLIDER_SELECTOR = ".slider.orientation-horizontal";
export const PLAYER_RAIL_SELECTOR = ".slider.orientation-horizontal .slider-rail";
export const PLAYER_TRACK_SELECTOR = ".slider.orientation-horizontal .slider-track";
export const PLAYER_THUMB_SELECTOR = ".slider.orientation-horizontal .slider-thumb";

/** Bölüm listesi. Sitede 11 kural; `.currentEpisode` 4 kural. */
export const PLAYER_EPISODE_SELECTOR = ".player-episode-list-item";
export const PLAYER_EPISODE_CURRENT_SELECTOR = ".player-episode-list-item.currentEpisode";

/**
 * Altyazı. Site `video::cue` kullanıyor (1 kural) AMA ASS/SSA altyazıları
 * `<canvas>`'a çiziyor — oraya CSS işlemez. Panelde bunu açıkça yazıyoruz.
 */
export const PLAYER_CUE_SELECTOR = "video::cue";

// --- Maskot (Setsuki) -------------------------------------------------------

/**
 * Sitede maskotun birden fazla örneği var; hepsi ayrı `<img>`.
 * `.setsuki #image` boyutu sitenin kendi `--s-width` / `--s-height`
 * değişkenlerinden geliyor — yani site bunu zaten expose ediyor.
 */
export interface MascotSlot {
	id: string;
	label: string;
	selector: string;
	hint: string;
	/**
	 * Sitede bu örneğin GERÇEK piksel boyutu. `null` ise boyut akışkandır
	 * (`height: 100%` / `width: auto`) ve sabit boyut vermek düzeni bozar —
	 * o slotlarda boyut kaydırıcısı gösterilmez.
	 */
	size: number | null;
}

export const MASCOT_SLOTS: MascotSlot[] = [
	{
		id: "generic",
		label: "Genel maskot",
		selector: ".setsuki #image",
		hint: "Boş durum ekranlarındaki Setsuki — boyutu akışkan",
		size: null
	},
	{
		id: "dialog",
		label: "Diyalog maskotu",
		selector: "#setsuki",
		hint: "Hakkında / merhaba diyaloğu — height: 100%, width: auto",
		size: null
	},
	{
		id: "notification",
		label: "Bildirim maskotu",
		selector: "#notification-setsuki",
		hint: "Bildirim paneli — sitede 170×170",
		size: 170
	},
	{
		id: "download",
		label: "İndirme maskotu",
		selector: "#download-setsuki",
		hint: "İndirme paneli — sitede 170×170",
		size: 170
	},
	{
		id: "mobileNotification",
		label: "Mobil bildirim maskotu",
		selector: "#mobile-notification-setsuki",
		hint: "Mobil bildirim — sitede 150×150",
		size: 150
	}
];

// --- Logo (3 temanın ortak kalıbı) -----------------------------------------

/**
 * Gizleme kuralı BİLEREK ikiye bölündü.
 *
 * Eskiden tek bir birleşik kural vardı ve "görsel VEYA metin" açıksa
 * hepsini gizliyordu. Sonuç: kullanıcı yalnızca site adını değiştirdiğinde
 * orijinal logo `<img>`'i de gizleniyor, ama yerine `::before` basılmadığı
 * için LOGO KAYBOLUYORDU. Artık her biri yalnızca kendi yerine bir şey
 * konduğunda gizleniyor.
 */
export const LOGO_IMAGE_HIDE_SELECTOR =
	".topbar a.logo img, .topbar a.logo svg, .topbar a.logo-button img, .topbar a.logo-button svg";

/**
 * Yalnızca site adı değişip görsel değişmediğinde devreye girer (bkz.
 * `advancedBuild.ts`). Orijinal ikon GİZLENMEZ, ama satırı flex yapan
 * `LOGO_ROW_SELECTOR` ve rozeti sabitleyen `LOGO_BADGE_SELECTOR` yüzünden
 * ikon satırdaki TEK esnek (shrink edilebilir) öğe kalır; yer daralınca 0
 * genişliğe küçülüp GÖRÜNMEZ olurdu. Bu seçici yalnızca flex-shrink'i
 * kapatıp konumunu sabitler — `LOGO_IMAGE_HIDE_SELECTOR`'dan BİLEREK farklı
 * bir seçici (doğrudan çocuk `>`): aynı string olsaydı kural haritasında
 * (selector -> body) aynı anahtara düşüp gizleme kuralıyla çakışırdı.
 */
export const LOGO_ICON_GUARD_SELECTOR =
	".topbar a.logo > img, .topbar a.logo > svg, .topbar a.logo-button > img, .topbar a.logo-button > svg";
/**
 * DOĞRUDAN çocuk (`>`) seçici BİLEREK kullanıldı: `#badge` (NEXT-GEN rozeti)
 * de kendi metnini aynı `.text-block` sınıfıyla basıyor, ama bir alt seviyede
 * (`a.logo-button #badge .text-block`). Descendant seçici kullanılırsa rozetin
 * metni de gizlenir ve rozet boş/görünmez kalır.
 */
export const LOGO_TEXT_HIDE_SELECTOR = ".topbar a.logo > .text-block, .topbar a.logo-button > .text-block";

/**
 * `#badge` (NEXT-GEN rozeti) BU LİSTEDE DEĞİL — bilerek.
 * Rozet sitenin kendi öğesi ve site adını değiştirmek onu yok etmemeli.
 * Uzun adların rozetin üstüne taşmaması `LOGO_ROW_SELECTOR` /
 * `LOGO_BADGE_SELECTOR` düzen kurallarıyla çözülüyor.
 */

/** Görselin basıldığı sözde-eleman. */
export const LOGO_IMAGE_SELECTOR = ".topbar a.logo::before, .topbar a.logo-button::before";
/** Metnin basıldığı sözde-eleman. */
export const LOGO_TEXT_SELECTOR = ".topbar a.logo::after, .topbar a.logo-button::after";

/**
 * Logo satırının düzeni.
 *
 * Sitenin kendi kuralı zaten `display: flex; align-items: center` — biz
 * yalnızca `gap` ve `min-width: 0` ekliyoruz. `min-width: 0` şart: flex
 * öğeleri varsayılan olarak `min-width: auto` ile içeriklerinden küçülemez,
 * bu yüzden `text-overflow: ellipsis` metin kutusunda ETKİSİZ kalır ve uzun
 * ad rozetin üstüne taşar.
 */
export const LOGO_ROW_SELECTOR = ".topbar a.logo, .topbar a.logo-button";

/**
 * Rozet asla ezilmesin/küçülmesin VE her zaman en sağda kalsın.
 *
 * `::after` (site adı) CSS gereği HER ZAMAN bir elemanın son kutusudur —
 * gerçek çocuklardan (rozet dâhil) sonra basılır. Yani "yalnızca `::after`
 * ekle" yaklaşımı, rozet gerçek bir çocuksa adı rozetin ARKASINA (sağına)
 * değil ÖNÜNE (soluna) atardı. `order` ile açıkça sıralanıyor: ikon (0),
 * ad (1), rozet (2) — bkz. `advancedBuild.ts`.
 */
export const LOGO_BADGE_SELECTOR = ".topbar a.logo #badge, .topbar a.logo-button #badge";

/**
 * Site DOM'unda `a.logo-button > a.logo` iç içe geçebiliyor. O durumda hem
 * dış hem iç elemana `::before`/`::after` basılır ve logo/ad ÇİFT görünür.
 * İç içe geçen bağlantılarda çift görünümü engeller.
 */
export const LOGO_NESTED_SELECTOR =
	".topbar a.logo-button a.logo::before, .topbar a.logo-button a.logo::after";

// --- Arkaplan ---------------------------------------------------------------

/**
 * Temalar arkaplan görselini `body`'ye basıp üstteki katmanları
 * şeffaflaştırıyor (aksi hâlde görsel görünmez).
 */
export const BG_BODY_SELECTOR = "body";
export const BG_TRANSPARENT_SELECTOR =
	"body, .page, .scene-inner-content, .openanime-scene-controller, .os-viewport";

// --- Tipografi --------------------------------------------------------------

export const FONT_TOKENS = [
	{ token: "--fds-font-family-text", label: "Gövde yazı tipi" },
	{ token: "--fds-font-family-display", label: "Başlık yazı tipi" },
	{ token: "--fds-font-family-small", label: "Küçük metin yazı tipi" }
];

// --- İçe aktarma: tanınan selector'lar --------------------------------------

/**
 * Kontrol paneline BAĞLI olan selector'ların tamamı.
 *
 * Dışarıdan bir tema içe aktarıldığında (GitHub, dosya, pano) Rust bu listeyi
 * kullanarak hangi kuralların kontrollere eşlenebileceğine karar veriyor
 * (`import_css_text` -> `parse_foreign_css`). Listede olmayan her kural ham
 * CSS'te olduğu gibi korunur — yani buradaki eksiklik veri kaybına değil,
 * yalnızca "o kural kontrolde görünmez" sonucuna yol açar.
 *
 * Liste Rust'a KOPYALANMIYOR: selector haritasının tek doğruluk kaynağı bu
 * dosya olsun diye her içe aktarmada parametre olarak gönderiliyor.
 *
 * `::before` / `::after` varyantları da dahil, çünkü temalar logo ve maskot
 * değişimini bu sözde elemanlar üzerinden yapıyor (bkz. TEMA-BULGULARI.md §3).
 */
export const KNOWN_SELECTORS: string[] = [
	CARD_SELECTOR,
	CARD_HOVER_SELECTOR,
	CARD_IMAGE_SELECTOR,
	SIDEBAR_SELECTOR,
	SIDEBAR_INDICATOR_SELECTOR,
	SIDEBAR_LABEL_SELECTOR,
	SCROLLBAR_SELECTOR,
	BADGE_SELECTOR,
	RELEASED_BADGE_SELECTOR,
	ENHANCED_SELECTOR,
	AVATAR_SELECTOR,
	BANNER_SELECTED_SELECTOR,
	BANNER_PROGRESS_SELECTOR,
	COMMENT_SELECTOR,
	COMMENT_INPUT_SELECTOR,
	PLAYER_BAR_SELECTOR,
	PLAYER_SCENE_BUTTON_SELECTOR,
	PLAYER_SLIDER_SELECTOR,
	PLAYER_RAIL_SELECTOR,
	PLAYER_TRACK_SELECTOR,
	PLAYER_THUMB_SELECTOR,
	PLAYER_EPISODE_SELECTOR,
	PLAYER_EPISODE_CURRENT_SELECTOR,
	PLAYER_CUE_SELECTOR,
	LOGO_IMAGE_HIDE_SELECTOR,
	LOGO_ICON_GUARD_SELECTOR,
	LOGO_TEXT_HIDE_SELECTOR,
	LOGO_IMAGE_SELECTOR,
	LOGO_TEXT_SELECTOR,
	LOGO_ROW_SELECTOR,
	LOGO_BADGE_SELECTOR,
	LOGO_NESTED_SELECTOR,
	BG_BODY_SELECTOR,
	BG_TRANSPARENT_SELECTOR,
	`${BG_BODY_SELECTOR}::before`,
	// Maskot yuvaları: her biri ayrı bir selector taşıyor.
	...MASCOT_SLOTS.map((slot) => slot.selector)
];

/** Temalarda kullanılan hazır Google Fonts seçenekleri. */
export const FONT_PRESETS = [
	{ name: "Site varsayılanı", family: "", importUrl: "" },
	{
		name: "Inter",
		family: "'Inter', sans-serif",
		importUrl: "https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap"
	},
	{
		name: "Permanent Marker",
		family: "'Permanent Marker', system-ui, sans-serif",
		importUrl: "https://fonts.googleapis.com/css2?family=Permanent+Marker&display=swap"
	}
];
