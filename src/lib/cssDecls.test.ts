/**
 * `bun test` ile çalışır.
 *
 * Buradaki testler tek bir hata sınıfını hedefliyor: harici bir `.css`
 * dosyasından gelen bildirimlerin, kontroller yeniden üretim yaptığında sessizce
 * kaybolması. Örnekler uydurma değil — üçü de gerçek OpenAnime temalarından
 * (ytanime, midnight, REzero) alınmış yapılar.
 */

import { expect, test } from "bun:test";

import {
	extractUrl,
	formatDeclarations,
	mergeDeclarations,
	mergeRuleOverrides,
	parseDeclarations,
	resolveLength,
	splitDeclarations
} from "$lib/cssDecls";
import { controlledRuleProps } from "$lib/advancedBuild";
import { PLAYER_SLIDER_SELECTOR, CARD_SELECTOR, BG_BODY_SELECTOR } from "$lib/advanced";

// --- Bölme -----------------------------------------------------------------

test("data URI içindeki `;` bildirim sınırı sayılmaz", () => {
	// ytanime teması `--fds-acrylic-noise-asset`i tam olarak böyle veriyor.
	const body = "--noise: url(data:image/png;base64,iVBORw0KGgo=); color: red;";
	const decls = parseDeclarations(body);

	expect(decls).toHaveLength(2);
	expect(decls[0]).toEqual({
		property: "--noise",
		value: "url(data:image/png;base64,iVBORw0KGgo=)"
	});
	expect(decls[1]).toEqual({ property: "color", value: "red" });
});

test("tırnak ve yorum içindeki `;` sınır sayılmaz", () => {
	const body = `content: "a;b"; /* not; içinde */ color: blue;`;
	const decls = parseDeclarations(body);

	expect(decls.map((d) => d.property)).toEqual(["content", "color"]);
	expect(decls[0].value).toBe('"a;b"');
});

test("satır sonu açıklaması değerin parçası olmaz", () => {
	// REzero teması her bildirimin sonuna açıklama yorumu koyuyor.
	const body = "--boyut: 55px; /* [Açıklama] Logo boyutu. */";
	expect(parseDeclarations(body)).toEqual([{ property: "--boyut", value: "55px" }]);
});

test("değerdeki `:` özellik sınırı sanılmaz", () => {
	const body = "background-image: url(https://example.com/a.png) !important;";
	expect(parseDeclarations(body)).toEqual([
		{ property: "background-image", value: "url(https://example.com/a.png) !important" }
	]);
});

test("boş ve bozuk parçalar atılır", () => {
	expect(parseDeclarations(";; color:; :red; ;")).toEqual([]);
	expect(splitDeclarations("")).toEqual([]);
});

// --- Birleştirme -----------------------------------------------------------

test("kontrolün sahip olmadığı bildirimler korunur", () => {
	// ytanime'ın `.slider.orientation-horizontal` kuralı: beş bildirim, biri
	// (`block-size`) kontrollere bağlı, dördü değil.
	const fromFile =
		"block-size: 4px; inline-size: 100%; justify-content: flex-start; " +
		"position: relative; color: red;";
	const fromControls = "block-size: 8px !important;";

	const merged = mergeDeclarations(fromFile, fromControls, new Set(["block-size"]));
	const decls = parseDeclarations(merged);

	expect(decls.find((d) => d.property === "color")?.value).toBe("red");
	expect(decls.find((d) => d.property === "position")?.value).toBe("relative");
	expect(decls.find((d) => d.property === "justify-content")?.value).toBe("flex-start");
	// Kontrolün sahip olduğu özellik dosyadakini eziyor ve BİR KEZ yazılıyor.
	expect(decls.filter((d) => d.property === "block-size")).toEqual([
		{ property: "block-size", value: "8px !important" }
	]);
});

test("bölüm kapatılınca yalnızca kontrolün özellikleri düşer", () => {
	const fromFile = "block-size: 4px; color: red;";
	const merged = mergeDeclarations(fromFile, "", new Set(["block-size"]));

	expect(parseDeclarations(merged)).toEqual([{ property: "color", value: "red" }]);
});

test("yalnızca kontrol bildirimleri kalmışsa gövde boşalır", () => {
	const merged = mergeDeclarations("block-size: 4px;", "", new Set(["block-size"]));
	expect(merged.trim()).toBe("");
});

test("formatDeclarations tur atar", () => {
	const body = "color: red; --x: url(data:image/png;base64,AA==);";
	expect(parseDeclarations(formatDeclarations(parseDeclarations(body)))).toEqual(
		parseDeclarations(body)
	);
});

// --- Harita düzeyi ---------------------------------------------------------

const OWNED = { [PLAYER_SLIDER_SELECTOR]: new Set(["block-size"]) };

test("harita birleştirmesi tanınmayan seçicilere dokunmaz", () => {
	const existing = { ".ozel-rozet": "background: red;" };
	const next = mergeRuleOverrides(existing, {}, [PLAYER_SLIDER_SELECTOR], OWNED);

	expect(next[".ozel-rozet"]).toBe("background: red;");
});

test("gövdesi tamamen boşalan seçici haritadan silinir", () => {
	const existing = { [PLAYER_SLIDER_SELECTOR]: "block-size: 4px;" };
	const next = mergeRuleOverrides(existing, {}, [PLAYER_SLIDER_SELECTOR], OWNED);

	expect(PLAYER_SLIDER_SELECTOR in next).toBe(false);
});

test("kontrolde karşılığı olmayan seçicinin gövdesi olduğu gibi kalır", () => {
	// `body`, KNOWN_SELECTORS'ta ama `buildAdvRules` ona doğrudan yazmıyor
	// (arkaplan `body::before`e yazılıyor). Kontrollerin sahip olduğu bir
	// özellik olmadığı için dosyadan gelen gövde bütünüyle korunmalı.
	const existing = { [BG_BODY_SELECTOR]: "background-color: black; background-size: cover;" };
	const next = mergeRuleOverrides(existing, {}, [BG_BODY_SELECTOR], controlledRuleProps());

	expect(parseDeclarations(next[BG_BODY_SELECTOR]).map((d) => d.property)).toEqual([
		"background-color",
		"background-size"
	]);
});

// --- Sahiplik listesinin türetilmesi ----------------------------------------

test("sahiplik listesi buildAdvRules'tan türetiliyor ve dolu", () => {
	const owned = controlledRuleProps();

	// Liste elle tutulsaydı bu iki beklenti sessizce eskirdi; türetildiği için
	// `buildAdvRules` değiştiğinde kendiliğinden güncelleniyor.
	expect(owned[CARD_SELECTOR]?.has("border-radius")).toBe(true);
	expect(Object.keys(owned).length).toBeGreaterThan(5);

	// Hiçbir kümede boş özellik adı olmamalı — olsaydı `mergeDeclarations`
	// yanlış bildirimleri silerdi.
	for (const props of Object.values(owned)) {
		for (const p of props) expect(p.trim()).not.toBe("");
	}
});

// --- resolveLength -----------------------------------------------------------
//
// İçe aktarılan temaların sayısal ayarları (logo büyüklüğü, kart yuvarlaklığı,
// kenar çubuğu genişliği) kontrollere hiç ulaşmıyordu. Aşağıdaki üç biçim de
// gerçek temalardan: Midnight logo boyutunu değişkenden veriyor, REzero
// yuvarlaklığı değişken + `!important` ile yazıyor, site kenar çubuğunu `rem`
// ile ölçüyor.

test("düz px değeri okunur", () => {
	expect(resolveLength("width: 20px;", "width", {})).toBe(20);
	expect(resolveLength("border-radius: 12px;", "border-radius", {})).toBe(12);
});

test("!important değeri bozmaz", () => {
	expect(resolveLength("width: 20px !important;", "width", {})).toBe(20);
});

test("var() zinciri çözülür", () => {
	const tokens = {
		"--logo-size-desktop": "20px",
		"--ayar-yuvarlaklik": "var(--taban)",
		"--taban": "14px"
	};
	expect(resolveLength("width: var(--logo-size-desktop) !important;", "width", tokens)).toBe(20);
	expect(resolveLength("border-radius: var(--ayar-yuvarlaklik);", "border-radius", tokens)).toBe(14);
});

test("var() yedeği kullanılır", () => {
	expect(resolveLength("gap: var(--yok, 8px);", "gap", {})).toBe(8);
});

test("rem birimi okunur", () => {
	expect(resolveLength("width: 4.5rem;", "width", {}, "rem")).toBe(4.5);
	// Birim uyuşmuyorsa değer okunmamalı — yanlış birimde bir sayıyı
	// kontrole yazmak sessizce yanlış bir düzen üretirdi.
	expect(resolveLength("width: 4.5rem;", "width", {})).toBeNull();
});

test("özellik sınırı korunur", () => {
	// `width` ararken `min-width` eşleşmemeli.
	expect(resolveLength("min-width: 99px; width: 20px;", "width", {})).toBe(20);
	expect(resolveLength("min-width: 99px;", "width", {})).toBeNull();
	// Tersi de: `-webkit-border-radius` `border-radius` sanılmamalı.
	expect(resolveLength("-webkit-border-radius: 99px;", "border-radius", {})).toBeNull();
});

// --- extractUrl --------------------------------------------------------------
//
// Temalar logoyu gömülü bir SVG olarak veriyor. O değerin içinde boşluk da var,
// `"` de, `/` de — karakter sınıfına dayanan eski desen ilk boşlukta kopuyor ve
// hiç eşleşmiyordu. Sonucu, içe aktarılan temanın logo görselinin hiç
// gelmemesiydi.

test("tırnaksız url okunur", () => {
	expect(extractUrl("url(https://example.com/a.png)")).toBe("https://example.com/a.png");
	expect(extractUrl("background: url( a.png ) no-repeat;")).toBe("a.png");
});

test("tırnaklı url okunur", () => {
	expect(extractUrl(`url("https://example.com/a.png")`)).toBe("https://example.com/a.png");
	expect(extractUrl("url('a.png')")).toBe("a.png");
});

/** Gerçek temadaki `--url-logo` biçimi: boşluklu, tırnaklı, eğik çizgili. */
test("gömülü SVG veri URI'si bütün olarak okunur", () => {
	const svg =
		`data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" ` +
		`viewBox="0 0 24 24" fill="none" stroke="%238db4f7"><path d="M21 12z"/></svg>`;
	expect(extractUrl(`url('${svg}')`)).toBe(svg);
	// Kısaltma değerin içinde de bulunmalı.
	expect(extractUrl(`url('${svg}') no-repeat center / contain !important`)).toBe(svg);
});

test("url yoksa boş döner", () => {
	expect(extractUrl("background: #fff;")).toBe("");
	expect(extractUrl("")).toBe("");
});

test("çözülemeyen değer null döner", () => {
	expect(resolveLength("width: var(--tanimsiz);", "width", {})).toBeNull();
	expect(resolveLength("width: auto;", "width", {})).toBeNull();
	expect(resolveLength(undefined, "width", {})).toBeNull();
});
