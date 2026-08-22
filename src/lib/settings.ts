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
import type { UpdateChannel } from "$lib/updater";

const KEY = "oa-editor-settings";

/**
 * Rich Presence'ın hangi ekranlarda görüneceği.
 *
 * `editor`: yalnızca bir tema açıkken (oluşturma ve düzenleme aynı ekran).
 * Ana ekranda, Ayarlar'da ve Hakkında'da aktivite tamamen temizlenir.
 */
export type DiscordScope = "always" | "editor";

/**
 * Açık tema yokken NavRail'deki "Editör" düğmesine tıklanınca sorulacak eylem.
 *
 * `editorQuickStart` KAPALIYKEN bu değerin hiçbir etkisi yok — seçici her
 * seferinde çıkar. Açıldığında bu değer hangi eylemin doğrudan çalışacağını
 * belirler; `"ask"` seçiliyse (kullanıcı bilerek bunu seçmiş olabilir) açık
 * olsa bile yine seçici çıkar. Bkz. `editorQuickStart`.
 */
export type EditorStartupAction = "ask" | "new" | "github" | "file";

export interface AppSettings {
	version: 2;
	/** Editör arayüzünün kendi teması — önizlemedeki siteyi etkilemez. */
	appTheme: ThemeMode;
	/**
	 * Editör'e hızlı başlama açık mı?
	 *
	 * Ana anahtar — Discord'daki `discordRpc` ile aynı kalıp. Kapalıyken
	 * (varsayılan) `editorStartupAction` ne olursa olsun seçici her zaman
	 * çıkar; bu, mevcut kullanıcıların davranışını sessizce değiştirmemek
	 * için varsayılan olarak kapalı tutuluyor. Kullanıcı bir kez hangi yolu
	 * hep seçtiğini fark edip açtığında, kapatması seçimi UNUTMAZ — yalnızca
	 * seçiciyi tekrar devreye sokar.
	 */
	editorQuickStart: boolean;
	/**
	 * Açık tema yokken Editör'e girildiğinde ne yapılacağı.
	 * Yalnızca `editorQuickStart` açıkken uygulanır. Bkz. `EditorStartupAction`.
	 */
	editorStartupAction: EditorStartupAction;
	/** Bir proje açıldığında hangi düzenleme modunda başlanacağı. */
	defaultEditMode: "visual" | "code";
	defaultViewport: "desktop" | "tablet" | "mobile";
	/** Önizlemenin açılışta gideceği sayfa. */
	defaultPreviewPath: string;
	/** Editörden ana ekrana dönerken açık proje otomatik kaydedilsin mi? */
	autoSaveOnLeave: boolean;
	/** Açılışta güncelleme olup olmadığı sessizce kontrol edilsin mi? */
	updateAutoCheck: boolean;
	/**
	 * "Daha sonra hatırlat" ile geçilen sürüm.
	 *
	 * Boş dize = hiçbir sürüm atlanmadı. Kullanıcı bir güncellemeyi
	 * ertelediğinde uygulama her açılışta aynı diyaloğu tekrar tekrar
	 * göstermemeli — yalnızca DAHA YENİ bir sürüm çıktığında tekrar sorulmalı.
	 */
	updateSkipVersion: string;
	/**
	 * Hangi yayın kanalından güncelleme alınacağı.
	 *
	 * Kanallar depoda ayrı manifest dosyaları (bkz. `src-tauri/src/updater.rs`).
	 * Stable kanaldaki bir kullanıcıya ön-sürüm asla sunulmaz; filtreleme
	 * istemcide değil, hangi dosyanın okunduğunda gerçekleşiyor.
	 *
	 * Varsayılan `stable`: ön-sürümler bilerek seçilmesi gereken bir şey.
	 */
	updateChannel: UpdateChannel;
	/**
	 * Discord'da "OpenAnime Theme oynuyor" olarak görünülsün mü?
	 *
	 * Ana anahtar. Kapalıyken aşağıdaki iki ayarın hiçbir etkisi yok ve Rust
	 * tarafı mevcut aktiviteyi Discord'dan temizliyor.
	 */
	discordRpc: boolean;
	/** Presence'ın görüneceği ekranlar. */
	discordRpcScope: DiscordScope;
	/**
	 * Düzenlenen temanın ADI da paylaşılsın mı?
	 *
	 * Ayrı bir anahtar, çünkü paylaşılan bilginin hassasiyeti diğerlerinden
	 * farklı: "tema düzenliyor" herkese açık bir etkinlik, temanın adı ise
	 * kullanıcının yayımlamadığı bir çalışmayı ele verebilir. Kapalıyken
	 * presence çalışmaya devam eder, yalnızca ad yerine genel bir satır yazar.
	 */
	discordRpcThemeName: boolean;
	/** Editördeki sol panelin genişliği (px). Kullanıcı ayraçtan sürükleyip ayarlar. */
	panelWidth: number;
}

export const DEFAULT_SETTINGS: AppSettings = {
	version: 2,
	appTheme: "system",
	editorQuickStart: false,
	editorStartupAction: "ask",
	defaultEditMode: "visual",
	defaultViewport: "desktop",
	defaultPreviewPath: "/",
	autoSaveOnLeave: true,
	updateAutoCheck: true,
	updateSkipVersion: "",
	updateChannel: "stable",
	discordRpc: true,
	discordRpcScope: "always",
	discordRpcThemeName: true,
	panelWidth: 420
};

/** `panelWidth` için izin verilen aralık — sürüklerken ve yüklenirken uygulanır. */
export const PANEL_WIDTH_MIN = 300;
export const PANEL_WIDTH_MAX = 760;

export function clampPanelWidth(width: number): number {
	return Math.min(PANEL_WIDTH_MAX, Math.max(PANEL_WIDTH_MIN, Math.round(width)));
}

export function loadSettings(): AppSettings {
	try {
		const stored = localStorage.getItem(KEY);
		if (!stored) return { ...DEFAULT_SETTINGS };
		// `version` kasıtlı olarak GENİŞ: diskteki değer bizim tanıdığımız
		// sürümlerden biri olmak zorunda değil (eski kurulum, elle düzenleme,
		// ileri sürümden geri dönüş). Dar tipte bırakılsaydı aşağıdaki sürüm
		// karşılaştırmaları derleyiciye göre "imkânsız" görünürdü.
		const parsed = JSON.parse(stored) as Omit<Partial<AppSettings>, "version"> & {
			version?: number;
		};

		// v1 -> v2: Discord ayarları eklendi ve eski tekil `discordRpc`
		// varsayılanı KAPALI idi. Eski değeri taşımak yerine düşürüyoruz:
		// v1'de bu alan hiçbir zaman kalıcı olamıyordu (her açılışta
		// varsayılanların üzerine yazılıyordu, bkz. `+page.svelte` içindeki
		// yükleme sırası), dolayısıyla saklanan `false` kullanıcının tercihi
		// değil, o hatanın kalıntısı. Geri kalan alanlar korunuyor — sürümü
		// tümden reddetmek kullanıcının tema/önizleme tercihlerini de
		// silerdi.
		if (parsed.version === 1) {
			const { discordRpc: _legacy, ...rest } = parsed;
			return { ...DEFAULT_SETTINGS, ...rest, version: 2 };
		}

		// Tanımadığımız sürüm: eksik alanla açılan bir editör, hiç ayar
		// olmamasından daha kötü davranırdı.
		if (parsed.version !== 2) return { ...DEFAULT_SETTINGS };
		return { ...DEFAULT_SETTINGS, ...parsed, version: 2 };
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
 * hex/RGB kutularının zeminleri — satır içi stille boyandığı için anında
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
