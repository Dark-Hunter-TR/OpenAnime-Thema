/**
 * Yeni özelleştirme bölümlerinin tanımları.
 *
 * Buradaki her token, openani.me'nin CANLI CSS'i taranarak seçildi — yeni bir
 * sistem icat edilmedi. Parantez içindeki sayılar, o token'ın sitede kaç
 * kuralda geçtiğini gösteriyor (52 CSS bundle üzerinden ölçüldü).
 */

export interface ColorTokenSpec {
	token: string;
	label: string;
	hint: string;
	/** Alfa kanalı anlamlı mı? Hover dolguları yarı saydam. */
	alpha: boolean;
	defaultAlpha: number;
}

/** :hover kurallarında gerçekten kullanılan token'lar. */
export const HOVER_TOKENS: ColorTokenSpec[] = [
	{
		token: "--fds-subtle-fill-secondary",
		label: "Hover dolgusu",
		hint: "Sitedeki 107 hover kuralının 25'i bunu kullanıyor — en yaygın olanı",
		alpha: true,
		defaultAlpha: 6
	},
	{
		token: "--fds-subtle-fill-tertiary",
		label: "Basılı dolgu",
		hint: "Tıklama anı (:active)",
		alpha: true,
		defaultAlpha: 4
	},
	{
		token: "--fds-control-fill-secondary",
		label: "Kontrol hover",
		hint: "Buton ve giriş alanlarının hover'ı (9 kural)",
		alpha: true,
		defaultAlpha: 8
	},
	{
		token: "--fds-control-fill-tertiary",
		label: "Kontrol basılı",
		hint: "Buton ve giriş alanlarının :active hâli",
		alpha: true,
		defaultAlpha: 6
	},
	{
		token: "--fds-accent-secondary",
		label: "Vurgu hover",
		hint: "Accent butonların hover'ı (7 kural)",
		alpha: true,
		defaultAlpha: 90
	},
	{
		token: "--fds-accent-tertiary",
		label: "Vurgu basılı",
		hint: "Accent butonların :active hâli",
		alpha: true,
		defaultAlpha: 80
	}
];

/** Buton görünümünü belirleyen token'lar (vurgu renginden bağımsız). */
export const BUTTON_TOKENS: ColorTokenSpec[] = [
	{
		token: "--fds-control-fill-default",
		label: "Arkaplan",
		hint: "Standart butonun dolgusu",
		alpha: true,
		defaultAlpha: 70
	},
	{
		token: "--fds-control-stroke-default",
		label: "Kenarlık",
		hint: "Buton çerçevesi",
		alpha: true,
		defaultAlpha: 10
	},
	{
		token: "--fds-accent-default",
		label: "Accent buton arkaplanı",
		hint: "variant=accent butonların dolgusu",
		alpha: false,
		defaultAlpha: 100
	},
	{
		token: "--fds-text-on-accent-primary",
		label: "Accent buton metni",
		hint: "Accent buton üzerindeki yazı rengi",
		alpha: false,
		defaultAlpha: 100
	}
];

/**
 * Standart butonun metin rengi için ayrılmış bir token yok — site
 * `--fds-text-primary`'yi kullanıyor, onu ezmek tüm metinleri değiştirirdi.
 * Bu yüzden sadece bu biri kural düzeyinde ezilir. `.button`,
 * fluent-svelte'in kendi public class'ı (Svelte hash'i değil), Stable.
 */
export const BUTTON_TEXT_SELECTOR = ".button";

/**
 * Bağlantı renkleri — vurgu renginden BAĞIMSIZ ayarlanabilir.
 *
 * Dayanak sitenin kendi kuralı:
 *   .button.style-hyperlink { color: var(--fds-accent-text-primary) }
 *   .button.style-hyperlink:active { color: var(--fds-accent-text-tertiary) }
 * Yani site linkleri `--fds-accent-*` rampasından değil, ayrı bir
 * `--fds-accent-text-*` setinden boyuyor. Bağlantı metinleri genel
 * vurgu renginden bağımsız ayarlanabilir.
 */
export const LINK_TOKENS: ColorTokenSpec[] = [
	{
		token: "--fds-accent-text-primary",
		label: "Bağlantı rengi",
		hint: ".button.style-hyperlink'in normal rengi",
		alpha: false,
		defaultAlpha: 100
	},
	{
		token: "--fds-accent-text-secondary",
		label: "Bağlantı (ikincil)",
		hint: "İkincil vurgu metinleri",
		alpha: false,
		defaultAlpha: 100
	},
	{
		token: "--fds-accent-text-tertiary",
		label: "Bağlantı (basılı)",
		hint: "Tıklama anındaki renk",
		alpha: false,
		defaultAlpha: 100
	}
];

/**
 * Sayfa zemini, panel yüzeyleri ve katmanlar için temel renk tonları.
 */
export const SURFACE_TOKENS: ColorTokenSpec[] = [
	{
		token: "--fds-solid-background-base",
		label: "Sayfa zemini",
		hint: "En alttaki opak sayfa zemini",
		alpha: true,
		defaultAlpha: 100
	},
	{
		token: "--fds-solid-background-secondary",
		label: "İkincil zemin",
		hint: "Kenar çubuğu / panel yüzeyi",
		alpha: true,
		defaultAlpha: 100
	},
	{
		token: "--fds-solid-background-tertiary",
		label: "Üçüncül zemin",
		hint: "Yükseltilmiş yüzeyler",
		alpha: true,
		defaultAlpha: 100
	},
	{
		token: "--fds-layer-background-default",
		label: "Katman",
		hint: "Kartların altındaki yarı saydam katman",
		alpha: true,
		defaultAlpha: 30
	},
	{
		token: "--fds-layer-background-alt",
		label: "Katman (alternatif)",
		hint: "İç içe katman yüzeyi",
		alpha: true,
		defaultAlpha: 100
	},
	{
		token: "--fds-smoke-background-default",
		label: "Diyalog karartması",
		hint: "Açılan diyalogların arkasındaki perde",
		alpha: true,
		defaultAlpha: 30
	}
];

/**
 * Durum renkleri. Sitenin `.info-badge.severity-*` kurallarında kullanılır.
 */
export const SYSTEM_TOKENS: ColorTokenSpec[] = [
	{
		token: "--fds-system-attention",
		label: "Bilgi",
		hint: ".info-badge.severity-attention uyarısı",
		alpha: false,
		defaultAlpha: 100
	},
	{
		token: "--fds-system-success",
		label: "Başarı",
		hint: ".info-badge.severity-success",
		alpha: false,
		defaultAlpha: 100
	},
	{
		token: "--fds-system-caution",
		label: "Uyarı",
		hint: ".info-badge.severity-caution",
		alpha: false,
		defaultAlpha: 100
	},
	{
		token: "--fds-system-critical",
		label: "Hata",
		hint: ".info-badge.severity-critical",
		alpha: false,
		defaultAlpha: 100
	}
];

/**
 * Kenar çubuğundaki seçili öğenin dolgusu.
 *
 * Sitenin kendi kuralı: `.sidebar a.selected { background-color:
 * var(--fds-control-solid-fill-default) }`. Hover/basılı hâller zaten
 * `--fds-subtle-fill-*` üzerinden "Hover ve tıklama renkleri" bölümünde.
 */
export const SIDEBAR_SELECTED_TOKEN: ColorTokenSpec = {
	token: "--fds-control-solid-fill-default",
	label: "Seçili öğe dolgusu",
	hint: ".sidebar a.selected'in arkaplanı",
	alpha: true,
	defaultAlpha: 100
};

/** Sitenin transition'larında kullandığı süre token'ları ve varsayılanları. */
export const DURATION_TOKENS: { token: string; base: number; label: string }[] = [
	{ token: "--fds-control-faster-duration", base: 83, label: "Çok hızlı" },
	{ token: "--fds-control-fast-duration", base: 167, label: "Hızlı" },
	{ token: "--fds-control-normal-duration", base: 250, label: "Normal" },
	{ token: "--fds-control-slow-duration", base: 333, label: "Yavaş" }
];

export const EASING_TOKEN = "--fds-control-fast-out-slow-in-easing";

export const EASINGS = [
	{ name: "Varsayılan (Fluent)", value: "cubic-bezier(0, 0, 0, 1)" },
	{ name: "Doğrusal", value: "linear" },
	{ name: "Yumuşak giriş-çıkış", value: "cubic-bezier(0.4, 0, 0.2, 1)" },
	{ name: "Sert duruş", value: "cubic-bezier(0.16, 1, 0.3, 1)" },
	{ name: "Zıplayan", value: "cubic-bezier(0.34, 1.56, 0.64, 1)" }
];

/**
 * Sitenin üst çubuğundaki logo: `<img src="/favicon.png" alt="logo">`,
 * `.topbar .logo` içinde. Selector'ı Svelte hash'i olmadan yazıyoruz
 * (`.topbar .logo img`) — hash her deploy'da değişir, bu iki class değişmez.
 */
export const LOGO_SELECTOR = ".topbar .logo img";
export const TITLE_SELECTOR = ".topbar .logo::after";

// --- Renk yardımcıları -------------------------------------------------------

/** `#rrggbb` -> `[r, g, b]`. Geçersizse `null`. */
export function hexToRgb(hex: string): [number, number, number] | null {
	const clean = hex.trim().replace(/^#/, "");
	const full =
		clean.length === 3
			? clean
					.split("")
					.map((c) => c + c)
					.join("")
			: clean;
	if (!/^[0-9a-fA-F]{6}$/.test(full)) return null;
	return [
		parseInt(full.slice(0, 2), 16),
		parseInt(full.slice(2, 4), 16),
		parseInt(full.slice(4, 6), 16)
	];
}

/** Kontrol değerlerini CSS'e çevirir. Alfa 100 ise opak `#hex` yazılır. */
export function toCssColor(hex: string, alphaPercent: number): string | null {
	const rgb = hexToRgb(hex);
	if (!rgb) return null;
	if (alphaPercent >= 100) return `#${hex.trim().replace(/^#/, "").toLowerCase()}`;
	const a = Math.max(0, Math.min(100, alphaPercent)) / 100;
	return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${Number(a.toFixed(3))})`;
}

/** CSS değerini kontrol durumuna geri çözer (kod editöründen gelen metin için). */
export function fromCssColor(value: string): { hex: string; alpha: number } | null {
	const text = value.trim();

	const rgba = text.match(/^rgba?\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([\d.]+)\s*)?\)$/);
	if (rgba) {
		const [r, g, b] = [Number(rgba[1]), Number(rgba[2]), Number(rgba[3])];
		const alpha = rgba[4] === undefined ? 100 : Math.round(Number(rgba[4]) * 100);
		const hex = [r, g, b].map((n) => n.toString(16).padStart(2, "0")).join("");
		return { hex: `#${hex}`, alpha };
	}

	if (/^#[0-9a-fA-F]{3}$|^#[0-9a-fA-F]{6}$/.test(text)) {
		const rgb = hexToRgb(text);
		if (!rgb) return null;
		const hex = rgb.map((n) => n.toString(16).padStart(2, "0")).join("");
		return { hex: `#${hex}`, alpha: 100 };
	}

	const hsl = text.match(
		/^hsla?\(\s*([\d.]+)(?:deg)?\s*,\s*([\d.]+)%\s*,\s*([\d.]+)%\s*(?:,\s*([\d.]+)(%?)\s*)?\)$/
	);
	if (hsl) {
		const rgb = hslToRgb(Number(hsl[1]), Number(hsl[2]), Number(hsl[3]));
		const hex = rgb.map((n) => n.toString(16).padStart(2, "0")).join("");
		let alpha = 100;
		if (hsl[4] !== undefined) {
			// Alfa hem `0.06` hem `6.05%` biçiminde gelebiliyor.
			alpha = hsl[5] === "%" ? Number(hsl[4]) : Number(hsl[4]) * 100;
		}
		return { hex: `#${hex}`, alpha: Math.round(alpha) };
	}

	return null;
}

/** HSL (0-360, 0-100, 0-100) -> RGB (0-255). */
export function hslToRgb(h: number, s: number, l: number): [number, number, number] {
	const sat = s / 100;
	const lig = l / 100;
	const c = (1 - Math.abs(2 * lig - 1)) * sat;
	const hp = (((h % 360) + 360) % 360) / 60;
	const x = c * (1 - Math.abs((hp % 2) - 1));
	const [r1, g1, b1] =
		hp < 1
			? [c, x, 0]
			: hp < 2
				? [x, c, 0]
				: hp < 3
					? [0, c, x]
					: hp < 4
						? [0, x, c]
						: hp < 5
							? [x, 0, c]
							: [c, 0, x];
	const m = lig - c / 2;
	return [
		Math.round((r1 + m) * 255),
		Math.round((g1 + m) * 255),
		Math.round((b1 + m) * 255)
	];
}

/** Accent rampasındaki basamak adları — katalog varsayılanlarını çözerken gerekli. */
const RAMP_ORDER = [
	"accent-light-3",
	"accent-light-2",
	"accent-light-1",
	"accent-base",
	"accent-dark-1",
	"accent-dark-2",
	"accent-dark-3"
];

/**
 * Katalogdaki ham varsayılanı kontrol değerine çevirir.
 *
 * Bazı varsayılanlar doğrudan renk değil, accent'e referans veriyor:
 * `hsl(var(--fds-accent-light-2))`. Bunları o anki rampadan çözüyoruz ki
 * kullanıcı gerçek rengi görsün — beyaz bir kutu değil.
 */
export function resolveTokenDefault(
	raw: string | undefined,
	ramp: string[]
): { hex: string; alpha: number } | null {
	if (!raw) return null;

	const resolved = raw.replace(/var\(\s*--fds-(accent-[a-z0-9-]+)\s*\)/g, (whole, name: string) => {
		const index = RAMP_ORDER.indexOf(name);
		return index >= 0 && ramp[index] ? ramp[index] : whole;
	});

	// `var(...)` kaldıysa çözemedik demektir.
	if (resolved.includes("var(")) return null;

	return fromCssColor(resolved);
}
