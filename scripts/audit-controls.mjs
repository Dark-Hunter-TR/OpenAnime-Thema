/**
 * Her gelişmiş kontrolü tek tek oynatır ve üretilen CSS'in gerçekten
 * değişip değişmediğini ölçer.
 *
 * Neden var: "ayarı değiştiriyorum, hiçbir şey olmuyor" şikâyetini tahminle
 * değil listeyle cevaplamak için. Betik `AdvState`'in HER alanını dolaşıyor —
 * elle yazılmış bir kontrol listesi yok, dolayısıyla yeni bir bölüm eklendiğinde
 * kendiliğinden kapsanıyor.
 *
 * Ölçtüğü şey ön yüz katmanı: kontrol -> `buildAdvTokens` / `buildAdvRules` /
 * `buildAdvImports`. Rust katmanı (belge -> CSS, temayı yenme) ayrı bir yerde
 * ölçülüyor: `src-tauri/src/theme/fidelity_tests.rs`.
 *
 * Çalıştırma: `bun scripts/audit-controls.mjs`
 * Çıkış kodu: ölü kontrol bulunursa 1.
 */

import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const { buildAdvImports, buildAdvRules, buildAdvTokens, defaultAdv, enableEverySection } =
	await import(resolve(root, "src/lib/advancedBuild.ts"));
const { SURFACE_OWNED_TOKENS } = await import(resolve(root, "src/lib/advancedBuild.ts"));
const { CARD_TOKENS, TEXT_TOKENS } = await import(resolve(root, "src/lib/advanced.ts"));
const { LINK_TOKENS, SURFACE_TOKENS, SYSTEM_TOKENS } = await import(
	resolve(root, "src/lib/customization.ts")
);

/**
 * Renk dizisi taşıyan bölümlerin token tanımları.
 *
 * Saydamlık alanını elemek için gerekiyor: `ColorField` alfa kaydırıcısını
 * yalnızca `spec.alpha` doğruyken çiziyor. Spec'i `false` olan bir yuvanın
 * alfası çıktıyı değiştirmiyor ama bu bir hata DEĞİL — o kontrol arayüzde hiç
 * yok. Elenmezse rapor gerçek bulguları gölgeleyen gürültüyle doluyor.
 */
const COLOR_SPECS = {
	text: TEXT_TOKENS,
	cards: CARD_TOKENS,
	surface: SURFACE_TOKENS,
	links: LINK_TOKENS,
	system: SYSTEM_TOKENS
};

/** `cards.colors[3].alpha` gibi bir yolun arayüzde karşılığı var mı? */
function hasControl(path) {
	const slot = path.match(/^(\w+)\.colors\[(\d+)\]\.(hex|alpha)$/);
	if (!slot) return true;

	const spec = COLOR_SPECS[slot[1]]?.[Number(slot[2])];
	if (!spec) return true;

	// Sahibi başka bir bölüm olan yuva burada ne yazılıyor ne çiziliyor
	// (bkz. `advancedBuild.ts` -> `SURFACE_OWNED_TOKENS`).
	if (slot[1] === "cards" && SURFACE_OWNED_TOKENS.has(spec.token)) return false;

	// Alfa kaydırıcısı yalnızca `spec.alpha` doğruyken çiziliyor
	// (bkz. `ColorField.svelte`).
	if (slot[3] === "alpha") return spec.alpha === true;
	return true;
}

/** Bir durumdan üretilen her şey — tek bir karşılaştırılabilir dize. */
function output(adv) {
	return JSON.stringify([buildAdvTokens(adv), buildAdvRules(adv), buildAdvImports(adv)]);
}

const clone = (value) => JSON.parse(JSON.stringify(value));

/**
 * Bir alana, mevcut değerinden KESİNLİKLE farklı bir değer üretir.
 *
 * Sayıyı büyütmek yetmiyor: kaydırıcıların bir kısmı sınırlı aralıkta ve
 * üst sınırdaki bir değeri artırmak aynı değeri geri verebilir. Bu yüzden
 * sayılarda hem artırma hem azaltma deneniyor.
 */
function mutations(value) {
	if (typeof value === "boolean") return [!value];
	if (typeof value === "number") return [value + 7, value - 7, value + 1, value - 1];
	if (typeof value === "string") {
		// Renk gibi görünen değerler renk kalmalı; aksi hâlde `toCssColor`
		// değeri eleyip çıktıyı hiç değiştirmiyor ve kontrol yanlışlıkla
		// "ölü" görünüyor.
		if (/^#[0-9a-fA-F]{3,8}$/.test(value)) return ["#123456", "#abcdef"];
		if (value.startsWith("url(") || value.startsWith("data:")) return ["data:image/png;base64,iVBORw0KGgo="];
		return [value === "OA_SENTINEL" ? "OA_SENTINEL_2" : "OA_SENTINEL"];
	}
	return [];
}

const dead = [];
const skipped = [];

/** `adv` içindeki her yaprak alanı gez. */
function walk(path, holder, key) {
	const value = holder[key];

	if (value !== null && typeof value === "object" && !Array.isArray(value)) {
		for (const inner of Object.keys(value)) walk(`${path}.${inner}`, value, inner);
		return;
	}

	if (Array.isArray(value)) {
		value.forEach((_, i) => walk(`${path}[${i}]`, value, i));
		return;
	}

	const options = mutations(value);
	if (options.length === 0) {
		skipped.push(`${path} (değiştirilemedi: ${JSON.stringify(value)})`);
		return;
	}

	// Açma/kapama anahtarları (`on`, `sizeOn`, `imageOn` …) burada değil,
	// aşağıdaki bölüm sondasında ölçülüyor. Bir anahtarın tek başına çıktı
	// üretmemesi çoğu zaman hata değil — yanına bir değer girilmesini bekliyor
	// (yazı tipi seçimi, arka plan görseli, logo metni). Burada da
	// raporlanırlarsa aynı şey iki kez, biri yanıltıcı biçimde görünüyor.
	if (key === "on" || /On$/.test(String(key))) return;

	const before = output(base);
	let changed = false;
	for (const candidate of options) {
		if (candidate === value) continue;
		holder[key] = candidate;
		if (output(base) !== before) {
			changed = true;
			break;
		}
	}
	holder[key] = value;

	if (changed) return;
	if (!hasControl(path)) {
		skipped.push(`${path} (arayüzde kontrolü yok)`);
		return;
	}
	dead.push(`${path}  (değer: ${JSON.stringify(value)})`);
}

// Bütün bölümler AÇIK: kapalı bir bölüm zaten hiçbir şey yazmıyor, o ayrı bir
// soru (`adoptDoc` bölümü içe aktarmada açıyor mu).
const base = enableEverySection(defaultAdv());

for (const section of Object.keys(base)) walk(section, base, section);

// --- Ayrı soru: bölüm KAPALIYKEN açmak çıktıyı değiştiriyor mu ---------------
const offState = defaultAdv();
const emptyOutput = output(offState);
const sectionsWithoutEffect = [];
for (const section of Object.keys(offState)) {
	const probe = clone(offState);
	const target = probe[section];
	// Bölümün açma anahtarı `on` olmayabilir (logo: `imageOn`/`textOn`).
	const switches = Object.keys(target).filter((k) => k === "on" || k.endsWith("On"));
	if (switches.length === 0) continue;
	for (const flag of switches) target[flag] = true;
	if (output(probe) === emptyOutput) sectionsWithoutEffect.push(`${section} (${switches.join(", ")})`);
}

console.log(`Denetlenen bölüm: ${Object.keys(base).length}`);
console.log(`Atlanan alan   : ${skipped.length}`);

if (sectionsWithoutEffect.length) {
	console.log(`\nAÇILDIĞINDA HİÇBİR ŞEY YAZMAYAN BÖLÜM: ${sectionsWithoutEffect.length}`);
	for (const line of sectionsWithoutEffect) console.log(`  ${line}`);
}

console.log(`\nOYNATILDIĞINDA ÇIKTIYI DEĞİŞTİRMEYEN ALAN: ${dead.length}`);
for (const line of dead) console.log(`  ${line}`);

if (dead.length || sectionsWithoutEffect.length) process.exit(1);
console.log("\nHer kontrol çıktıyı değiştiriyor.");
