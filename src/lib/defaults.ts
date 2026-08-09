/**
 * Varsayılan değerlerin TEK kaynağı.
 *
 * İki tür varsayılan var ve ikisi de tahmin DEĞİL:
 *
 * 1. **Token varsayılanları** — `catalog.generated.ts` üzerinden
 *    fluent-svelte-extra'nın `theme.css` / `switchable.css` dosyalarından
 *    okunur. Moda (açık/koyu) ve o anki accent rampasına göre çözülür.
 *
 * 2. **Sayısal/yapısal varsayılanlar** — openani.me'nin canlı CSS
 *    bundle'larından ölçülmüştür. Her birinin yanında hangi kuraldan geldiği
 *    yazıyor; bir değeri değiştirmeden önce o kuralı tekrar doğrulayın.
 *
 * Kontrol paneli hiç değiştirilmeden açıldığında önizleme sitenin orijinal
 * hâliyle birebir aynı görünmeli — bu dosya o garantiyi sağlıyor.
 */

import { FDS_TOKENS } from "$lib/catalog.generated";
import { resolveTokenDefault, type ColorTokenSpec } from "$lib/customization";

export interface ColorState {
	hex: string;
	alpha: number;
}

/** Bir token'ın o moddaki HAM varsayılanı (henüz `var()` çözülmemiş). */
export function catalogDefault(token: string, mode: string): string | undefined {
	const info = FDS_TOKENS.find((t) => t.name === token);
	if (!info) return undefined;
	const effectiveMode =
		mode === "system"
			? typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: light)").matches
				? "light"
				: "dark"
			: mode;
	return effectiveMode === "light" ? (info.light ?? info.root) : (info.dark ?? info.root);
}

/**
 * Tek bir token'ı kontrol değerine çevirir.
 *
 * Çözülemeyen tek durum, varsayılanın kendisinin bir `var()` zinciri olması —
 * o zaman `fallbackAlpha` ile beyaza düşüyoruz. Kullanıcı zaten o alanı
 * açtığında kendi rengini seçecek.
 */
export function seedColor(
	token: string,
	mode: string,
	ramp: string[],
	fallbackAlpha = 100
): ColorState {
	return (
		resolveTokenDefault(catalogDefault(token, mode), ramp) ?? {
			hex: "#ffffff",
			alpha: fallbackAlpha
		}
	);
}

/** Bir spec listesinin tamamını tohumlar. */
export function seedColors(specs: ColorTokenSpec[], mode: string, ramp: string[]): ColorState[] {
	return specs.map((spec) => seedColor(spec.token, mode, ramp, spec.defaultAlpha));
}

/**
 * Sitenin canlı CSS'inden ölçülmüş sayısal varsayılanlar.
 *
 * Kaynak: openani.me'nin 52 CSS bundle'ı (aynı kaynak `bun run catalog`'un da
 * kullandığı). Yorumdaki kural, değeri bulduğumuz yerdir.
 */
export const SITE_DEFAULTS = {
	// .anime-card { border-radius: var(--fds-overlay-corner-radius) }
	// --fds-overlay-corner-radius kütüphanede 8px. Kart yarıçapı için 12
	// yazmak sitenin kendi değerinden sapmaktı.
	cardRadius: 8,
	// .anime-card.hoverable:hover { transform: translateY(-2px) }
	cardLift: 2,
	// .anime-card { box-shadow: var(--fds-card-shadow) }  ->  hover: --fds-flyout-shadow
	// Parıltı sitede yok; kapalı başlar.
	cardGlowBlur: 18,
	// .grid-view-item { border-radius: var(--fds-control-corner-radius) }
	gridRadius: 4,

	// .slider.orientation-horizontal .slider-rail { block-size: 4px }
	playerRailHeight: 4,
	// .slider.orientation-horizontal { block-size: 32px }
	playerSliderHeight: 32,

	// #progress { height: .3rem; background: #fff; border-radius: 50px }
	bannerProgressHeight: 5,
	bannerProgressColor: "#ffffff",
	bannerProgressRadius: 50,

	// .os-theme-dark, .os-theme-light { --os-size: 10px }
	scrollbarSize: 10,
	// .os-scrollbar { --os-track-border-radius: 50px }
	scrollbarTrackRadius: 50,
	// .os-theme-* { --os-handle-border-radius: 10px }
	scrollbarHandleRadius: 10,

	// .default-avatar { --fds-person-picture-size: 32px }  (üst çubuktaki avatar)
	avatarSize: 32,

	// .sidebar { min-width: 4.5rem; max-width: 4.5rem }  ->  72px
	sidebarWidth: 72,
	// .list-item::before { inline-size: .25rem; block-size: 1.5rem; border-radius: 999px } (4px x 24px capsule)
	sidebarIndicatorWidth: 4,
	sidebarIndicatorHeight: 24,
	sidebarIndicatorRadius: 999,

	// .topbar .logo img { width: 1.1rem; height: 1.1rem }  ->  17.6px
	logoSize: 18,
	// .topbar .logo img { margin-right: 1rem }  — logo ile yazı arası
	logoGap: 16,
	// Logo metni için sitede ayrı bir boyut yok; gövde boyutu kullanılır.
	// --fds-body-font-size: 14px
	logoTextSize: 14,
	// Uzun site adı "NEXT-GEN" rozetinin üstüne taşmasın diye üst sınır.
	logoTextMaxWidth: 180,

	// Maskotun sitede GERÇEKTEN sabit boyutu olan örnekleri:
	//   .header-right #notification-setsuki { width: 170px; height: 170px }
	//   .header-right #download-setsuki     { width: 170px; height: 170px }
	//   #mobile-notification-setsuki        { width: 150px; height: 150px }
	// #setsuki ve .setsuki #image akışkan (height:100%/width:auto) — onlara
	// sabit boyut vermek düzeni bozardı, o yüzden boyut kontrolleri yok.
	mascotSizes: {
		notification: 170,
		download: 170,
		mobileNotification: 150
	} as Record<string, number>,

	// #badge { background: linear-gradient(135deg, hsl(var(--fds-accent-light-1)) 0%,
	//          var(--fds-accent-default) 100%) }
	// Accent'ten türediği için sabit hex yerine accent rampasından çözülür.
	badgeGradientAngle: 135,
	// .released-badge { background-image: linear-gradient(to right, #6371da,
	//                   var(--fds-accent-tertiary)) }
	releasedBadgeFrom: "#6371da",
	// .enhanced-highlight { background: linear-gradient(96.58deg, #66faff -100%, #196a91) }
	enhancedFrom: "#66faff",
	enhancedTo: "#196a91",
	enhancedAngle: 96.58,

	// Sitenin süre token'ları (customization.ts'teki DURATION_TOKENS ile aynı).
	motionScale: 1,

	// --fds-control-corner-radius: 4px, --fds-overlay-corner-radius: 8px
	controlRadius: 4,
	overlayRadius: 8,

	// Yazı tipi ölçeği: 1 = sitenin kendi boyutları.
	fontScale: 1
};
// `as const` BİLEREK yok: bu değerler kontrollerin başlangıç değeri olarak
// atanıyor ve sonra kullanıcı tarafından değiştiriliyor. `as const` onları
// literal tipe (`4`, `8`, `1`) daraltıp her atamayı tip hatasına çevirirdi.

/**
 * `--fds-*-font-size` token'larının kütüphanedeki gerçek değerleri.
 * Ölçek kaydırıcısı bunları çarpar; yeni bir tipografi sistemi kurmaz.
 */
export const FONT_SIZE_TOKENS: { token: string; base: number; label: string }[] = [
	{ token: "--fds-caption-font-size", base: 12, label: "Açıklama" },
	{ token: "--fds-body-font-size", base: 14, label: "Gövde" },
	{ token: "--fds-body-large-font-size", base: 18, label: "Büyük gövde" },
	{ token: "--fds-subtitle-font-size", base: 20, label: "Alt başlık" },
	{ token: "--fds-title-font-size", base: 28, label: "Başlık" },
	{ token: "--fds-title-large-font-size", base: 40, label: "Büyük başlık" },
	{ token: "--fds-display-font-size", base: 68, label: "Display" }
];
