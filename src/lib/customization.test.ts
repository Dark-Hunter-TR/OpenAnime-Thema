/**
 * `bun test` ile çalışır.
 *
 * Buradaki testler tek bir hata sınıfını hedefliyor: içe aktarılan bir temanın
 * renk değerinin okunamaması. Okunamayan değer sessiz kalmıyor — kontrol o
 * yuvada temanın rengini değil SİTE VARSAYILANINI gösteriyor ve kullanıcı
 * bölümü açtığı anda o varsayılan temanın üstüne yazılıyor.
 *
 * Örnekler uydurma değil: `hsl(0deg 0% 100%)` biçimi örnek temanın metin
 * renklerinin tamamında, `!important` eki ise bildirimlerinin çoğunda geçiyor.
 */

import { expect, test } from "bun:test";

import { DEFAULT_ACCENT_RAMP, fromCssColor, resolveTokenDefault } from "$lib/customization";

// --- Boş rampayla tohumlama ---------------------------------------------------
//
// Kontroller uygulama açılırken, ilk `applyTheme` daha DÖNMEDEN bir kez
// tohumlanıyor; o anda rampa boş. Eskiden bu durumda vurgudan türeyen renkler
// çözülemiyor ve beyaza düşüyordu. Belirtisi: bir bölüm kapalıyken site kendi
// rengini çiziyor, açılınca yanlış tohum yazılıyor ve renk zıplıyordu.

test("boş rampada vurgu token'ı varsayılan rampadan çözülür", () => {
	const withRamp = resolveTokenDefault("hsl(var(--fds-accent-light-1))", DEFAULT_ACCENT_RAMP);
	const withoutRamp = resolveTokenDefault("hsl(var(--fds-accent-light-1))", []);

	expect(withoutRamp).not.toBeNull();
	// Beyaza düşmemeli — eski hatanın imzası buydu.
	expect(withoutRamp?.hex).not.toBe("#ffffff");
	// Ve rampa verilmiş hâliyle aynı sonucu vermeli.
	expect(withoutRamp).toEqual(withRamp);
});

/**
 * Varsayılan rampanın hex karşılıkları.
 *
 * Rust tarafı aynı rampayı `default_base_produces_library_ramp` ile HSL olarak
 * sabitliyor; burada hex karşılıkları sabitleniyor ki iki taraf sessizce
 * ayrışmasın. `#0092fa` özellikle önemli: bir zamanlar bu basamağın yedeği
 * `#00a2ff` yazılmıştı ve gözle ayırt edilen farklı bir maviydi.
 */
test("varsayılan rampanın basamakları sabit", () => {
	const hex = (i: number) => resolveTokenDefault(`hsl(var(--fds-${["accent-light-3", "accent-light-2", "accent-light-1", "accent-base", "accent-dark-1", "accent-dark-2", "accent-dark-3"][i]}))`, [])?.hex;

	expect(hex(2)).toBe("#0092fa"); // accent-light-1
	expect(hex(3)).toBe("#0079d6"); // accent-base
	expect(DEFAULT_ACCENT_RAMP).toHaveLength(7);
});

test("virgüllü klasik yazımlar okunur", () => {
	expect(fromCssColor("#ffffff")).toEqual({ hex: "#ffffff", alpha: 100 });
	expect(fromCssColor("#fff")).toEqual({ hex: "#ffffff", alpha: 100 });
	expect(fromCssColor("rgb(255, 0, 0)")).toEqual({ hex: "#ff0000", alpha: 100 });
	expect(fromCssColor("rgba(255, 0, 0, 0.5)")).toEqual({ hex: "#ff0000", alpha: 50 });
	expect(fromCssColor("hsl(0, 0%, 100%)")).toEqual({ hex: "#ffffff", alpha: 100 });
});

/**
 * CSS Color 4'ün boşluklu yazımı. Eski desenler yalnızca virgüle bakıyordu ve
 * bu değerlerde `null` dönüyordu.
 */
test("boşluklu CSS Color 4 yazımı okunur", () => {
	expect(fromCssColor("hsl(0deg 0% 100%)")).toEqual({ hex: "#ffffff", alpha: 100 });
	expect(fromCssColor("hsl(0 0% 100%)")).toEqual({ hex: "#ffffff", alpha: 100 });
	expect(fromCssColor("rgb(255 0 0)")).toEqual({ hex: "#ff0000", alpha: 100 });
});

test("eğik çizgili alfa okunur", () => {
	expect(fromCssColor("rgb(255 0 0 / 50%)")).toEqual({ hex: "#ff0000", alpha: 50 });
	expect(fromCssColor("hsl(0deg 0% 100% / 20%)")).toEqual({ hex: "#ffffff", alpha: 20 });
});

/** Örnek temanın metin rengi — birebir bu biçimde yazılmış. */
test("gerçek temadaki ikincil metin rengi okunur", () => {
	const parsed = fromCssColor("hsl(0deg 0% 68.65% / 78.6%)");
	expect(parsed).not.toBeNull();
	expect(parsed?.alpha).toBe(79);
});

test("4 ve 8 basamaklı hex alfa taşır", () => {
	expect(fromCssColor("#ff000080")).toEqual({ hex: "#ff0000", alpha: 50 });
	expect(fromCssColor("#f00f")).toEqual({ hex: "#ff0000", alpha: 100 });
});

/**
 * Bir dosyadan gelen bildirimde `!important` ve kapanış noktalı virgülü sık.
 * Temizlenmezse hiçbir desen tutmuyor ve renk "okunamadı" sayılıyordu.
 */
test("!important ve noktalı virgül değeri bozmaz", () => {
	expect(fromCssColor("#ff0000 !important")).toEqual({ hex: "#ff0000", alpha: 100 });
	expect(fromCssColor("  hsl(0deg 0% 100%) !important ; ")).toEqual({
		hex: "#ffffff",
		alpha: 100
	});
});

/** Çözülemeyen değerler `null` dönmeye devam etmeli — sessizce yanlış bir
 * renge düşmek, kontrolün temayı bozmasına yol açardı. */
test("değişken başvurusu ve tanınmayan değer null döner", () => {
	expect(fromCssColor("var(--accent-primary)")).toBeNull();
	expect(fromCssColor("")).toBeNull();
	expect(fromCssColor("linear-gradient(90deg, #f00, #00f)")).toBeNull();
});
