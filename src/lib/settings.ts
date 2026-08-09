/**
 * Uygulamanın kendi tercihleri.
 *
 * Bunlar TEMA değil, editörün nasıl açılacağına dair ayarlar. Bu yüzden proje
 * dosyalarında değil, uygulama genelinde tek bir yerde tutuluyorlar.
 *
 * Depolama olarak `localStorage` kullanılıyor: ayarlar birkaç yüz bayt, tek
 * makineye ait ve kaybolmaları yıkıcı değil. Projeleri diske yazan Rust
 * yoluna sokmak, karşılığında hiçbir şey kazandırmadan iki IPC komutu daha
 * eklerdi.
 */

import type { ThemeMode } from "$lib/theme";

const KEY = "oa-editor-settings";

export interface AppSettings {
	version: 1;
	/** Editör arayüzünün kendi teması — önizlemedeki siteyi etkilemez. */
	appTheme: ThemeMode;
	/** Bir proje açıldığında hangi düzenleme modunda başlanacağı. */
	defaultEditMode: "visual" | "code";
	defaultViewport: "desktop" | "tablet" | "mobile";
	/** Önizlemenin açılışta gideceği sayfa. */
	defaultPreviewPath: string;
	/** Editörden ana ekrana dönerken açık proje otomatik kaydedilsin mi? */
	autoSaveOnLeave: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
	version: 1,
	appTheme: "system",
	defaultEditMode: "visual",
	defaultViewport: "desktop",
	defaultPreviewPath: "/",
	autoSaveOnLeave: true
};

export function loadSettings(): AppSettings {
	try {
		const stored = localStorage.getItem(KEY);
		if (!stored) return { ...DEFAULT_SETTINGS };
		const parsed = JSON.parse(stored) as Partial<AppSettings>;
		// Sürüm uymuyorsa varsayılana dön: eksik alanla açılan bir editör,
		// hiç ayar olmamasından daha kötü davranırdı.
		if (parsed.version !== 1) return { ...DEFAULT_SETTINGS };
		return { ...DEFAULT_SETTINGS, ...parsed, version: 1 };
	} catch {
		return { ...DEFAULT_SETTINGS };
	}
}

export function saveSettings(settings: AppSettings): void {
	try {
		localStorage.setItem(KEY, JSON.stringify(settings));
	} catch {
		// Kota dolu ya da depolama kapalı — ayarlar kalıcı olmaz ama uygulama
		// çalışmaya devam etmeli.
	}
}

/**
 * Tema değişimi sırasında geçişleri kapatan sınıf.
 * Kuralı `+layout.svelte` içinde (global olmak zorunda).
 */
const SWITCHING_CLASS = "oa-theme-switching";

/**
 * En son uygulanan mod.
 *
 * `applyAppTheme` her ayar değişiminde (ör. varsayılan viewport) yeniden
 * çağrılıyor; tema gerçekten değişmediyse DOM'a dokunmanın ve geçişleri
 * kapatıp açmanın anlamı yok.
 */
let applied: ThemeMode | null = null;

/**
 * Editör arayüzünün temasını uygular.
 *
 * Sitenin kullandığı mekanizmanın aynısı: `switchable.css` `.fds-theme-light`
 * ve `.fds-theme-dark` sınıflarını tanımlıyor, sınıf yokken `theme.css`'in
 * `prefers-color-scheme` blokları devreye giriyor. Yani "sistem" = sınıf yok.
 *
 * ## Geçiş neden tek karede bitirilmek zorunda
 *
 * fluent-svelte-extra bileşenlerinde 30 kadar `transition:` kuralı var ve
 * bunlar rengi 83–250ms boyunca YUMUŞATARAK değiştiriyor. Buna karşılık
 * editördeki renk alanları — renk örnekleri, ColorPicker'ın palet yüzeyi,
 * hex/RGB kutularının zeminleri — satır içi stille boyandığı için ANINDA
 * değişiyor. Sonuç: sınıf değiştiği anda ekranın bir kısmı yeni paleti,
 * bir kısmı hâlâ eskisini gösteriyor ve arada tutarsız, "bozuk" görünen
 * kareler oluşuyor. En belirgin hâli "Sistem"e geçişte, çünkü işletim
 * sisteminin tercihi tersse palet tümüyle ters çevriliyor.
 *
 * Çözüm renk verisini değiştirmek değil — o zaten doğru; geçişleri bir
 * karelik süreyle kapatmak. Böylece bütün yüzeyler yeni değerlere AYNI
 * boyamada oturuyor ve ara durum hiç görünmüyor.
 */
export function applyAppTheme(mode: ThemeMode): void {
	if (applied === mode) return;
	applied = mode;

	const root = document.documentElement;

	root.classList.add(SWITCHING_CLASS);

	root.classList.remove("fds-theme-light", "fds-theme-dark");
	if (mode === "light") {
		root.classList.add("fds-theme-light");
		root.style.colorScheme = "light";
	} else if (mode === "dark") {
		root.classList.add("fds-theme-dark");
		root.style.colorScheme = "dark";
	} else {
		root.style.colorScheme = "light dark";
	}

	// Yeni değerleri bu karede hesaplat; aksi hâlde tarayıcı sınıf ekleme ile
	// kaldırmayı tek bir düzene toplayıp geçişleri yine çalıştırabiliyor.
	void root.offsetHeight;

	// İki kare bekliyoruz: birincisi yeni paletin boyandığı kare, ikincisinde
	// geçişleri geri açıyoruz. Tek kare bazı makinelerde erken kalıyordu.
	requestAnimationFrame(() => {
		requestAnimationFrame(() => root.classList.remove(SWITCHING_CLASS));
	});
}
