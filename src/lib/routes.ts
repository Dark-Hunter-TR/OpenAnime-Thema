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
	{ path: "/4k-releases", name: "4K çıkışlar" },
	{ path: "/episodes/latest/1", name: "Son bölümler" },
	{ path: "/episodes/populars/1", name: "Popüler bölümler" },
	{ path: "/recommendations", name: "Öneriler", auth: true },
	{ path: "/library", name: "Kitaplık", auth: true },
	{ path: "/activity/1", name: "Aktivite", auth: true },
	{ path: "/personalized-playlist", name: "Kişisel liste", auth: true },
	{ path: "/settings", name: "Ayarlar" },
	{ path: "/plus", name: "Plus" },
	{ path: "/login", name: "Giriş" },
	{ path: "/signup", name: "Kayıt" },
	{ path: "/about", name: "Hakkında" },
	{ path: "/tos", name: "Kullanım şartları" }
];

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
