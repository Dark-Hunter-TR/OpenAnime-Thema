import { invoke } from "@tauri-apps/api/core";

export type ThemeMode = "system" | "light" | "dark";

/** Rust tarafındaki `ThemeDoc` ile birebir aynı şekil (serde camelCase). */
export interface ThemeDoc {
	accent: [number, number, number];
	mode: ThemeMode;
	controlCornerRadius: number | null;
	overlayCornerRadius: number | null;
	/** `@import` URL'leri — CSS'in en başına yazılır (yazı tipi yüklemek için). */
	imports: string[];
	/** Tam property adı -> değer. Hover, süreler, buton dolguları buradan geçer. */
	tokenOverrides: Record<string, string>;
	/** Selector -> bildirim gövdesi. Token karşılığı olmayanlar (logo, buton metni). */
	ruleOverrides: Record<string, string>;
	rawCss: string;
}

export interface ApplyResult {
	css: string;
	/** Yedi accent basamağı, `"206, 100%, 42%"` formatında. */
	ramp: string[];
}

/** fluent-svelte-extra `theme.css` varsayılanı. */
export const DEFAULT_ACCENT: [number, number, number] = [206, 100, 42];

export function defaultDoc(): ThemeDoc {
	return {
		accent: [...DEFAULT_ACCENT],
		mode: "dark",
		controlCornerRadius: null,
		overlayCornerRadius: null,
		imports: [],
		tokenOverrides: {},
		ruleOverrides: {},
		rawCss: ""
	};
}

export const applyTheme = (doc: ThemeDoc) => invoke<ApplyResult>("apply_theme", { doc });

/**
 * Kod editöründeki metni uygular ve geri çözümlenmiş dokümanı döndürür.
 * `applyTheme`'in ters yönü — görsel kontroller ile kod editörü aynı state
 * üzerinde çalışsın diye.
 */
export const applyCssText = (text: string) => invoke<ThemeDoc>("apply_css_text", { text });

/** Harici bir .css dosyasını okur (yol, dosya seçicisinden gelir). */
export const readCssFile = (path: string) => invoke<string>("read_css_file", { path });

/** Düzenlenen CSS'i aynı dosyaya geri yazar. */
export const writeCssFile = (path: string, contents: string) =>
	invoke<void>("write_css_file", { path, contents });

/**
 * Bir görseli `data:` URI'sine çevirir.
 *
 * Tema tek bir CSS metni olarak taşındığı için görselin yolunu değil gömülü
 * hâlini saklıyoruz — aksi hâlde tema başka makinede açılınca görsel kaybolur.
 */
export const readImageDataUri = (path: string) =>
	invoke<string>("read_image_data_uri", { path });

export const setPreviewBounds = (x: number, y: number, width: number, height: number) =>
	invoke<void>("set_preview_bounds", { x, y, width, height });

export const previewNavigate = (url: string) => invoke<void>("preview_navigate", { url });

export interface LoginState {
	loggedIn: boolean;
}

/**
 * Önizlemede openani.me oturumu açık mı?
 *
 * Rust, önizleme webview'inin çerez kavanozundaki `token` çerezine bakıyor —
 * sitenin bütün giriş yolları bu çerezi yazıyor, çıkış ise süresini geçmişe
 * çekiyor (bkz. `lib.rs` -> `preview_login_state`). Yalnızca VARLIĞI
 * okunuyor; token'ın değeriyle siteye istek atılmıyor.
 */
export const previewLoginState = () => invoke<LoginState>("preview_login_state");

/**
 * Ardışık çağrıları tek bir çağrıya indirger.
 *
 * Slider sürüklerken saniyede onlarca olay geliyor; her birini IPC'ye
 * göndermek gereksiz. 16ms ≈ bir kare, yani kullanıcı gecikmeyi fark etmiyor
 * ama IPC trafiği bir kata iniyor.
 */
export function debounce<A extends unknown[]>(fn: (...args: A) => void, ms = 16) {
	let timer: ReturnType<typeof setTimeout> | undefined;
	return (...args: A) => {
		if (timer) clearTimeout(timer);
		timer = setTimeout(() => fn(...args), ms);
	};
}

/** Hazır accent paletleri — taban (H, S, L) olarak. */
export const PRESETS: { name: string; hsl: [number, number, number] }[] = [
	{ name: "Varsayılan", hsl: [206, 100, 42] },
	{ name: "Sakura", hsl: [340, 82, 52] },
	{ name: "Mor", hsl: [280, 70, 50] },
	{ name: "Zümrüt", hsl: [152, 76, 36] },
	{ name: "Amber", hsl: [38, 95, 48] },
	{ name: "Mercan", hsl: [12, 88, 55] },
	{ name: "Çelik", hsl: [210, 12, 46] }
];
