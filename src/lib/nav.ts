/**
 * Uygulamanın üst düzey görünümleri.
 *
 * Ayrı bir dosyada, çünkü hem `NavRail.svelte` hem `+page.svelte` bu tipi
 * kullanıyor ve Svelte bileşenlerinin instance script'inden tip export
 * edilemiyor.
 *
 * SvelteKit route'u AÇMIYORUZ. Önizleme, ana pencereye `add_child` ile
 * eklenmiş NATIVE bir webview ve her zaman host sayfanın içeriğinin üstüne
 * çiziliyor; route değişimi onun ömrünü ve konumunu yönetmeyi gereksiz yere
 * karmaşıklaştırırdı. Görünüm değişimi bu yüzden basit bir durum makinesi.
 */
export type NavId = "home" | "editor" | "settings" | "about";
