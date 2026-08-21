/**
 * `bun test` ile çalışır.
 *
 * Ayarların KALICILIĞI burada sınanıyor, çünkü bozulduğunda sessizce
 * bozuluyor: uygulama çalışmaya devam ediyor, yalnızca her açılışta
 * varsayılanlara dönüyor. Elle fark etmek için uygulamayı kapatıp açmak
 * gerekiyordu; bu dosya aynı şeyi milisaniyede yapıyor.
 */

import { beforeEach, expect, test } from "bun:test";

import { DEFAULT_SETTINGS, loadSettings, saveSettings } from "$lib/settings";

const KEY = "oa-editor-settings";

/**
 * Bun'da Web Storage yok; `settings.ts` yalnızca `getItem`/`setItem`
 * kullandığı için bu kadarı yeterli.
 */
function stubStorage(): Map<string, string> {
	const store = new Map<string, string>();
	(globalThis as { localStorage?: unknown }).localStorage = {
		getItem: (k: string) => store.get(k) ?? null,
		setItem: (k: string, v: string) => void store.set(k, v)
	};
	return store;
}

let store: Map<string, string>;
beforeEach(() => {
	store = stubStorage();
});

test("kaydedilen ayarlar aynen geri okunur", () => {
	saveSettings({ ...DEFAULT_SETTINGS, appTheme: "dark", defaultViewport: "mobile" });
	const loaded = loadSettings();
	expect(loaded.appTheme).toBe("dark");
	expect(loaded.defaultViewport).toBe("mobile");
});

test("hiç kayıt yoksa varsayılanlar gelir", () => {
	expect(loadSettings()).toEqual(DEFAULT_SETTINGS);
});

test("v1 kaydı v2'ye taşınır: diğer alanlar korunur, Discord varsayılana döner", () => {
	// v1'de `discordRpc` varsayılanı kapalıydı ve kalıcı olamıyordu; saklanan
	// `false` kullanıcının tercihi değil, o hatanın kalıntısı.
	store.set(
		KEY,
		JSON.stringify({ version: 1, appTheme: "light", autoSaveOnLeave: false, discordRpc: false })
	);

	const loaded = loadSettings();
	expect(loaded.version).toBe(2);
	expect(loaded.appTheme).toBe("light");
	expect(loaded.autoSaveOnLeave).toBe(false);
	expect(loaded.discordRpc).toBe(DEFAULT_SETTINGS.discordRpc);
	expect(loaded.discordRpcScope).toBe(DEFAULT_SETTINGS.discordRpcScope);
});

test("tanınmayan sürüm ve bozuk kayıt varsayılanlara düşer", () => {
	store.set(KEY, JSON.stringify({ version: 99, appTheme: "dark" }));
	expect(loadSettings()).toEqual(DEFAULT_SETTINGS);

	store.set(KEY, "{ bu JSON değil");
	expect(loadSettings()).toEqual(DEFAULT_SETTINGS);
});

test("güncelleme kanalı varsayılanda Stable ve tur atar", () => {
	// Kanal kalıcı bir kullanıcı tercihi: yanlış okunursa ön-sürüm kullanan
	// biri sessizce Stable kanala düşer (ya da tersi).
	expect(DEFAULT_SETTINGS.updateChannel).toBe("stable");

	saveSettings({ ...DEFAULT_SETTINGS, updateChannel: "alpha" });
	expect(loadSettings().updateChannel).toBe("alpha");
});

test("Discord ayarları tur atar", () => {
	saveSettings({
		...DEFAULT_SETTINGS,
		discordRpc: true,
		discordRpcScope: "editor",
		discordRpcThemeName: false
	});
	const loaded = loadSettings();
	expect(loaded.discordRpcScope).toBe("editor");
	expect(loaded.discordRpcThemeName).toBe(false);
});
