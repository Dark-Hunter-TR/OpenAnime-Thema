/**
 * Gelişmiş bölümlerin durumu -> CSS üretimi.
 *
 * Mevcut mimariye eklemedir: üretilen çıktı yine `tokenOverrides` /
 * `ruleOverrides` / `imports` haritalarına akar, dolayısıyla kod editörü ve
 * harici dosya senkronu bu bölümler için de kendiliğinden çalışır.
 *
 * İki kural:
 *   1. Her selector openani.me'nin canlı CSS'inde doğrulandı (bkz. advanced.ts).
 *   2. Her varsayılan sitenin gerçek değeri — `defaults.ts` ya da katalog.
 *      Bir bölüm kapalıyken hiçbir şey yazılmaz; açıkken varsayılan
 *      değerleriyle yazılan CSS sitenin kendi görünümüyle aynıdır.
 */

import {
	AVATAR_SELECTOR,
	BADGE_SELECTOR,
	BANNER_PROGRESS_SELECTOR,
	BANNER_SELECTED_SELECTOR,
	BG_BODY_SELECTOR,
	BG_TRANSPARENT_SELECTOR,
	CARD_HOVER_SELECTOR,
	CARD_IMAGE_SELECTOR,
	CARD_SELECTOR,
	CARD_TOKENS,
	COMMENT_INPUT_SELECTOR,
	COMMENT_SELECTOR,
	ENHANCED_SELECTOR,
	FONT_PRESETS,
	FONT_TOKENS,
	LOGO_BADGE_SELECTOR,
	LOGO_ICON_GUARD_SELECTOR,
	LOGO_IMAGE_HIDE_SELECTOR,
	LOGO_IMAGE_SELECTOR,
	LOGO_NESTED_SELECTOR,
	LOGO_ROW_SELECTOR,
	LOGO_TEXT_HIDE_SELECTOR,
	LOGO_TEXT_SELECTOR,
	MASCOT_SLOTS,
	PLAYER_BAR_SELECTOR,
	PLAYER_CUE_SELECTOR,
	PLAYER_EPISODE_CURRENT_SELECTOR,
	PLAYER_EPISODE_SELECTOR,
	PLAYER_RAIL_SELECTOR,
	PLAYER_SCENE_BUTTON_SELECTOR,
	PLAYER_SLIDER_SELECTOR,
	PLAYER_THUMB_SELECTOR,
	PLAYER_TRACK_SELECTOR,
	RELEASED_BADGE_SELECTOR,
	SCROLLBAR_SELECTOR,
	SIDEBAR_INDICATOR_SELECTOR,
	SIDEBAR_SELECTOR,
	TEXT_TOKENS
} from "$lib/advanced";
import {
	LINK_TOKENS,
	SIDEBAR_SELECTED_TOKEN,
	SURFACE_TOKENS,
	SYSTEM_TOKENS,
	toCssColor
} from "$lib/customization";
import { FONT_SIZE_TOKENS, SITE_DEFAULTS, seedColor, seedColors, type ColorState } from "$lib/defaults";

export type { ColorState };

export interface AdvState {
	text: { on: boolean; colors: ColorState[] };
	cards: {
		on: boolean;
		colors: ColorState[];
		radius: number;
		lift: number;
		borderWidth: number;
		glow: boolean;
		glowColor: string;
		maskOn: boolean;
		maskStart: number;
	};
	typo: { on: boolean; preset: number; custom: string; sizeOn: boolean; scale: number };
	bg: { on: boolean; dataUri: string; dim: number; blur: number };
	mascot: { images: Record<string, string>; sizeOn: boolean; sizes: Record<string, number> };
	logo: {
		imageOn: boolean;
		dataUri: string;
		size: number;
		gap: number;
		textOn: boolean;
		text: string;
		textSize: number;
		maxWidth: number;
	};
	sidebar: {
		on: boolean;
		width: number;
		selected: ColorState;
		indicatorWidth: number;
		indicatorHeight: number;
		indicatorRadius: number;
		indicatorColor: string;
	};
	surface: { on: boolean; colors: ColorState[] };
	links: { on: boolean; colors: ColorState[] };
	scrollbar: {
		on: boolean;
		size: number;
		handle: ColorState;
		track: ColorState;
		handleRadius: number;
		trackRadius: number;
	};
	badges: {
		on: boolean;
		badgeFrom: string;
		badgeTo: string;
		releasedFrom: string;
		releasedTo: string;
		enhancedFrom: string;
		enhancedTo: string;
	};
	avatar: { on: boolean; size: number };
	banner: {
		on: boolean;
		outlineColor: string;
		progressHeight: number;
		progressColor: string;
		progressRadius: number;
	};
	system: { on: boolean; colors: ColorState[] };
	comments: { on: boolean; bg: ColorState; radius: number; focusColor: string };
	player: {
		on: boolean;
		barBg: ColorState;
		iconColor: string;
		progressHeight: number;
		railColor: ColorState;
		trackColor: string;
		thumbColor: string;
		glow: boolean;
		episodeBg: ColorState;
		currentColor: string;
		cueOn: boolean;
		cueSize: number;
		cueColor: string;
		cueOutline: boolean;
	};
}

/** Bölüm anahtarları — sıfırlama ve yeniden tohumlama bunlar üzerinden. */
export type AdvSection = keyof AdvState;

const color = (hex: string, alpha = 100): ColorState => ({ hex, alpha });

/**
 * Bir accent basamağını hex'e çevirir. Rozet gradyanları sitede accent'ten
 * türediği için varsayılanları da oradan çözüyoruz — sabit bir mavi yazmak
 * kullanıcının seçtiği vurgu rengiyle çelişirdi.
 */
function rampHex(ramp: string[], index: number, fallback: string): string {
	const step = ramp[index];
	if (!step) return fallback;
	const parts = step.split(",").map((p) => parseFloat(p));
	if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) return fallback;
	const [h, s, l] = parts;
	const sat = s / 100;
	const lig = l / 100;
	const c = (1 - Math.abs(2 * lig - 1)) * sat;
	const hp = (((h % 360) + 360) % 360) / 60;
	const x = c * (1 - Math.abs((hp % 2) - 1));
	const [r1, g1, b1] =
		hp < 1 ? [c, x, 0] : hp < 2 ? [x, c, 0] : hp < 3 ? [0, c, x] : hp < 4 ? [0, x, c] : hp < 5 ? [x, 0, c] : [c, 0, x];
	const m = lig - c / 2;
	return (
		"#" +
		[r1, g1, b1]
			.map((v) => Math.round((v + m) * 255).toString(16).padStart(2, "0"))
			.join("")
	);
}

/** Rampa indeksleri (açıktan koyuya): 0=light-3 … 3=base … 6=dark-3. */
const RAMP_LIGHT_1 = 2;
const RAMP_BASE = 3;
const RAMP_DARK_1 = 4;

/**
 * Varsayılan durum.
 *
 * `mode` ve `ramp` alıyor çünkü renk varsayılanlarının çoğu moda ve o anki
 * accent rampasına bağlı. Parametresiz çağrılırsa koyu mod + kütüphane
 * varsayılan rampası varsayılır.
 */
export function defaultAdv(mode = "dark", ramp: string[] = []): AdvState {
	const effectiveMode =
		mode === "system"
			? typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: light)").matches
				? "light"
				: "dark"
			: mode;
	return {
		text: { on: false, colors: seedColors(TEXT_TOKENS, mode, ramp) },
		cards: {
			on: false,
			colors: seedColors(CARD_TOKENS, mode, ramp),
			// .anime-card { border-radius: var(--fds-overlay-corner-radius) } -> 8px
			radius: SITE_DEFAULTS.cardRadius,
			// .anime-card.hoverable:hover { transform: translateY(-2px) }
			lift: SITE_DEFAULTS.cardLift,
			// Sitede kartın kenarlığı yok; 0 = dokunma.
			borderWidth: 0,
			glow: false,
			glowColor: rampHex(ramp, RAMP_LIGHT_1, "#00a2ff"),
			maskOn: false,
			// Görsel altı maskeleme başlangıç seviyesi varsayılan %60
			maskStart: 60
		},
		typo: { on: false, preset: 0, custom: "", sizeOn: false, scale: SITE_DEFAULTS.fontScale },
		bg: { on: false, dataUri: "", dim: 40, blur: 0 },
		mascot: { images: {}, sizeOn: false, sizes: { ...SITE_DEFAULTS.mascotSizes } },
		logo: {
			imageOn: false,
			dataUri: "",
			// .topbar .logo img { width: 1.1rem } -> 17.6px
			size: SITE_DEFAULTS.logoSize,
			// .topbar .logo img { margin-right: 1rem }
			gap: SITE_DEFAULTS.logoGap,
			textOn: false,
			text: "",
			// --fds-body-font-size: 14px
			textSize: SITE_DEFAULTS.logoTextSize,
			maxWidth: SITE_DEFAULTS.logoTextMaxWidth
		},
		sidebar: {
			on: false,
			// .sidebar { min-width: 4.5rem; max-width: 4.5rem }
			width: SITE_DEFAULTS.sidebarWidth,
			selected: seedColor(SIDEBAR_SELECTED_TOKEN.token, mode, ramp, 100),
			// .list-item::before { inline-size: 3px; block-size: 16px; border-radius: 3px }
			indicatorWidth: SITE_DEFAULTS.sidebarIndicatorWidth,
			indicatorHeight: SITE_DEFAULTS.sidebarIndicatorHeight,
			indicatorRadius: SITE_DEFAULTS.sidebarIndicatorRadius,
			// background-color: var(--fds-accent-default)
			indicatorColor: rampHex(ramp, effectiveMode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_1, "#0078d4")
		},
		surface: { on: false, colors: seedColors(SURFACE_TOKENS, mode, ramp) },
		links: { on: false, colors: seedColors(LINK_TOKENS, mode, ramp) },
		scrollbar: {
			on: false,
			// .os-theme-dark/.os-theme-light { --os-size: 10px }
			size: SITE_DEFAULTS.scrollbarSize,
			// --os-handle-bg: var(--fds-control-strong-fill-default)
			handle: seedColor("--fds-control-strong-fill-default", mode, ramp, 54),
			// --os-track-bg-hover: var(--fds-layer-background-default)
			track: seedColor("--fds-layer-background-default", mode, ramp, 30),
			handleRadius: SITE_DEFAULTS.scrollbarHandleRadius,
			trackRadius: SITE_DEFAULTS.scrollbarTrackRadius
		},
		badges: {
			on: false,
			// #badge: linear-gradient(135deg, hsl(var(--fds-accent-light-1)), var(--fds-accent-default))
			badgeFrom: rampHex(ramp, RAMP_LIGHT_1, "#00a2ff"),
			badgeTo: rampHex(ramp, effectiveMode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_1, "#0078d4"),
			// .released-badge: linear-gradient(to right, #6371da, var(--fds-accent-tertiary))
			releasedFrom: SITE_DEFAULTS.releasedBadgeFrom,
			releasedTo: rampHex(ramp, RAMP_BASE, "#0078d4"),
			// .enhanced-highlight: linear-gradient(96.58deg, #66faff -100%, #196a91)
			enhancedFrom: SITE_DEFAULTS.enhancedFrom,
			enhancedTo: SITE_DEFAULTS.enhancedTo
		},
		// .default-avatar { --fds-person-picture-size: 32px }
		avatar: { on: false, size: SITE_DEFAULTS.avatarSize },
		banner: {
			on: false,
			// .slider-card.selected { outline-color: var(--fds-accent-default) }
			outlineColor: rampHex(ramp, effectiveMode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_1, "#0078d4"),
			// #progress { height: .3rem; background: #fff; border-radius: 50px }
			progressHeight: SITE_DEFAULTS.bannerProgressHeight,
			progressColor: SITE_DEFAULTS.bannerProgressColor,
			progressRadius: SITE_DEFAULTS.bannerProgressRadius
		},
		system: { on: false, colors: seedColors(SYSTEM_TOKENS, mode, ramp) },
		comments: {
			on: false,
			// Sitede yorumun kendi arkaplanı yok; kart yüzeyini temel alıyoruz.
			bg: seedColor("--fds-card-background-secondary", mode, ramp, 3),
			radius: SITE_DEFAULTS.overlayRadius,
			focusColor: rampHex(ramp, effectiveMode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_1, "#0078d4")
		},
		player: {
			on: false,
			// .bottom-controls'un kendi arkaplanı yok; şeffaftan başlıyoruz.
			barBg: color("#000000", 0),
			iconColor: "#ffffff",
			// .slider-rail { block-size: 4px }
			progressHeight: SITE_DEFAULTS.playerRailHeight,
			railColor: seedColor("--fds-control-strong-fill-default", mode, ramp, 54),
			trackColor: rampHex(ramp, effectiveMode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_1, "#0078d4"),
			thumbColor: rampHex(ramp, effectiveMode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_1, "#0078d4"),
			glow: false,
			episodeBg: seedColor("--fds-card-background-default", mode, ramp, 5),
			currentColor: rampHex(ramp, effectiveMode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_1, "#0078d4"),
			cueOn: false,
			cueSize: 20,
			cueColor: "#ffffff",
			cueOutline: true
		}
	};
}

/**
 * Tek bir bölümü varsayılanına döndürür.
 *
 * Yeni bir nesne döndürüyor (mutasyon yok) ki Svelte'in reaktivitesi
 * tetiklensin ve `adv = resetAdvSection(...)` yeterli olsun.
 */
export function resetAdvSection(
	adv: AdvState,
	section: AdvSection,
	mode: string,
	ramp: string[]
): AdvState {
	const fresh = defaultAdv(mode, ramp);
	return { ...adv, [section]: fresh[section] };
}

/** CSS metin değerini güvenle tırnaklar. */
const quote = (s: string) => `"${s.replace(/["\\]/g, "\\$&")}"`;

export function buildAdvImports(adv: AdvState): string[] {
	if (!adv.typo.on) return [];
	const preset = FONT_PRESETS[adv.typo.preset];
	return preset?.importUrl ? [preset.importUrl] : [];
}

export function buildAdvTokens(adv: AdvState): Record<string, string> {
	const map: Record<string, string> = {};

	const put = (specs: { token: string; alpha: boolean }[], colors: ColorState[]) => {
		specs.forEach((spec, i) => {
			const state = colors[i];
			if (!state) return;
			const css = toCssColor(state.hex, spec.alpha ? state.alpha : 100);
			if (css) map[spec.token] = css;
		});
	};

	if (adv.text.on) put(TEXT_TOKENS, adv.text.colors);
	if (adv.cards.on) put(CARD_TOKENS, adv.cards.colors);
	if (adv.surface.on) put(SURFACE_TOKENS, adv.surface.colors);
	if (adv.links.on) put(LINK_TOKENS, adv.links.colors);
	if (adv.system.on) put(SYSTEM_TOKENS, adv.system.colors);

	if (adv.sidebar.on) {
		const css = toCssColor(adv.sidebar.selected.hex, adv.sidebar.selected.alpha);
		if (css) map[SIDEBAR_SELECTED_TOKEN.token] = css;
	}

	if (adv.typo.on) {
		const preset = FONT_PRESETS[adv.typo.preset];
		const family = adv.typo.custom.trim() || preset?.family || "";
		if (family) for (const { token } of FONT_TOKENS) map[token] = family;
	}

	// Yazı boyutu ölçeği: sitenin KENDİ boyut token'larını çarpıyoruz,
	// yeni bir tipografi ölçeği icat etmiyoruz.
	if (adv.typo.on && adv.typo.sizeOn && adv.typo.scale !== 1) {
		for (const { token, base } of FONT_SIZE_TOKENS) {
			map[token] = `${Math.round(base * adv.typo.scale * 10) / 10}px`;
		}
	}

	// Not: `--fds-person-picture-size` bilerek burada değil. Site onu her
	// PersonPicture örneğinde ayrı veriyor; global ezmek kart içindeki
	// `inline-size: 100%` avatarları bozar. Kural olarak, üst çubukla
	// sınırlı biçimde `buildAdvRules` içinde yazılıyor.

	return map;
}

export function buildAdvRules(adv: AdvState): Record<string, string> {
	const map: Record<string, string> = {};

	// --- Kartlar -------------------------------------------------------------
	if (adv.cards.on) {
		const parts = [`border-radius: ${adv.cards.radius}px !important;`];
		if (adv.cards.borderWidth > 0) {
			parts.push(
				`border: ${adv.cards.borderWidth}px solid var(--fds-card-stroke-default) !important;`
			);
		}
		map[CARD_SELECTOR] = parts.join(" ");

		// Sitenin kendi hover'ı `translateY(-2px)`; yalnızca mesafeyi değiştiriyoruz.
		const hover: string[] = [`transform: translateY(-${adv.cards.lift}px) !important;`];
		if (adv.cards.glow) {
			hover.push(
				`box-shadow: 0 0 ${SITE_DEFAULTS.cardGlowBlur}px ${adv.cards.glowColor} !important;`
			);
		}
		map[CARD_HOVER_SELECTOR] = hover.join(" ");

		if (adv.cards.maskOn) {
			// Görselin alt kısmını yumuşak geçişle silikleştir.
			const mask = `linear-gradient(to bottom, #000 ${adv.cards.maskStart}%, transparent 100%)`;
			map[CARD_IMAGE_SELECTOR] =
				`-webkit-mask-image: ${mask} !important; mask-image: ${mask} !important;`;
		}
	}

	// --- Arkaplan ------------------------------------------------------------
	// Görsel sabit bir katmana basılıyor; üstteki yüzeyler şeffaflaştırılmazsa
	// görünmez. Temaların yaptığı da bu.
	if (adv.bg.on && adv.bg.dataUri) {
		const dim = Math.max(0, Math.min(100, adv.bg.dim)) / 100;
		map[BG_TRANSPARENT_SELECTOR] = "background-color: transparent !important;";
		map[`${BG_BODY_SELECTOR}::before`] =
			"content: \"\"; position: fixed; inset: 0; z-index: -1; " +
			`background-image: linear-gradient(rgba(0,0,0,${dim}), rgba(0,0,0,${dim})), url(${quote(adv.bg.dataUri)}); ` +
			"background-size: cover; background-position: center; background-repeat: no-repeat; " +
			(adv.bg.blur > 0 ? `filter: blur(${adv.bg.blur}px); transform: scale(1.05);` : "");
	}

	// --- Logo ve site adı ----------------------------------------------------
	// Gizleme kuralları bağımsız: yalnızca site adını değiştiren kullanıcının
	// logosu kaybolmasın diye (bkz. `advanced.ts` -> `LOGO_IMAGE_HIDE_SELECTOR`).
	const logoImageActive = adv.logo.imageOn && adv.logo.dataUri !== "";
	const logoTextActive = adv.logo.textOn && adv.logo.text.trim() !== "";

	if (logoImageActive) {
		map[LOGO_IMAGE_HIDE_SELECTOR] = "display: none !important;";
		map[LOGO_IMAGE_SELECTOR] =
			`content: ""; display: block; order: 0; flex: 0 0 auto; width: ${adv.logo.size}px; height: ${adv.logo.size}px; ` +
			`background: url(${quote(adv.logo.dataUri)}) no-repeat center / contain;`;
	} else if (logoTextActive) {
		// Görsel değişmiyor: orijinal ikon olduğu gibi kalmalı. Ama satır
		// artık flex (LOGO_ROW_SELECTOR) ve rozet sabit genişlik alıyor
		// (LOGO_BADGE_SELECTOR) — ikonun flex-shrink'i kapatılmazsa satırdaki
		// tek esnek öğe o kalır ve yer daralınca 0 genişliğe küçülüp
		// GÖRÜNMEZ olur. `order: 0` ile de en solda kalması garantilenir.
		map[LOGO_ICON_GUARD_SELECTOR] = "order: 0; flex: 0 0 auto !important;";
	}

	if (logoTextActive) {
		map[LOGO_TEXT_HIDE_SELECTOR] = "display: none !important;";
		map[LOGO_TEXT_SELECTOR] =
			`content: ${quote(adv.logo.text.trim())}; display: block; order: 1; ` +
			// Sitenin orijinal adı `.text-block.type-caption` ile basılıyor
			// (font-family-small, weight 400, line-height 16px) — burada da
			// aynı temel stil kullanılır ki yazı tipi orijinalinden farklı
			// görünmesin. Boyut kullanıcının kendi kaydırıcısından geliyor.
			`font-family: var(--fds-font-family-small); font-size: ${adv.logo.textSize}px; ` +
			"font-weight: 400; line-height: 16px; color: var(--fds-text-primary); " +
			// Rozetin üstüne taşmayı önleyen üçlü: küçülebilmesi için min-width,
			// bir üst sınır, ve taşarsa üç nokta.
			`flex: 0 1 auto; min-width: 0; max-width: ${adv.logo.maxWidth}px; ` +
			"overflow: hidden; text-overflow: ellipsis; white-space: nowrap;";
	}

	if (logoImageActive || logoTextActive) {
		// Satır düzeni: site zaten flex; biz yalnızca boşluğu ve küçülebilmeyi
		// veriyoruz. `min-width: 0` olmadan ellipsis çalışmaz.
		map[LOGO_ROW_SELECTOR] =
			`display: flex !important; align-items: center !important; ` +
			`gap: ${adv.logo.gap}px !important; min-width: 0 !important; overflow: hidden !important;`;
		// NEXT-GEN rozeti asla ezilmesin ve `::after` her zaman bir elemanın
		// SON kutusu olduğundan (site adından sonra basılır) rozeti açıkça
		// en sona (order: 2) sabitleyip gerçek DOM sırasından bağımsızlaştır.
		map[LOGO_BADGE_SELECTOR] = "order: 2; flex: 0 0 auto !important; margin-left: 0 !important;";
		// `a.logo-button > a.logo` iç içeyse çift basmayı engelle.
		map[LOGO_NESTED_SELECTOR] = "content: none !important; display: none !important;";
	}

	// --- Maskot --------------------------------------------------------------
	for (const slot of MASCOT_SLOTS) {
		const uri = adv.mascot.images[slot.id];
		const parts: string[] = [];
		if (uri) {
			// <img> üzerinde `content: url(...)` kaynağı değiştirir (Chromium).
			parts.push(`content: url(${quote(uri)});`, "object-fit: contain;");
		}
		// Boyut yalnızca sitede GERÇEKTEN sabit ölçüsü olan örneklere verilir;
		// akışkan olanlara (height:100%) sabit ölçü vermek düzeni bozardı.
		if (adv.mascot.sizeOn && slot.size !== null) {
			const size = adv.mascot.sizes[slot.id] ?? slot.size;
			parts.push(`width: ${size}px !important;`, `height: ${size}px !important;`);
		}
		if (parts.length) map[slot.selector] = parts.join(" ");
	}

	// --- Kenar çubuğu --------------------------------------------------------
	if (adv.sidebar.on) {
		map[SIDEBAR_SELECTOR] =
			`min-width: ${adv.sidebar.width}px !important; max-width: ${adv.sidebar.width}px !important;`;
		map[SIDEBAR_INDICATOR_SELECTOR] =
			`inline-size: ${adv.sidebar.indicatorWidth}px !important; ` +
			`block-size: ${adv.sidebar.indicatorHeight}px !important; ` +
			`border-radius: ${adv.sidebar.indicatorRadius}px !important; ` +
			`background-color: ${adv.sidebar.indicatorColor} !important;`;
	}

	// --- Kaydırma çubuğu -----------------------------------------------------
	// Sitenin kendi `--os-*` API'si. `::-webkit-scrollbar` bu sitede kapalı.
	if (adv.scrollbar.on) {
		const handle = toCssColor(adv.scrollbar.handle.hex, adv.scrollbar.handle.alpha);
		const track = toCssColor(adv.scrollbar.track.hex, adv.scrollbar.track.alpha);
		const decls = [`--os-size: ${adv.scrollbar.size}px;`];
		if (handle) {
			decls.push(
				`--os-handle-bg: ${handle} !important;`,
				`--os-handle-bg-hover: ${handle} !important;`,
				`--os-handle-bg-active: ${handle} !important;`
			);
		}
		if (track) {
			decls.push(
				`--os-track-bg-hover: ${track} !important;`,
				`--os-track-bg-active: ${track} !important;`
			);
		}
		decls.push(
			`--os-handle-border-radius: ${adv.scrollbar.handleRadius}px !important;`,
			`--os-track-border-radius: ${adv.scrollbar.trackRadius}px !important;`
		);
		map[SCROLLBAR_SELECTOR] = decls.join(" ");
	}

	// --- Rozetler ------------------------------------------------------------
	if (adv.badges.on) {
		map[BADGE_SELECTOR] =
			`background: linear-gradient(${SITE_DEFAULTS.badgeGradientAngle}deg, ` +
			`${adv.badges.badgeFrom} 0%, ${adv.badges.badgeTo} 100%) !important;`;
		map[RELEASED_BADGE_SELECTOR] =
			`background-image: linear-gradient(to right, ${adv.badges.releasedFrom}, ${adv.badges.releasedTo}) !important;`;
		map[ENHANCED_SELECTOR] =
			`background: linear-gradient(${SITE_DEFAULTS.enhancedAngle}deg, ` +
			`${adv.badges.enhancedFrom} -100%, ${adv.badges.enhancedTo}) !important;`;
	}

	// --- Profil fotoğrafı ----------------------------------------------------
	if (adv.avatar.on) {
		// Yalnızca üst çubuktaki profil görseline uygulanır — kart içindeki avatarları etkilemez.
		map[AVATAR_SELECTOR] =
			`--fds-person-picture-size: ${adv.avatar.size}px !important; ` +
			`min-width: ${adv.avatar.size}px !important; min-height: ${adv.avatar.size}px !important;`;
	}

	// --- Banner / kayan kartlar ----------------------------------------------
	if (adv.banner.on) {
		map[BANNER_SELECTED_SELECTOR] = `outline-color: ${adv.banner.outlineColor} !important;`;
		map[BANNER_PROGRESS_SELECTOR] =
			`height: ${adv.banner.progressHeight}px !important; ` +
			`background: ${adv.banner.progressColor} !important; ` +
			`border-radius: ${adv.banner.progressRadius}px !important;`;
	}

	// --- Yorumlar ------------------------------------------------------------
	if (adv.comments.on) {
		const bg = toCssColor(adv.comments.bg.hex, adv.comments.bg.alpha);
		map[COMMENT_SELECTOR] =
			(bg ? `background-color: ${bg} !important; ` : "") +
			`border-radius: ${adv.comments.radius}px !important; padding: 8px !important;`;
		map[`${COMMENT_INPUT_SELECTOR}:focus-within`] =
			`outline: 2px solid ${adv.comments.focusColor} !important; outline-offset: -2px;`;
	}

	// --- Oynatıcı ------------------------------------------------------------
	if (adv.player.on) {
		const barBg = toCssColor(adv.player.barBg.hex, adv.player.barBg.alpha);
		if (barBg) map[PLAYER_BAR_SELECTOR] = `background-color: ${barBg} !important;`;

		map[PLAYER_SCENE_BUTTON_SELECTOR] = `color: ${adv.player.iconColor} !important;`;
		map[`${PLAYER_SCENE_BUTTON_SELECTOR} svg, ${PLAYER_BAR_SELECTOR} button svg path`] =
			`fill: ${adv.player.iconColor} !important;`;

		map[PLAYER_SLIDER_SELECTOR] =
			`color: ${adv.player.trackColor} !important;` +
			(adv.player.glow ? ` filter: drop-shadow(0 0 4px ${adv.player.trackColor}) !important;` : "");

		const rail = toCssColor(adv.player.railColor.hex, adv.player.railColor.alpha);
		map[PLAYER_RAIL_SELECTOR] =
			`block-size: ${adv.player.progressHeight}px !important;` +
			(rail ? ` background-color: ${rail} !important;` : "");
		map[PLAYER_TRACK_SELECTOR] = `background-color: ${adv.player.trackColor} !important;`;
		map[PLAYER_THUMB_SELECTOR] = `background-color: ${adv.player.thumbColor} !important;`;

		const epBg = toCssColor(adv.player.episodeBg.hex, adv.player.episodeBg.alpha);
		if (epBg) map[PLAYER_EPISODE_SELECTOR] = `background-color: ${epBg} !important;`;
		map[PLAYER_EPISODE_CURRENT_SELECTOR] =
			`border-color: ${adv.player.currentColor} !important; ` +
			`box-shadow: 0 0 10px ${adv.player.currentColor}55 !important;`;

		if (adv.player.cueOn) {
			const outline = adv.player.cueOutline
				? " text-shadow: -1px -1px 0 #000, 1px -1px 0 #000, -1px 1px 0 #000, 1px 1px 0 #000;"
				: " text-shadow: none;";
			map[PLAYER_CUE_SELECTOR] =
				`font-size: ${adv.player.cueSize}px; color: ${adv.player.cueColor}; background: none;` +
				outline;
		}
	}

	return map;
}
