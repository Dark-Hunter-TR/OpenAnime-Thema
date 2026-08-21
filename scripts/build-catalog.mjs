// Kod editörünün otomatik tamamlama kataloğunu ÜRETİR — elle yazılmaz.
//
// İki kaynaktan besleniyor:
//   1) node_modules/fluent-svelte-extra/{theme,switchable}.css  -> --fds-* token'ları,
//      grup başlıkları ve açık/koyu varsayılan değerleri
//   2) https://openani.me canlı CSS bundle'ları -> sitede gerçekten kullanılan
//      class / id isimleri ve fds dışı custom property'ler
//
// Böylece kütüphane ya da site güncellendiğinde katalog `bun run catalog` ile
// kendiliğinden tazelenir.
//
// Çıktı: src/lib/catalog.generated.ts

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SITE = "https://openani.me";
const UA =
	"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";

/** `sel {` ile başlayan bloğun gövdesini süslü parantez sayarak çıkarır. */
function blockBody(css, selector) {
	const start = css.indexOf(selector);
	if (start === -1) return null;
	const open = css.indexOf("{", start);
	if (open === -1) return null;
	let depth = 0;
	for (let i = open; i < css.length; i++) {
		if (css[i] === "{") depth++;
		else if (css[i] === "}") {
			depth--;
			if (depth === 0) return css.slice(open + 1, i);
		}
	}
	return null;
}

/** Bir blok gövdesindeki `/* Grup *​/` başlıklarını ve altındaki token'ları eşler. */
function parseDeclarations(body) {
	const out = [];
	let group = "Diğer";
	// Yorumları ve bildirimleri sırayla gez.
	const re = /\/\*([^*]*(?:\*(?!\/)[^*]*)*)\*\/|(--[a-zA-Z0-9-]+)\s*:\s*([^;]+);/g;
	let m;
	while ((m = re.exec(body))) {
		if (m[1] !== undefined) {
			const text = m[1].trim();
			// "--fds-x: y;" biçimindeki yorumlu (devre dışı) satırları grup sanma.
			if (text && !text.startsWith("--")) group = text;
		} else {
			out.push({ name: m[2], group, value: m[3].trim().replace(/\s+/g, " ") });
		}
	}
	return out;
}

// --- 1) Kütüphane token'ları ------------------------------------------------

const themeCss = readFileSync(
	resolve(ROOT, "node_modules/fluent-svelte-extra/theme.css"),
	"utf8"
);
const switchableCss = readFileSync(
	resolve(ROOT, "node_modules/fluent-svelte-extra/switchable.css"),
	"utf8"
);

const rootDecls = parseDeclarations(blockBody(themeCss, ":root") ?? "");
const lightDecls = parseDeclarations(blockBody(switchableCss, ".fds-theme-light") ?? "");
const darkDecls = parseDeclarations(blockBody(switchableCss, ".fds-theme-dark") ?? "");

const tokens = new Map();
const put = (decl, key) => {
	const t = tokens.get(decl.name) ?? { name: decl.name, group: decl.group };
	t[key] = decl.value;
	if (decl.group && decl.group !== "Diğer") t.group = decl.group;
	tokens.set(decl.name, t);
};
rootDecls.forEach((d) => put(d, "root"));
lightDecls.forEach((d) => put(d, "light"));
darkDecls.forEach((d) => put(d, "dark"));

// Uzun data: URI'leri katalogda taşımanın anlamı yok — kırp.
for (const t of tokens.values()) {
	for (const k of ["root", "light", "dark"]) {
		if (t[k] && t[k].length > 120) t[k] = t[k].slice(0, 100) + "…";
	}
}

// --- 2) Sitenin canlı CSS'i -------------------------------------------------

async function fetchSiteCss() {
	const res = await fetch(SITE + "/", { headers: { "user-agent": UA } });
	const html = await res.text();
	const files = [
		...new Set(
			[...html.matchAll(/\.\/(__openanime\/immutable\/assets\/[A-Za-z0-9_\-.]+\.css)/g)].map(
				(m) => m[1]
			)
		)
	];
	if (!files.length) throw new Error("sitede CSS bundle bulunamadı");
	const parts = await Promise.all(
		files.map((f) =>
			fetch(`${SITE}/${f}`, { headers: { "user-agent": UA } })
				.then((r) => r.text())
				.catch(() => "")
		)
	);
	return { css: parts.join("\n"), count: files.length };
}

let siteClasses = [];
let siteIds = [];
let siteVars = [];

try {
	const { css, count } = await fetchSiteCss();
	console.log(`  siteden ${count} CSS bundle indirildi (${css.length} bayt)`);

	// Selector metnini elde etmek için bildirim bloklarını at.
	const selectorText = css.replace(/\{[^{}]*\}/g, " ");

	const isSvelteHash = (n) => /^svelte-[a-z0-9]{6,}$/.test(n);

	siteClasses = [
		...new Set(
			[...selectorText.matchAll(/\.(-?[_a-zA-Z][\w-]*)/g)]
				.map((m) => m[1])
				.filter((n) => !isSvelteHash(n))
		)
	].sort();

	siteIds = [...new Set([...selectorText.matchAll(/#(-?[_a-zA-Z][\w-]*)/g)].map((m) => m[1]))].sort();

	siteVars = [
		...new Set(
			[...css.matchAll(/(--[a-zA-Z0-9-]+)\s*:/g)]
				.map((m) => m[1])
				.filter((n) => !n.startsWith("--fds-"))
		)
	].sort();
} catch (e) {
	console.warn(`  UYARI: site CSS'i alınamadı (${e.message}). Class kataloğu boş kalacak.`);
}

// --- 3) Yaz -----------------------------------------------------------------

const tokenList = [...tokens.values()].sort((a, b) => a.name.localeCompare(b.name));

const out = `// OTOMATİK ÜRETİLDİ — elle düzenlemeyin.
// Yenilemek için: bun run catalog   (scripts/build-catalog.mjs)
// Kaynaklar: fluent-svelte-extra@${JSON.parse(readFileSync(resolve(ROOT, "node_modules/fluent-svelte-extra/package.json"), "utf8")).version} + ${SITE}
// Üretim tarihi: ${new Date().toISOString().slice(0, 10)}

export interface TokenInfo {
	name: string;
	group: string;
	/** Moddan bağımsız varsayılan (theme.css :root). */
	root?: string;
	/** .fds-theme-light altındaki varsayılan. */
	light?: string;
	/** .fds-theme-dark altındaki varsayılan. */
	dark?: string;
}

/** fluent-svelte-extra'nın resmi tema token'ları. */
export const FDS_TOKENS: TokenInfo[] = ${JSON.stringify(tokenList, null, "\t")};

/** openani.me'nin CSS'inde geçen class isimleri (Svelte hash'leri ayıklandı). */
export const SITE_CLASSES: string[] = ${JSON.stringify(siteClasses, null, "\t")};

/** openani.me'nin CSS'inde geçen id'ler. */
export const SITE_IDS: string[] = ${JSON.stringify(siteIds, null, "\t")};

/** Sitenin fds dışı custom property'leri. Stable API DEĞİL — çoğu
 *  Svelte scoped selector'lara bağlı ve her deploy'da değişebilir. */
export const SITE_VARS: string[] = ${JSON.stringify(siteVars, null, "\t")};
`;

mkdirSync(resolve(ROOT, "src/lib"), { recursive: true });
writeFileSync(resolve(ROOT, "src/lib/catalog.generated.ts"), out, "utf8");

console.log(
	`✔ src/lib/catalog.generated.ts yazıldı — ` +
		`${tokenList.length} token, ${siteClasses.length} class, ${siteIds.length} id, ${siteVars.length} site değişkeni`
);
