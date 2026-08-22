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
		mode: "system",
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
export const applyCssText = (text: string, knownSelectors: string[] = []) =>
	invoke<ThemeDoc>("apply_css_text", { text, knownSelectors });

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

/**
 * Önizlemenin çerezlerini ve site verilerini (localStorage, önbellek…) silip
 * siteyi baştan yükler.
 *
 * WebView2/wry yalnızca ayrım gözetmeyen tek bir "tüm gezinti verisini
 * temizle" komutu sunuyor — "yalnızca çerezler" ile "yalnızca site verisi"
 * ayrı işlemler DEĞİL (bkz. `preview.rs` -> `clear_data`).
 */
export const previewClearData = () => invoke<void>("preview_clear_data");

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

export interface LoginOutcome {
	/** `false` ise oturum açıldı ama e-posta doğrulaması bekliyor. */
	verified: boolean;
}

/**
 * openani.me'de oturum açar.
 *
 * Form uygulamanın kendisinde, ama İSTEK yine önizlemedeki sayfadan çıkıyor:
 * `POST /user/auth` de Vanguard'ın arkasında ve `Gateway-Token` başlığı
 * yalnızca sayfanın yamalı `fetch`'iyle eklenebiliyor. Yani önizlemenin
 * yüklenmiş olması bu çağrı için de şart.
 *
 * Parola buradan Rust'a, oradan sayfaya geçer ve orada kalır; uygulama onu
 * hiçbir yere yazmaz. Dönen erişim token'ı ise uygulamaya hiç uğramaz —
 * sayfa onu doğrudan çereze yazar (bkz. `preview_init.js` -> `__OA_API_LOGIN__`).
 */
export const accountLogin = (email: string, password: string) =>
	invoke<LoginOutcome>("account_login", { email, password });

/** QR akışından gelen tek bir olay (bkz. `lib.rs` -> `QrEvent`). */
export interface QrEvent {
	/** `"qr"` | `"success"` | `"error"` | `"idle"` */
	kind: string;
	/** `kind === "qr"`: gösterilecek QR görselinin kaynağı. */
	image?: string;
	/** `kind === "success"`: hesabın e-postası doğrulanmış mı. */
	verified?: boolean;
	/** `kind === "error"`: gösterilecek mesaj. */
	message?: string;
}

/**
 * QR (DAG) ile giriş akışından bir sonraki olayı bekler.
 *
 * Akış ilk çağrıda kendiliğinden açılıyor. Bir DÖNGÜDE çağrılmalı: `"idle"`
 * gelirse olay yok demektir, tekrar sorulur; `"qr"` gelince görsel tazelenir
 * (kod kısa aralıklarla yenileniyor); `"success"` ya da `"error"` döngüyü
 * bitirir. Bittiğinde `stopAccountQr` çağrılmalı ki akış kapansın.
 */
export const accountQrNext = () => invoke<QrEvent>("account_qr_next");

/** QR akışını kapatır. Hata yutuluyor — kapatma başarısızlığı kullanıcıyı
 * ilgilendiren bir şey değil. */
export const stopAccountQr = () => invoke<void>("account_qr_stop").catch(() => {});

/**
 * openani.me oturumunu kapatır.
 *
 * Sunucudaki `refreshToken`'ı iptal eder ve önizlemenin çerezlerini siler.
 * Sunucu isteği en iyi çaba — çıkış, geçit erişilemez olsa bile gerçekleşir
 * (bkz. `preview_init.js` -> `__OA_API_LOGOUT__`). Sonrasında önizleme
 * kendini yeniliyor ki site de kullanıcıyı çıkmış göstersin.
 */
export const accountLogout = () => invoke<void>("account_logout");

/**
 * Sunucunun İngilizce hata mesajını Türkçeye çevirir.
 *
 * Eşleştirmeler sitenin kendi çeviri dosyasından alındı; anahtarlar API'nin
 * `error` alanında döndürdüğü metinlerin birebir kendisi. Tanımadığımız bir
 * kod gelirse olduğu gibi gösteriyoruz — yanlış bir tahminle çevirmektense
 * ham mesajı göstermek daha yararlı.
 */
export function loginErrorText(raw: string): string {
	const map: Record<string, string> = {
		"Invalid password": "Parola hatalı.",
		"User not found": "Bu e-posta ile bir hesap bulunamadı.",
		"No such user found": "Bu e-posta ile bir hesap bulunamadı.",
		"Specify a valid e-mail address": "Geçerli bir e-posta adresi girin.",
		"Invalid data": "Geçersiz veri.",
		"Invalid body": "Hatalı istek içeriği.",
		"Invalid token": "Token geçersiz veya sunucu hatası oluştu.",
		"Captcha invalid": "Captcha geçersiz.",
		Unauthorized: "Yetersiz yetki."
	};
	return map[raw] ?? raw;
}

/** openani.me'nin `/user` uç noktasından dönen ham hesap nesnesi. Şekli sabit
 * değil — arayüz elindeki alanları genel biçimde gösterir (bkz. `AccountCard`). */
export type AccountInfo = Record<string, unknown>;

/**
 * Giriş yapmış kullanıcının hesap bilgilerini çeker.
 *
 * İstek uygulamadan değil, önizleme webview'inde açık olan openani.me
 * sayfasının İÇİNDEN atılıyor: `api.openani.me` "Vanguard" geçidinin
 * arkasında ve `Gateway-Token` başlığı olmayan her isteği — kimlik
 * doğrulama gerektirmeyenler dahil — 401'liyor. O başlığı sitenin kendi
 * `window.fetch` yaması ekliyor, değerini `/osc.wasm` 35 saniyede bir
 * yeniden imzalıyor; yani ne kopyalanabiliyor ne yeniden üretilebiliyor.
 *
 * Pratik sonuç: bu çağrı yalnızca önizleme yüklenmişken çalışır. Akışın
 * tamamı için `src-tauri/src/preview_init.js` -> `__OA_ACCOUNT_FETCH__` ve
 * `lib.rs` -> `fetch_account_info`.
 */
export const fetchAccountInfo = () => invoke<AccountInfo>("fetch_account_info");

/** Takipçi/takip edilen listesindeki bir kayıt (bkz. `lib.rs` -> `fetch_account_follows`). */
export interface FollowUser {
	id: string;
	username: string;
	avatar?: string;
}

/**
 * Takipçi ya da takip edilen listesini çeker.
 *
 * `fetchAccountInfo` ile aynı köprüyü kullanır — yani yine önizlemedeki
 * openani.me sayfasının içinden geçer. Uç noktalar sitenin kendi profil
 * diyaloglarıyla aynı: `/user/<id>/followers` ve `/user/<id>/following`.
 */
export const fetchAccountFollows = (userId: string, kind: "followers" | "following") =>
	invoke<FollowUser[]>("fetch_account_follows", { userId, kind });

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
