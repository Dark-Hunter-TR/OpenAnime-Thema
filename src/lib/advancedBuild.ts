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

import { parseDeclarations } from "$lib/cssDecls";
import {
	AVATAR_SELECTOR,
	BADGE_SELECTOR,
	BADGE_TEXT_HIDE_SELECTOR,
	BADGE_TEXT_SELECTOR,
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
	ENHANCED_TEXT_HIDE_SELECTOR,
	ENHANCED_TEXT_SELECTOR,
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
	RELEASED_TEXT_HIDE_SELECTOR,
	RELEASED_TEXT_SELECTOR,
	SCROLLBAR_SELECTOR,
	SIDEBAR_INDICATOR_SELECTOR,
	SIDEBAR_SELECTOR,
	TEXT_TOKENS
} from "$lib/advanced";
import {
	DEFAULT_ACCENT_RAMP,
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
		/**
		 * NEXT-GEN rozetini gizle.
		 *
		 * Eskiden rozet hiçbir koşulda gizlenmiyordu ve arayüz bunu bir kural
		 * olarak duyuruyordu. Oysa topluluk temaları rozeti rutin biçimde
		 * gizliyor (`.topbar a.logo #badge { display: none !important }`) —
		 * yani engel teknik değil, bizim eksiğimizdi.
		 */
		badgeHidden: boolean;
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
		/**
		 * Rozetleri tamamen gizle.
		 *
		 * Renkten bağımsız tutuluyor: kullanıcı rozeti kaldırmak isterken
		 * renklerini de özelleştirmek zorunda kalmamalı.
		 */
		badgeHidden: boolean;
		releasedHidden: boolean;
		enhancedHidden: boolean;
		/**
		 * Rozetin yazısı. Boşsa sitenin kendi yazısı kalır.
		 *
		 * Değiştirmenin CSS'teki tek yolu, orijinal metni basan elemanı gizleyip
		 * yerine `::after` ile `content` yazmak — logoda da aynı kalıp
		 * kullanılıyor (bkz. `advanced.ts` -> `BADGE_TEXT_SELECTOR`).
		 */
		badgeText: string;
		releasedText: string;
		enhancedText: string;
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
function rampHex(ramp: string[], index: number): string {
	// Rampa boşsa (uygulama açılırken ilk `applyTheme` daha dönmedi) ya da
	// bozuksa kütüphanenin VARSAYILAN rampasına düşülüyor — elle yazılmış bir
	// yedek hex'e değil. Gerekçe: `DEFAULT_ACCENT_RAMP`.
	const step = ramp[index] ?? DEFAULT_ACCENT_RAMP[index];
	const parts = (step ?? "").split(",").map((p) => parseFloat(p));
	if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) {
		// Buraya yalnızca `index` rampanın dışındaysa düşülür; o bir kodlama
		// hatası olur ve sessizce yanlış bir renk yazmaktansa görünür olmalı.
		throw new RangeError(`rampHex: geçersiz rampa basamağı (${index})`);
	}
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
const RAMP_LIGHT_2 = 1;
const RAMP_LIGHT_1 = 2;
const RAMP_BASE = 3;
const RAMP_DARK_1 = 4;

/**
 * `--fds-accent-default`ın hangi rampa basamağından türediği.
 *
 * Kipe BAĞLI ve `accent-base` DEĞİL. Sitenin canlı CSS'inden okundu:
 *
 *   açık kip:  --fds-accent-default: hsl(var(--fds-accent-dark-1))
 *   koyu kip:  --fds-accent-default: hsla(var(--fds-accent-light-2))
 *
 * Burada bir zamanlar `accent-base` kullanılıyordu ve koyu kipte belirgin bir
 * hataya yol açıyordu: bu token'dan türeyen her varsayılan (NEXT-GEN rozetinin
 * bitiş rengi, "yayınlandı" rozeti, banner çerçevesi, oynatıcı rayı…) açık
 * mavi olması gerekirken koyu mavi çıkıyordu.
 */
const accentDefaultStep = (mode: string) => (mode === "light" ? RAMP_DARK_1 : RAMP_LIGHT_2);

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
			glowColor: rampHex(ramp, RAMP_LIGHT_1),
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
			maxWidth: SITE_DEFAULTS.logoTextMaxWidth,
			// Varsayılan sitenin kendi hâli: rozet görünür.
			badgeHidden: false
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
			indicatorColor: rampHex(ramp, accentDefaultStep(effectiveMode))
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
			// #badge: linear-gradient(135deg, hsl(var(--fds-accent-light-1)) 0%,
			//         var(--fds-accent-default) 100%)
			//
			// İkinci durak `--fds-accent-default`; hangi basamaktan türediği
			// KİPE BAĞLI (gerekçe: `accentDefaultStep`).
			badgeFrom: rampHex(ramp, RAMP_LIGHT_1),
			badgeTo: rampHex(ramp, accentDefaultStep(effectiveMode)),
			// .released-badge: linear-gradient(to right, #6371da, var(--fds-accent-tertiary))
			//
			// `--fds-accent-tertiary`, `--fds-accent-default` ile AYNI basamaktan
			// türüyor, üstüne %80 alfa alıyor. Sekiz basamaklı hex ile alfa
			// taşınıyor (`cc` = %80).
			releasedFrom: SITE_DEFAULTS.releasedBadgeFrom,
			releasedTo: `${rampHex(ramp, accentDefaultStep(effectiveMode))}cc`,
			// .enhanced-highlight: linear-gradient(96.58deg, #66faff -100%, #196a91)
			enhancedFrom: SITE_DEFAULTS.enhancedFrom,
			enhancedTo: SITE_DEFAULTS.enhancedTo,
			// Varsayılan sitenin kendi hâli: rozetler görünür, yazıları sitenin.
			badgeHidden: false,
			releasedHidden: false,
			enhancedHidden: false,
			badgeText: "",
			releasedText: "",
			enhancedText: ""
		},
		// .default-avatar { --fds-person-picture-size: 32px }
		avatar: { on: false, size: SITE_DEFAULTS.avatarSize },
		banner: {
			on: false,
			// .slider-card.selected { outline-color: var(--fds-accent-default) }
			outlineColor: rampHex(ramp, accentDefaultStep(effectiveMode)),
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
			focusColor: rampHex(ramp, accentDefaultStep(effectiveMode))
		},
		player: {
			on: false,
			// .bottom-controls'un kendi arkaplanı yok; şeffaftan başlıyoruz.
			barBg: color("#000000", 0),
			iconColor: "#ffffff",
			// .slider-rail { block-size: 4px }
			progressHeight: SITE_DEFAULTS.playerRailHeight,
			railColor: seedColor("--fds-control-strong-fill-default", mode, ramp, 54),
			trackColor: rampHex(ramp, accentDefaultStep(effectiveMode)),
			thumbColor: rampHex(ramp, accentDefaultStep(effectiveMode)),
			glow: false,
			episodeBg: seedColor("--fds-card-background-default", mode, ramp, 5),
			currentColor: rampHex(ramp, accentDefaultStep(effectiveMode)),
			cueOn: false,
			cueSize: 20,
			cueColor: "#ffffff",
			cueOutline: true
		}
	};
}

/**
 * Bir bölümün "kapalıyken tabandan tazelenen" hâlini kurar.
 *
 * Kapalı bölümlerin değerleri sürekli tabandan yeniden yazılıyor; amaç,
 * kontrol kutularında sitenin (ya da içe aktarılan temanın) gerçek değerinin
 * görünmesi. Ama bölümün `on` anahtarına BAĞLI OLMAYAN alanlar da varsa o
 * tazeleme onları da eziyordu.
 *
 * Somut hata: rozetlerde "Rozeti gizle" anahtarı açılmıyordu. Anahtar
 * `adv.badges` nesnesinin içinde; kullanıcı açtığı anda tepkisel ifade
 * çalışıp `adv.badges`ı tabandan yeniden kuruyor ve anahtar geri kapanıyordu.
 * Yazı kutuları da aynı şekilde temizleniyordu.
 *
 * `keep` listesindeki alanlar korunuyor. Sıfırlama düğmesi bu yoldan
 * geçmiyor (`resetAdvSection` bölümü olduğu gibi tabana döndürür), yani
 * "hepsini sıfırla" davranışı değişmiyor.
 */
export function reseedSection<T extends object>(
	current: T,
	baseline: T,
	keep: readonly (keyof T)[]
): T {
	const next = structuredClone(baseline);
	for (const key of keep) next[key] = current[key];
	return next;
}

/** Rozetlerde renk anahtarına bağlı OLMAYAN alanlar. */
export const BADGE_INDEPENDENT_FIELDS = [
	"badgeHidden",
	"releasedHidden",
	"enhancedHidden",
	"badgeText",
	"releasedText",
	"enhancedText"
] as const;

/**
 * Tek bir bölümü varsayılanına döndürür.
 *
 * Yeni bir nesne döndürüyor (mutasyon yok) ki Svelte'in reaktivitesi
 * tetiklensin ve `adv = resetAdvSection(...)` yeterli olsun.
 */
export function resetAdvSection(
	adv: AdvState,
	section: AdvSection,
	baseline: AdvState
): AdvState {
	return { ...adv, [section]: structuredClone(baseline[section]) };
}

/** CSS metin değerini güvenle tırnaklar. */
const quote = (s: string) => `"${s.replace(/["\\]/g, "\\$&")}"`;

export function buildAdvImports(adv: AdvState): string[] {
	if (!adv.typo.on) return [];
	const preset = FONT_PRESETS[adv.typo.preset];
	return preset?.importUrl ? [preset.importUrl] : [];
}

/**
 * "Yüzeyler" bölümüne AİT token'lar.
 *
 * `--fds-layer-background-default` iki spec listesinde birden duruyor:
 * `CARD_TOKENS` ve `SURFACE_TOKENS`. İkisi de yazıldığında sonra gelen
 * kazanıyordu, yani "Kartlar → Katman arka planı" kontrolü, "Yüzeyler" bölümü
 * de açıkken hiçbir şey yapmıyordu. Ölçümle bulundu:
 * `bun scripts/audit-controls.mjs`.
 *
 * Çakışma sahiplik verilerek çözülüyor; token anlamca yüzey ailesinden
 * (kaydırma çubuğu izi, açılır katman zemini), o yüzden sahibi "Yüzeyler".
 * Kartlar tarafında hem yazılmıyor hem de kontrolü çizilmiyor.
 */
export const SURFACE_OWNED_TOKENS: ReadonlySet<string> = new Set(
	SURFACE_TOKENS.map((spec) => spec.token)
);

export function buildAdvTokens(adv: AdvState): Record<string, string> {
	const map: Record<string, string> = {};

	const put = (
		specs: { token: string; alpha: boolean }[],
		colors: ColorState[],
		skip?: ReadonlySet<string>
	) => {
		specs.forEach((spec, i) => {
			// Atlanan yuvanın İNDİSİ korunuyor: `colors` dizisi spec listesiyle
			// aynı uzunlukta ve kaydedilmiş projelerde de öyle. Listeyi
			// filtrelemek indisleri kaydırır ve eski projelerin renkleri birer
			// yuva kayardı.
			if (skip?.has(spec.token)) return;
			const state = colors[i];
			if (!state) return;
			const css = toCssColor(state.hex, spec.alpha ? state.alpha : 100);
			if (css) map[spec.token] = css;
		});
	};

	if (adv.text.on) put(TEXT_TOKENS, adv.text.colors);
	if (adv.cards.on) put(CARD_TOKENS, adv.cards.colors, SURFACE_OWNED_TOKENS);
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
		// NEXT-GEN rozeti: ya gizleniyor ya da yeri sabitleniyor.
		//
		// Gizlenmediğinde ezilmemesi gerekiyor; `::after` her zaman bir
		// elemanın SON kutusu olduğundan (site adından sonra basılır) rozeti
		// açıkça en sona (order: 2) sabitleyip gerçek DOM sırasından
		// bağımsızlaştırıyoruz.
		//
		// Düzen bildirimleri iki durumda da yazılıyor, `display` yalnızca
		// gizlerken ekleniyor. İki dal farklı özellik kümeleri yazsaydı
		// `display` kontrollerin SAHİPLENDİĞİ özellikler listesine girmez
		// (`controlledRuleProps`), kullanıcı rozeti tekrar gösterdiğinde eski
		// `display: none` gövdede takılı kalırdı.
		map[LOGO_BADGE_SELECTOR] =
			(adv.logo.badgeHidden ? "display: none !important; " : "") +
			"order: 2; flex: 0 0 auto !important; margin-left: 0 !important;";
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
	//
	// Üç yetenek birbirinden BAĞIMSIZ: renk ("Rozet renklerini özelleştir"),
	// gizleme ve yazı. Kullanıcı rozeti kaldırmak ya da yazısını değiştirmek
	// isterken renklerini de özelleştirmek zorunda kalmamalı, o yüzden hepsi
	// tek bir `on` anahtarına bağlanmıyor.
	{
		/**
		 * Bir rozetin gövdesini parça parça kurar; boşsa anahtar hiç yazılmaz.
		 *
		 * Gizleme, rengi BASTIRMIYOR — ikisi de yazılıyor. Bastırsaydı iki dal
		 * farklı özellik kümeleri üretirdi ve `controlledRuleProps` (kontrollerin
		 * sahiplendiği özellikler) hangi dal açıksa yalnızca onunkini görürdü:
		 * kullanıcı gizlemeyi kapattığında eski `display: none` gövdede takılı
		 * kalırdı. Gizli bir rozette fazladan duran renk bildiriminin ise
		 * görünür bir etkisi yok.
		 */
		const badgeBody = (color: string | null, hidden: boolean) => {
			const parts: string[] = [];
			if (hidden) parts.push("display: none !important;");
			if (color) parts.push(color);
			return parts.join(" ");
		};

		/**
		 * Yazı değiştirme: orijinal metni basan elemanı gizle, yerine `::after`
		 * ile yaz. Logodaki kalıbın aynısı; rozetin yazısını değiştirmenin
		 * CSS'te başka yolu yok.
		 *
		 * Gizleme burada da yazıyı BASTIRMIYOR; gerekçesi `badgeBody` ile aynı —
		 * bastırılsaydı yazı özellikleri kontrollerin sahiplendiği kümeden
		 * düşer ve gizleme kapatıldığında eski değerler gövdede kalırdı.
		 */
		const putText = (hideSel: string, afterSel: string, text: string, style: string) => {
			const value = text.trim();
			if (!value) return;
			map[hideSel] = "display: none !important;";
			map[afterSel] = `content: ${quote(value)}; ${style}`;
		};

		const badge = badgeBody(
			adv.badges.on
				? `background: linear-gradient(${SITE_DEFAULTS.badgeGradientAngle}deg, ` +
						`${adv.badges.badgeFrom} 0%, ${adv.badges.badgeTo} 100%) !important;`
				: null,
			adv.badges.badgeHidden
		);
		if (badge) map[BADGE_SELECTOR] = badge;

		const released = badgeBody(
			adv.badges.on
				? `background-image: linear-gradient(to right, ${adv.badges.releasedFrom}, ${adv.badges.releasedTo}) !important;`
				: null,
			adv.badges.releasedHidden
		);
		if (released) map[RELEASED_BADGE_SELECTOR] = released;

		const enhanced = badgeBody(
			adv.badges.on
				? `background: linear-gradient(${SITE_DEFAULTS.enhancedAngle}deg, ` +
						`${adv.badges.enhancedFrom} -100%, ${adv.badges.enhancedTo}) !important;`
				: null,
			adv.badges.enhancedHidden
		);
		if (enhanced) map[ENHANCED_SELECTOR] = enhanced;

		// Her rozetin yazı stili FARKLI; enjekte edilen metin orijinaliyle aynı
		// görünsün diye üçü de kendi ölçüsüyle yazılıyor. Değerler sitenin
		// canlı CSS'inden okundu (bkz. `SITE_DEFAULTS` başlığındaki kaynak
		// notu); tek bir ortak stil kullanıldığında rozet yazısı orijinalinden
		// belirgin biçimde büyük ve ince çıkıyordu.
		putText(
			BADGE_TEXT_HIDE_SELECTOR,
			BADGE_TEXT_SELECTOR,
			adv.badges.badgeText,
			// #badge .text-block
			"text-transform: uppercase; font-size: 10px; font-weight: 600; letter-spacing: .5px;"
		);
		putText(
			RELEASED_TEXT_HIDE_SELECTOR,
			RELEASED_TEXT_SELECTOR,
			adv.badges.releasedText,
			// .released-badge kendi yazı ölçüsünü ezmiyor; sitenin küçük metin
			// ailesi kullanılıyor.
			"font-family: var(--fds-font-family-small); font-size: var(--fds-caption-font-size); font-weight: 400;"
		);
		putText(
			ENHANCED_TEXT_HIDE_SELECTOR,
			ENHANCED_TEXT_SELECTOR,
			adv.badges.enhancedText,
			// .enhanced-highlight { font-size: 10px; line-height: 14px } +
			// içindeki .text-block { font-weight: 600 }
			"font-size: 10px; line-height: 14px; font-weight: 600;"
		);
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

/**
 * Bu modülün üretebileceği HER token anahtarı — bölümün açık/kapalı olması
 * önemsiz. `+page.svelte`, bir bölüm kapatıldığında `doc.tokenOverrides`
 * içindeki bu anahtarları silip ÖYLE yeniden birleştiriyor; aksi hâlde
 * `{...eski, ...yeni}` gibi bir SPREAD, `buildAdvTokens`'ın artık üretmediği
 * (yani kapatılmış bir bölüme ait) anahtarı silmez, eski değer kalıcı yapışıp
 * kalır ve "kapat" görsel olarak hiçbir şey yapmamış gibi görünür.
 */
export const ADV_TOKEN_KEYS: string[] = [
	...TEXT_TOKENS,
	...CARD_TOKENS,
	...SURFACE_TOKENS,
	...LINK_TOKENS,
	...SYSTEM_TOKENS
]
	.map((s) => s.token)
	.concat(SIDEBAR_SELECTED_TOKEN.token)
	.concat(FONT_TOKENS.map((t) => t.token))
	.concat(FONT_SIZE_TOKENS.map((t) => t.token));

/**
 * Kontrollerin bir seçicide YAZABİLECEĞİ özellik adları — seçici başına.
 *
 * Nerede kullanıldığı: `+page.svelte`, kontrollerin çıktısını mevcut
 * `ruleOverrides` haritasının üstüne bindirirken bu kümeye bakıyor. Kümedeki
 * özellikler kontrollerin sahibi sayılıp değiştiriliyor ya da (bölüm
 * kapatılmışsa) siliniyor; kümede OLMAYAN her bildirim olduğu gibi kalıyor.
 * Böylece harici bir dosyadan gelen ve hiçbir kontrolün karşılamadığı
 * bildirimler kaybolmuyor (gerekçe: `cssDecls.ts` -> `mergeDeclarations`).
 *
 * Liste elle tutulmuyor, `buildAdvRules`'ın "her bölüm açık" çıktısından
 * TÜRETİLİYOR. Elle tutulan bir liste bu modülle sessizce ayrı düşerdi: yeni
 * bir özellik eklendiğinde listeye yazmayı unutan biri, o özelliğin kontrolden
 * silinemediğini ancak kullanıcı şikâyet edince öğrenirdi.
 */
export function controlledRuleProps(): Record<string, Set<string>> {
	const everything = enableEverySection(defaultAdv());
	const out: Record<string, Set<string>> = {};

	for (const [selector, body] of Object.entries(buildAdvRules(everything))) {
		out[selector] = new Set(parseDeclarations(body).map((d) => d.property));
	}
	return out;
}

/** `controlledRuleProps` için: bütün bölümleri açık, bütün görselleri dolu bir durum. */
/**
 * Bütün bölümleri açık bir kopya döndürür.
 *
 * `controlledRuleProps` bunu zaten kullanıyordu; dışa açılmasının sebebi
 * denetim: kontrollerin yazabildiği token/kural yüzeyinin tamamını üretmek
 * (`scripts/dump-control-surface.mjs`) ancak her bölüm açıkken mümkün.
 */
export function enableEverySection(adv: AdvState): AdvState {
	const next: AdvState = structuredClone(adv);

	// Yalnızca bir yer tutucu; değerin kendisi önemsiz, çünkü çıktının
	// yalnızca ÖZELLİK ADLARI okunuyor. Boş bırakılırsa `buildAdvRules`
	// görsele bağlı kuralları hiç üretmez ve o kuralların özellikleri
	// listeden düşerdi.
	const PLACEHOLDER = "x";
	const FILLED_STRINGS = new Set(["dataUri", "text", "custom"]);

	/**
	 * Bu alan bir METİN/ADRES mi (yer tutucuyla doldurulmalı), yoksa renk mi
	 * (dokunulmamalı)?
	 *
	 * `Text` ekiyle biten her ad da sayılıyor. Sabit liste tek başına
	 * yetmiyordu: rozetlere `badgeText` / `releasedText` / `enhancedText`
	 * eklendiğinde listeye girmedikleri için boş kalıyor, `buildAdvRules` o
	 * kuralları hiç üretmiyor ve `controlledRuleProps` onları kontrollerin
	 * SAHİPLENDİĞİ özellikler arasında görmüyordu. Sonucu görünür bir hataydı:
	 * kullanıcı rozete yazı yazıp sonra siliyor, `mergeRuleOverrides` eski
	 * kuralı kaldıramıyor ve rozetin orijinal yazısı gizli kalmaya devam
	 * ediyordu.
	 */
	const isTextField = (key: string) => FILLED_STRINGS.has(key) || key.endsWith("Text");

	const walk = (node: Record<string, unknown>) => {
		for (const key of Object.keys(node)) {
			const value = node[key];
			if (typeof value === "boolean") {
				node[key] = true;
			} else if (typeof value === "string") {
				// Renk alanlarına dokunulmuyor: zaten dolular ve yer tutucu
				// yazmak geçersiz bir renk üretirdi.
				if (isTextField(key) && value === "") node[key] = PLACEHOLDER;
			} else if (Array.isArray(value)) {
				for (const item of value) {
					if (item && typeof item === "object") walk(item as Record<string, unknown>);
				}
			} else if (value && typeof value === "object") {
				walk(value as Record<string, unknown>);
			}
		}
	};
	walk(next as unknown as Record<string, unknown>);

	// Maskot görselleri bir sözlük; yuvalar önceden tanımlı olmadığı için
	// yukarıdaki gezinme onları dolduramıyor.
	for (const slot of MASCOT_SLOTS) next.mascot.images[slot.id] = PLACEHOLDER;

	return next;
}
