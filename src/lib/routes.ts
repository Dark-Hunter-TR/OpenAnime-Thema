/**
 * openani.me'nin önizlemede gezilebilecek route'ları.
 *
 * Bu liste tahmin değil: sitenin SvelteKit route manifest'i canlı JS
 * bundle'larından çıkarıldı (`/src/routes/**​/+page.svelte` kayıtları).
 * Parametre isteyen route'lar (`/anime/[slug]`, `/room/[roomCode]` …) buraya
 * alınmadı — onlar için serbest adres kutusu var ya da önizleme içinde
 * doğrudan tıklanabilir.
 */
export interface PreviewRoute {
	path: string;
	name: string;
	/** Giriş yapılmamışsa ana sayfaya yönlendirir (307). */
	auth?: boolean;
}

export const ROUTES: PreviewRoute[] = [
	{ path: "/", name: "Ana sayfa" },
	{ path: "/explore", name: "Keşfet" },
	{ path: "/calendar", name: "Takvim" },
	// `/4k-releases` tek BAŞINA 404 veriyor — `/episodes/latest/1` gibi bu da
	// sayfalanmış bir liste, sayfa numarası segmenti ZORUNLU. Canlı sitede
	// doğrulandı: `/4k-releases` -> 404, `/4k-releases/1` -> 200.
	{ path: "/4k-releases/1", name: "4K çıkışlar" },
	{ path: "/episodes/latest/1", name: "Son bölümler" },
	{ path: "/episodes/populars/1", name: "Popüler bölümler" },
	{ path: "/recommendations", name: "Öneriler", auth: true },
	{ path: "/library", name: "Kitaplık", auth: true },
	{ path: "/activity/1", name: "Aktivite", auth: true },
	{ path: "/personalized-playlist", name: "Kişisel liste", auth: true },
	{ path: "/settings", name: "Ayarlar" },
	{ path: "/plus", name: "Plus" },
	{ path: "/about", name: "Hakkında" },
	{ path: "/tos", name: "Kullanım şartları" }
];

// `/login` ve `/signup` bilerek yok — ikisi de canlı sitede 404 veriyor.
// Sebebi tahmin değil: sitenin giriş/kayıt akışı ayrı bir SAYFA değil,
// `window.__SECRET_INTERNALS_DO_NOT_USE_OR_YOU_WILL_BE_FIRED.openAuthDialog(...)`
// ile açılan bir DİYALOG (bkz. `preview_init.js` başlığındaki not — aynı
// köprü zaten uygulamanın kendi `LoginDialog.svelte`'i tarafından kullanılıyor).
// Yani gidilecek ayrı bir adres hiç yok; kaldırmak eksik değil, doğru davranış.

/**
 * Parametre isteyen route şablonları — adres kutusunda ipucu olarak gösterilir.
 */
export const PARAM_ROUTES = [
	"/anime/<slug>",
	"/anime/<slug>/<sezon>/<bölüm>",
	"/fansub/<slug>",
	"/profile/<userId>",
	"/playlist/<playlistId>"
];

export const SITE_ORIGIN = "https://openani.me";

/** Viewport simülasyonu. Genişlik `null` ise mevcut alanın tamamı kullanılır. */
export interface Viewport {
	id: "desktop" | "tablet" | "mobile";
	name: string;
	width: number | null;
	/** Önizlemenin üstündeki floating seçicide gösterilen Fluent ikonu. */
	icon: "desktop" | "tablet" | "mobile";
}

export const VIEWPORTS: Viewport[] = [
	{ id: "desktop", name: "Masaüstü", width: null, icon: "desktop" },
	{ id: "tablet", name: "Tablet", width: 834, icon: "tablet" },
	{ id: "mobile", name: "Mobil", width: 390, icon: "mobile" }
];
