/**
 * `bun test` ile çalışır.
 *
 * Burada sınanan üç şey, elle doğrulaması en pahalı olanlar:
 *   1. Logo/site adı bağımsızlığı (daha önce logonun kaybolmasına yol açan hata)
 *   2. Varsayılanların sitenin gerçek değerleriyle eşleşmesi
 *   3. Sıfırlamanın yalnızca hedef bölümü etkilemesi ve CSS'ten tam düşmesi
 */

import { expect, test } from "bun:test";

import {
	BADGE_SELECTOR,
	BADGE_TEXT_HIDE_SELECTOR,
	BADGE_TEXT_SELECTOR,
	LOGO_BADGE_SELECTOR,
	LOGO_ICON_GUARD_SELECTOR,
	LOGO_IMAGE_HIDE_SELECTOR,
	LOGO_IMAGE_SELECTOR,
	LOGO_NESTED_SELECTOR,
	LOGO_ROW_SELECTOR,
	LOGO_TEXT_HIDE_SELECTOR,
	LOGO_TEXT_SELECTOR,
	SIDEBAR_SELECTOR
} from "$lib/advanced";
import {
	BADGE_INDEPENDENT_FIELDS,
	buildAdvRules,
	buildAdvTokens,
	controlledRuleProps,
	defaultAdv,
	reseedSection,
	resetAdvSection
} from "$lib/advancedBuild";

const PNG = "data:image/png;base64,iVBORw0KGgo=";

// --- Madde 6: logo ve site adı bağımsızlığı ---------------------------------

test("yalnızca site adı değişince logo görseli gizlenmez", () => {
	const adv = defaultAdv();
	adv.logo.textOn = true;
	adv.logo.text = "Benim Sitem";

	const rules = buildAdvRules(adv);

	// Hatanın kendisi buydu: img gizleniyor ama yerine bir şey konmuyordu.
	expect(rules[LOGO_IMAGE_HIDE_SELECTOR]).toBeUndefined();
	expect(rules[LOGO_IMAGE_SELECTOR]).toBeUndefined();
	// Ad yine de basılmalı.
	expect(rules[LOGO_TEXT_HIDE_SELECTOR]).toBe("display: none !important;");
	expect(rules[LOGO_TEXT_SELECTOR]).toContain('content: "Benim Sitem"');
});

test("yalnızca site adı değişince orijinal ikon shrink ile kaybolmasın diye korunur", () => {
	const adv = defaultAdv();
	adv.logo.textOn = true;
	adv.logo.text = "test";

	const rules = buildAdvRules(adv);

	// İkon gizlenmiyor (önceki test) ama flex satırda tek esnek öğe kalıp
	// 0 genişliğe küçülmesin diye ayrı bir korumaya sahip olmalı.
	expect(rules[LOGO_ICON_GUARD_SELECTOR]).toContain("flex: 0 0 auto");
	expect(rules[LOGO_ICON_GUARD_SELECTOR]).toContain("order: 0");
});

test("logo, ad ve rozet DOM sırasından bağımsız olarak ikon -> ad -> rozet sırasına zorlanır", () => {
	const adv = defaultAdv();
	adv.logo.imageOn = true;
	adv.logo.dataUri = PNG;
	adv.logo.textOn = true;
	adv.logo.text = "test";

	const rules = buildAdvRules(adv);

	// `::after` (ad) CSS gereği her zaman gerçek çocuklardan (rozet dâhil)
	// sonra basılır — `order` olmadan ad her zaman rozetin sağına düşerdi.
	expect(rules[LOGO_IMAGE_SELECTOR]).toContain("order: 0");
	expect(rules[LOGO_TEXT_SELECTOR]).toContain("order: 1");
	expect(rules[LOGO_BADGE_SELECTOR]).toContain("order: 2");
});

test("yalnızca logo görseli değişince site adı gizlenmez", () => {
	const adv = defaultAdv();
	adv.logo.imageOn = true;
	adv.logo.dataUri = PNG;

	const rules = buildAdvRules(adv);

	expect(rules[LOGO_TEXT_HIDE_SELECTOR]).toBeUndefined();
	expect(rules[LOGO_TEXT_SELECTOR]).toBeUndefined();
	expect(rules[LOGO_IMAGE_HIDE_SELECTOR]).toBe("display: none !important;");
	expect(rules[LOGO_IMAGE_SELECTOR]).toContain(PNG);
});

test("NEXT-GEN rozeti hiçbir durumda gizlenmez", () => {
	const adv = defaultAdv();
	adv.logo.imageOn = true;
	adv.logo.dataUri = PNG;
	adv.logo.textOn = true;
	adv.logo.text = "Çok Çok Çok Uzun Bir Site Adı";

	for (const [selector, body] of Object.entries(buildAdvRules(adv))) {
		if (body.includes("display: none")) {
			expect(selector).not.toContain("#badge");
		}
	}
	expect(buildAdvRules(adv)[LOGO_BADGE_SELECTOR]).toContain("flex: 0 0 auto");
});

test("uzun site adı rozete taşmak yerine üç noktayla kısalır", () => {
	const adv = defaultAdv();
	adv.logo.textOn = true;
	adv.logo.text = "Çok Çok Çok Uzun Bir Site Adı";

	const rules = buildAdvRules(adv);
	const text = rules[LOGO_TEXT_SELECTOR];

	expect(text).toContain("text-overflow: ellipsis");
	expect(text).toContain("overflow: hidden");
	expect(text).toContain("white-space: nowrap");
	// `min-width: 0` olmadan ellipsis flex içinde ETKİSİZDİR — asıl taşma sebebi.
	expect(text).toContain("min-width: 0");
	expect(text).toContain("max-width:");
	expect(rules[LOGO_ROW_SELECTOR]).toContain("min-width: 0");
});

test("iç içe a.logo-button > a.logo çift basmayı engeller", () => {
	const adv = defaultAdv();
	adv.logo.textOn = true;
	adv.logo.text = "X";
	expect(buildAdvRules(adv)[LOGO_NESTED_SELECTOR]).toContain("content: none");
});

test("boş ad ya da seçilmemiş görsel hiçbir kural üretmez", () => {
	const adv = defaultAdv();
	adv.logo.textOn = true;
	adv.logo.text = "   ";
	adv.logo.imageOn = true;
	adv.logo.dataUri = "";

	const rules = buildAdvRules(adv);
	expect(rules[LOGO_TEXT_SELECTOR]).toBeUndefined();
	expect(rules[LOGO_IMAGE_SELECTOR]).toBeUndefined();
	expect(rules[LOGO_IMAGE_HIDE_SELECTOR]).toBeUndefined();
	expect(rules[LOGO_TEXT_HIDE_SELECTOR]).toBeUndefined();
});

// --- Madde 4: varsayılanlar sitenin kendi değerleri --------------------------

test("hiçbir bölüm açık değilken tek satır CSS bile üretilmez", () => {
	const adv = defaultAdv();
	expect(Object.keys(buildAdvTokens(adv))).toEqual([]);
	expect(Object.keys(buildAdvRules(adv))).toEqual([]);
});

test("sayısal varsayılanlar sitenin canlı CSS'iyle eşleşir", () => {
	const adv = defaultAdv();
	expect(adv.cards.radius).toBe(8); // border-radius: var(--fds-overlay-corner-radius)
	expect(adv.cards.lift).toBe(2); // :hover { transform: translateY(-2px) }
	expect(adv.cards.borderWidth).toBe(0); // sitede kart kenarlığı yok
	expect(adv.player.progressHeight).toBe(4); // .slider-rail { block-size: 4px }
	expect(adv.scrollbar.size).toBe(10); // .os-theme-* { --os-size: 10px }
	expect(adv.scrollbar.trackRadius).toBe(50); // --os-track-border-radius: 50px
	expect(adv.avatar.size).toBe(32); // --fds-person-picture-size: 32px
	expect(adv.sidebar.width).toBe(72); // .sidebar { min-width: 4.5rem }
	expect(adv.sidebar.indicatorWidth).toBe(4); // .list-item::before { inline-size: .25rem (4px) }
	expect(adv.sidebar.indicatorHeight).toBe(24); // block-size: 1.5rem (24px)
	expect(adv.banner.progressHeight).toBe(5); // #progress { height: .3rem }
	expect(adv.banner.progressColor).toBe("#ffffff"); // #progress { background: #fff }
	expect(adv.typo.scale).toBe(1); // ölçek yok = sitenin boyutları
});

test("renk varsayılanları katalogdan çözülür, beyaz yer tutucu değil", () => {
	const adv = defaultAdv("dark", []);
	// --fds-text-primary koyu modda hsla(0,0%,100%,100%) -> beyaz, DOĞRU beyaz.
	expect(adv.text.colors[0].hex).toBe("#ffffff");
	// --fds-text-secondary koyu modda %78.6 saydam beyaz.
	expect(adv.text.colors[1].alpha).toBeGreaterThan(70);
	expect(adv.text.colors[1].alpha).toBeLessThan(85);

	const light = defaultAdv("light", []);
	// Açık modda ana metin siyaha yakın olmalı — mod gerçekten okunuyor mu?
	expect(light.text.colors[0].hex).toBe("#000000");
});

test("açık ve koyu mod farklı yüzey varsayılanları verir", () => {
	const dark = defaultAdv("dark", []);
	const light = defaultAdv("light", []);
	// --fds-solid-background-base: koyu hsl(0,0%,13%), açık hsl(0,0%,95%)
	expect(dark.surface.colors[0].hex).not.toBe(light.surface.colors[0].hex);
});

test("accent'ten türeyen varsayılanlar rampayı takip eder", () => {
	const withRamp = defaultAdv("dark", [
		"191, 98%, 80%",
		"199, 99%, 69%",
		"205, 100%, 49%",
		"206, 100%, 42%",
		"209, 100%, 36%",
		"215, 100%, 29%",
		"226, 100%, 20%"
	]);
	// #badge sitede accent-light-1'den başlıyor: 205,100%,49% -> #00a2fa civarı
	expect(withRamp.badges.badgeFrom).toMatch(/^#[0-9a-f]{6}$/);
	expect(withRamp.badges.badgeFrom).not.toBe("#ffffff");
});

// --- Madde 1: sıfırlama ------------------------------------------------------

test("bölüm sıfırlama yalnızca o bölümü etkiler", () => {
	let adv = defaultAdv();
	adv.cards.on = true;
	adv.cards.radius = 33;
	adv.sidebar.on = true;
	adv.sidebar.width = 199;

	adv = resetAdvSection(adv, "cards", defaultAdv());

	expect(adv.cards.on).toBe(false);
	expect(adv.cards.radius).toBe(8);
	// Komşu bölüm dokunulmamış kalmalı.
	expect(adv.sidebar.on).toBe(true);
	expect(adv.sidebar.width).toBe(199);
});

test("sıfırlanan bölüm üretilen CSS'ten tamamen düşer", () => {
	let adv = defaultAdv();
	adv.sidebar.on = true;
	adv.sidebar.width = 199;
	expect(buildAdvRules(adv)[SIDEBAR_SELECTOR]).toContain("199px");

	adv = resetAdvSection(adv, "sidebar", defaultAdv());
	expect(buildAdvRules(adv)[SIDEBAR_SELECTOR]).toBeUndefined();
});

test("logo sıfırlama seçilmiş görseli ve adı da temizler", () => {
	let adv = defaultAdv();
	adv.logo.imageOn = true;
	adv.logo.dataUri = PNG;
	adv.logo.textOn = true;
	adv.logo.text = "X";

	adv = resetAdvSection(adv, "logo", defaultAdv());

	expect(adv.logo.dataUri).toBe("");
	expect(adv.logo.text).toBe("");
	expect(Object.keys(buildAdvRules(adv))).toEqual([]);
});

/**
 * Sıfırlama, düzenlenen şeyin ORİJİNALİNE döner.
 *
 * Kullanıcının bildirdiği hata: içe aktarılmış bir temada bir bölümü yalnızca
 * kapatıp açmak, temanın değerlerini SİTE varsayılanlarıyla değiştiriyordu —
 * logonun yazı tipi ve rengi gidiyordu. Kural iki durumlu: yeni tema
 * oluşturuluyorsa taban sitenin değerleri, bir dosya/GitHub içeriği
 * düzenleniyorsa o temanın değerleri.
 */
test("sıfırlama, içe aktarılan temanın değerlerine döner", () => {
	// İçe aktarılmış temanın kontrol durumu.
	const imported = defaultAdv();
	imported.logo.textOn = true;
	imported.logo.text = "Midnight";
	imported.logo.textSize = 15;
	imported.cards.radius = 10;

	// Kullanıcı üzerinde oynuyor.
	let adv = structuredClone(imported);
	adv.logo.text = "Başka";
	adv.logo.textSize = 28;

	adv = resetAdvSection(adv, "logo", imported);

	expect(adv.logo.text).toBe("Midnight");
	expect(adv.logo.textSize).toBe(15);
	expect(adv.logo.textOn).toBe(true);
});

/** Taban site varsayılanıyken davranış eskisi gibi kalmalı. */
test("taban site varsayılanıyken sıfırlama siteye döner", () => {
	let adv = defaultAdv();
	adv.logo.textOn = true;
	adv.logo.text = "X";

	adv = resetAdvSection(adv, "logo", defaultAdv());

	expect(adv.logo.textOn).toBe(false);
	expect(adv.logo.text).toBe("");
	expect(Object.keys(buildAdvRules(adv))).toEqual([]);
});

// --- Madde 3/4: maskot boyutları --------------------------------------------

test("akışkan maskotlara sabit boyut verilmez", () => {
	const adv = defaultAdv();
	adv.mascot.sizeOn = true;
	const rules = buildAdvRules(adv);

	// #setsuki ve .setsuki #image sitede height:100% / width:auto.
	expect(rules["#setsuki"]).toBeUndefined();
	expect(rules[".setsuki #image"]).toBeUndefined();
	// Sabit ölçülü olanlar sitedeki gerçek değerleriyle çıkar.
	expect(rules["#notification-setsuki"]).toContain("170px");
	expect(rules["#download-setsuki"]).toContain("170px");
	expect(rules["#mobile-notification-setsuki"]).toContain("150px");
});

test("maskot görseli boyut kapalıyken de uygulanır", () => {
	const adv = defaultAdv();
	adv.mascot.images.generic = PNG;
	const rules = buildAdvRules(adv);
	expect(rules[".setsuki #image"]).toContain(PNG);
	expect(rules[".setsuki #image"]).not.toContain("width:");
});

// --- Madde 3: yeni bölümler gerçekten CSS üretiyor mu -----------------------

test("kaydırma çubuğu sitenin --os-* API'sini yazar, webkit sözde elemanını değil", () => {
	const adv = defaultAdv();
	adv.scrollbar.on = true;
	const rules = buildAdvRules(adv);
	const body = rules[".os-scrollbar"];

	expect(body).toContain("--os-size");
	expect(body).toContain("--os-handle-bg");
	// Webkit scrollbar kuralının yazılmadığından emin ol.
	expect(JSON.stringify(rules)).not.toContain("::-webkit-scrollbar");
});

test("bağlantılar vurgu rampasından değil accent-text setinden boyanır", () => {
	const adv = defaultAdv();
	adv.links.on = true;
	const tokens = buildAdvTokens(adv);
	expect(tokens["--fds-accent-text-primary"]).toBeDefined();
	// Vurgu rampasının kendisine dokunulmamalı.
	expect(tokens["--fds-accent-base"]).toBeUndefined();
});

test("yazı boyutu ölçeği 1.00x iken hiçbir boyut token'ı yazılmaz", () => {
	const adv = defaultAdv();
	adv.typo.on = true;
	adv.typo.sizeOn = true;
	adv.typo.scale = 1;
	expect(buildAdvTokens(adv)["--fds-body-font-size"]).toBeUndefined();

	adv.typo.scale = 1.25;
	// --fds-body-font-size sitede 14px -> 17.5px
	expect(buildAdvTokens(adv)["--fds-body-font-size"]).toBe("17.5px");
});

test("ölü .calendar-card selector'ı üretilen CSS'te yok", () => {
	const adv = defaultAdv();
	adv.cards.on = true;
	expect(JSON.stringify(buildAdvRules(adv))).not.toContain("calendar-card");
});

// --- Rozetler: gizleme ve yazı -----------------------------------------------

/**
 * Gizleme ve yazı, renk anahtarından BAĞIMSIZ olmalı: rozeti kaldırmak ya da
 * yazısını değiştirmek isteyen biri renklerini de özelleştirmek zorunda
 * kalmamalı.
 */
test("rozet gizleme renk anahtarı kapalıyken de çalışır", () => {
	const adv = defaultAdv();
	expect(buildAdvRules(adv)[BADGE_SELECTOR]).toBeUndefined();

	adv.badges.badgeHidden = true;
	expect(adv.badges.on).toBe(false);
	expect(buildAdvRules(adv)[BADGE_SELECTOR]).toContain("display: none");
});

test("rozet yazısı orijinali gizleyip yerine yazar", () => {
	const adv = defaultAdv();
	adv.badges.badgeText = "  Yeni  ";

	const rules = buildAdvRules(adv);
	expect(rules[BADGE_TEXT_HIDE_SELECTOR]).toContain("display: none");
	// Baştaki/sondaki boşluk kırpılır, değer tırnaklanır.
	expect(rules[BADGE_TEXT_SELECTOR]).toContain('content: "Yeni"');
});

test("yazıdaki tırnak ve ters bölü kaçırılır", () => {
	const adv = defaultAdv();
	adv.badges.badgeText = 'A"B\\C';
	// Kaçırılmasaydı `content` bildirimi ortadan bölünür ve kural bozulurdu.
	expect(buildAdvRules(adv)[BADGE_TEXT_SELECTOR]).toContain('content: "A\\"B\\\\C"');
});

/**
 * Gizleme, rengi ve yazıyı bastırmamalı.
 *
 * Bastırsaydı iki dal farklı özellik kümeleri üretirdi ve
 * `controlledRuleProps` hangi dal açıksa yalnızca onunkini görürdü: kullanıcı
 * gizlemeyi kapattığında eski `display: none` gövdede takılı kalırdı.
 */
/**
 * Rozet yazısı SİLİNDİĞİNDE kural da kalkmalı.
 *
 * Kullanıcının bildirdiği hata: rozete yazı yazıp sonra silince düzelmiyordu —
 * orijinal yazı gizli kalmaya devam ediyor, yerine de bir şey yazılmıyordu.
 *
 * Sebep `controlledRuleProps`ta: o, `enableEverySection` ile bütün bölümleri
 * açıp üretilen kuralların ÖZELLİK ADLARINI topluyor ve `mergeRuleOverrides`
 * neyi kaldırabileceğini oradan öğreniyor. `badgeText` gibi alanlar boş
 * kaldığı sürece o kurallar hiç üretilmiyor, dolayısıyla listeye girmiyor ve
 * eski kural kaldırılamıyordu.
 */
/**
 * Enjekte edilen rozet yazısı, sitenin KENDİ ölçüsüyle yazılmalı.
 *
 * Değerler openani.me'nin canlı CSS'inden okundu:
 *   `#badge .text-block { text-transform: uppercase; font-size: 10px;
 *    font-weight: 600; letter-spacing: .5px }`
 *
 * Üç rozetin yazı stili birbirinden farklı. Tek bir ortak stil kullanıldığında
 * rozet yazısı orijinalinden belirgin biçimde büyük ve ince çıkıyordu.
 */
test("rozet yazısı sitenin ölçüsüyle basılır", () => {
	const adv = defaultAdv();
	adv.badges.badgeText = "Yeni";

	const body = buildAdvRules(adv)[BADGE_TEXT_SELECTOR];
	expect(body).toContain("font-size: 10px");
	expect(body).toContain("font-weight: 600");
	expect(body).toContain("text-transform: uppercase");
});

/**
 * "Yayınlandı" rozetinin ikinci durağı SAYDAM.
 *
 * Site: `linear-gradient(to right, #6371da, var(--fds-accent-tertiary))` ve
 * `--fds-accent-tertiary` = `hsla(var(--fds-accent-base), 80%)`. Opak
 * yazıldığında rozetin varsayılanı sitedekinden koyu çıkıyordu.
 */
test("yayınlandı rozetinin bitiş rengi %80 alfa taşır", () => {
	const adv = defaultAdv();
	// Sekiz basamaklı hex; `cc` = %80.
	expect(adv.badges.releasedTo).toMatch(/^#[0-9a-f]{6}cc$/i);
});

test("rozet yazısı seçicileri kontrollerin sahiplendiği kümede", () => {
	const owned = controlledRuleProps();

	expect(owned[BADGE_TEXT_SELECTOR]?.has("content")).toBe(true);
	expect(owned[BADGE_TEXT_HIDE_SELECTOR]?.has("display")).toBe(true);
});

/**
 * Kapalı bölümün tazelenmesi, `on`a BAĞLI OLMAYAN alanları ezmemeli.
 *
 * Kullanıcının bildirdiği hata: "Rozeti gizle" anahtarı açılmıyordu. Anahtar
 * `adv.badges` nesnesinin içinde ve bölüm kapalıyken o nesne tabandan yeniden
 * kuruluyor — kullanıcı anahtarı açtığı anda tepkisel ifade çalışıp geri
 * kapatıyordu. Yazı kutuları da aynı şekilde temizleniyordu.
 *
 * Bu sınıf kontrol denetiminin (`scripts/audit-controls.mjs`) kapsamı dışında:
 * o, üretilen CSS'i ölçüyor, Svelte'in tepkiselliğini değil.
 */
test("tazeleme rozetlerin gizleme ve yazı alanlarını korur", () => {
	const baseline = defaultAdv();

	const current = defaultAdv();
	current.badges.badgeHidden = true;
	current.badges.badgeText = "Yeni";
	// Renk ise tabandan tazelenmeli.
	current.badges.badgeFrom = "#123456";

	const next = reseedSection(current.badges, baseline.badges, BADGE_INDEPENDENT_FIELDS);

	expect(next.badgeHidden).toBe(true);
	expect(next.badgeText).toBe("Yeni");
	expect(next.badgeFrom).toBe(baseline.badges.badgeFrom);
});

/** Sıfırlama düğmesi bu yoldan geçmiyor: her şeyi tabana döndürmeli. */
test("sıfırlama gizleme ve yazıyı da temizler", () => {
	let adv = defaultAdv();
	adv.badges.badgeHidden = true;
	adv.badges.badgeText = "Yeni";

	adv = resetAdvSection(adv, "badges", defaultAdv());

	expect(adv.badges.badgeHidden).toBe(false);
	expect(adv.badges.badgeText).toBe("");
});

test("gizli rozette renk bildirimi de yazılmaya devam eder", () => {
	const adv = defaultAdv();
	adv.badges.on = true;
	adv.badges.badgeHidden = true;

	const body = buildAdvRules(adv)[BADGE_SELECTOR];
	expect(body).toContain("display: none");
	expect(body).toContain("linear-gradient");
});
