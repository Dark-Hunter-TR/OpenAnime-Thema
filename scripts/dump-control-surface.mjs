/**
 * Kontrollerin yazabildiği HER token ve kuralı JSON'a döker.
 *
 * Amaç denetim: "kullanıcı bu kontrolü oynattığında üretilen CSS gerçekten
 * değişiyor mu?" sorusunu tahminle değil listeyle cevaplamak için. Liste elle
 * yazılmıyor — `enableEverySection` ile bütün bölümler açılıp `buildAdvTokens`
 * / `buildAdvRules` çalıştırılıyor, yani kaynak neyi üretebiliyorsa o.
 *
 * Çıktı `src-tauri/tests/control-surface.json`; Rust tarafındaki denetim
 * testi bunu okuyor. İki dilin arasında elle senkronlanan bir kopya
 * bırakmamak için üretiliyor.
 *
 * Çalıştırma: `bun scripts/dump-control-surface.mjs`
 */

import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

const {
	ADV_TOKEN_KEYS,
	buildAdvRules,
	buildAdvTokens,
	defaultAdv,
	enableEverySection
} = await import(resolve(root, "src/lib/advancedBuild.ts"));
const { KNOWN_SELECTORS } = await import(resolve(root, "src/lib/advanced.ts"));
const { BUTTON_TOKENS, DURATION_TOKENS, EASING_TOKEN, HOVER_TOKENS } = await import(
	resolve(root, "src/lib/customization.ts")
);
const { parseDeclarations } = await import(resolve(root, "src/lib/cssDecls.ts"));

const everything = enableEverySection(defaultAdv());

const tokenName = (t) => (typeof t === "string" ? t : t.token);

// Bölümlerin gerçekten ürettikleri + evrensel anahtar listesi. İkisinin
// birleşimi alınıyor: biri diğerini kaçırırsa denetim yine de görür.
const tokens = new Set([
	...ADV_TOKEN_KEYS.map(tokenName),
	...HOVER_TOKENS.map(tokenName),
	...BUTTON_TOKENS.map(tokenName),
	...DURATION_TOKENS.map(tokenName),
	EASING_TOKEN,
	...Object.keys(buildAdvTokens(everything))
]);

// Selector -> kontrollerin o kuralda SAHİP OLDUĞU özellikler.
const rules = {};
for (const [selector, body] of Object.entries(buildAdvRules(everything))) {
	rules[selector] = parseDeclarations(body).map((d) => d.property);
}
for (const selector of KNOWN_SELECTORS) {
	if (!rules[selector]) rules[selector] = [];
}

const out = {
	note: "Üretilmiş dosya — `bun scripts/dump-control-surface.mjs`.",
	tokens: [...tokens].filter(Boolean).sort(),
	rules
};

const target = resolve(root, "src-tauri/tests/control-surface.json");
mkdirSync(dirname(target), { recursive: true });
writeFileSync(target, JSON.stringify(out, null, "\t") + "\n", "utf8");

console.log(`${out.tokens.length} token, ${Object.keys(rules).length} kural -> ${target}`);
