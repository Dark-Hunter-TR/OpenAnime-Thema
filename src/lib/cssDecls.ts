/**
 * CSS bildirim (declaration) yardımcıları.
 *
 * Neden ayrı bir modül: kural gövdeleri iki farklı yerden geliyor — biri
 * kontrollerin ürettiği metin, diğeri kullanıcının açtığı dosyadan olduğu gibi
 * alınan metin. İkisini BİRLEŞTİRMEK zorundayız (gerekçe:
 * `mergeRuleOverrides`) ve birleştirme, gövdeyi tek parça metin olarak değil
 * bildirim bildirim ele almayı gerektiriyor.
 */

/**
 * Gövdeyi ÜST DÜZEY `;` sınırlarından böler.
 *
 * `String.split(";")` neden yetmiyor: CSS değerlerinin içinde noktalı virgül
 * geçebiliyor — en yaygını `url(data:image/png;base64,…)`. Naif bölme o
 * bildirimi ortadan kesip `url(` parantezini açık bırakır; sonuç yalnızca o
 * değerin kaybı değil, çünkü açık parantez kendisinden sonraki bildirimleri de
 * yutar. Bu yüzden `;` ancak parantezlerin, tırnakların ve yorumların
 * dışındaysa sınır sayılıyor. (Rust tarafındaki karşılığı:
 * `src-tauri/src/theme/parse.rs` -> `split_top_level`.)
 */
export function splitDeclarations(body: string): string[] {
	const out: string[] = [];
	let start = 0;
	let depth = 0;
	let i = 0;

	while (i < body.length) {
		const c = body[i];

		if (c === "/" && body[i + 1] === "*") {
			const end = body.indexOf("*/", i + 2);
			i = end === -1 ? body.length : end + 2;
			continue;
		}
		if (c === '"' || c === "'") {
			let j = i + 1;
			while (j < body.length && body[j] !== c) {
				if (body[j] === "\\") j++;
				j++;
			}
			i = j + 1;
			continue;
		}

		if (c === "(") depth++;
		else if (c === ")") depth = Math.max(0, depth - 1);
		else if (c === ";" && depth === 0) {
			out.push(body.slice(start, i));
			start = i + 1;
		}
		i++;
	}

	if (start < body.length) out.push(body.slice(start));
	return out;
}

/** Tek bir bildirim. `important` ayrı tutulmuyor; `value`nun parçası. */
export interface Declaration {
	property: string;
	value: string;
}

/**
 * Gövdeyi bildirimlere çözer. Sıra korunuyor: CSS'te aynı özelliğin ikinci
 * yazımı birinciyi ezdiği için sıra anlam taşır.
 */
export function parseDeclarations(body: string): Declaration[] {
	const out: Declaration[] = [];
	for (const raw of splitDeclarations(body)) {
		// Yorumları burada atıyoruz: `--x: 4px /* not */` gibi satır sonu
		// açıklamaları gerçek temalarda kural, istisna değil.
		const decl = raw.replace(/\/\*[\s\S]*?\*\//g, "").trim();
		if (!decl) continue;

		// İlk `:` özellik sınırı — özellik adları iki nokta içeremez, bu yüzden
		// `url(https://…)` gibi değerler bunu bozmuyor.
		const at = decl.indexOf(":");
		if (at <= 0) continue;

		const property = decl.slice(0, at).trim();
		const value = decl.slice(at + 1).trim();
		if (!property || !value) continue;
		out.push({ property, value });
	}
	return out;
}

/** Bildirimleri tek satırlık bir gövdeye çevirir. */
export function formatDeclarations(decls: Declaration[]): string {
	return decls.map((d) => `${d.property}: ${d.value};`).join(" ");
}

/**
 * Kontrollerin ürettiği gövdeyi mevcut gövdenin ÜSTÜNE bindirir.
 *
 * Kural şu: kontrollerin sahip olduğu özellikler (`owned`) mevcut gövdeden
 * çıkarılır, sonra kontrolün o an ürettikleri eklenir. Kontrolün sahip
 * OLMADIĞI her bildirim olduğu gibi kalır.
 *
 * Gerekçe somut: ytanime teması `.slider.orientation-horizontal` kuralına
 * `block-size`, `inline-size`, `justify-content`, `position` ve `color`
 * yazıyor. Bu seçici uygulamanın "Oynatıcı" bölümüne bağlı olduğu için kural
 * kontrollere alınıyor; kontroller ise yalnızca birkaç özelliği biliyor.
 * Gövdeyi komple değiştiren eski yol, temanın yazdığı geri kalan her şeyi
 * kullanıcı hiçbir şeye dokunmadan siliyordu.
 */
export function mergeDeclarations(
	existingBody: string,
	controlBody: string,
	owned: Set<string>
): string {
	const kept = parseDeclarations(existingBody).filter((d) => !owned.has(d.property));
	const added = parseDeclarations(controlBody);

	// Kontrolün yazdığı bir özellik korunanlarda da varsa (ör. bölüm kapalıyken
	// dosyadan gelen bir değer), `owned` filtresi onu zaten çıkarmış olur.
	return formatDeclarations([...kept, ...added]);
}

/**
 * `ruleOverrides` haritasını kontrollerin çıktısıyla birleştirir.
 *
 * `universe`: kontrollerin yönettiği seçicilerin tamamı. Bir seçici bu kümede
 * değilse haritada olduğu gibi kalır — kontrollerin onunla bir işi yok.
 *
 * `owned`: seçici başına, kontrollerin YAZDIĞI özellik adları. Bir bölüm
 * kapatıldığında o özellikler çıkarılıyor ama aynı kuraldaki diğer bildirimler
 * duruyor; kural bildirimsiz kalırsa anahtar tamamen siliniyor ki üretilen
 * CSS'te boş bloklar birikmesin.
 */
export function mergeRuleOverrides(
	existing: Record<string, string>,
	controls: Record<string, string>,
	universe: readonly string[],
	owned: Record<string, Set<string>>
): Record<string, string> {
	const next: Record<string, string> = { ...existing };

	for (const selector of new Set([...universe, ...Object.keys(controls)])) {
		const currentBody = next[selector] ?? "";
		const controlBody = controls[selector] ?? "";
		if (!currentBody && !controlBody) {
			delete next[selector];
			continue;
		}

		const merged = mergeDeclarations(currentBody, controlBody, owned[selector] ?? new Set());
		if (merged.trim()) next[selector] = merged;
		else delete next[selector];
	}

	return next;
}

/**
 * Bir değerin içindeki ilk `url(...)` adresini çıkarır.
 *
 * Basit bir `/url\(\s*["']?([^"'\s)]+)["']?\s*\)/` deseni yetmiyordu ve bu,
 * içe aktarılan temaların LOGO GÖRSELİNİN hiç gelmemesinin sebebiydi. Gerçek
 * temalar logoyu gömülü bir SVG olarak veriyor:
 *
 *     --url-logo: url('data:image/svg+xml,<svg viewBox="0 0 24 24" …>…</svg>');
 *
 * Bu değerin içinde boşluk da var, `"` de, `/` de. Karakter sınıfına dayanan
 * desen ilk boşlukta kopuyor, kapanış parantezini bulamıyor ve hiç eşleşmiyor.
 *
 * Burada tırnak asıl alınıyor: tırnaklıysa kapanış tırnağına kadar okunuyor
 * (araya ne girerse girsin), tırnaksızsa kapanış parantezine kadar — CSS
 * tırnaksız `url()` içinde zaten boşluk ve parantez kabul etmiyor.
 */
export function extractUrl(value: string): string {
	const at = value.toLowerCase().indexOf("url(");
	if (at === -1) return "";

	let i = at + 4;
	while (i < value.length && /\s/.test(value[i])) i++;

	const quote = value[i];
	if (quote === '"' || quote === "'") {
		i++;
		let out = "";
		while (i < value.length && value[i] !== quote) {
			// Kaçırılmış tırnak değerin parçası.
			if (value[i] === "\\" && i + 1 < value.length) {
				out += value[i + 1];
				i += 2;
				continue;
			}
			out += value[i];
			i++;
		}
		return out.trim();
	}

	const end = value.indexOf(")", i);
	return (end === -1 ? value.slice(i) : value.slice(i, end)).trim();
}

/**
 * Bir kural gövdesinden sayısal bir uzunluk okur.
 *
 * Düz bir `/prop\s*:\s*(\d+)px/` araması yetmiyordu ve bu, içe aktarılan
 * temaların sayısal ayarlarının (logo büyüklüğü, kart yuvarlaklığı, kenar
 * çubuğu genişliği…) kontrollere hiç ulaşmamasının sebebiydi. Gerçek temalar
 * üç şeyi birden yapıyor:
 *
 *   width: var(--logo-size-desktop) !important;
 *   border-radius: var(--ayar-kose-yuvarlakligi-genel) !important;
 *   width: 4.5rem;
 *
 * Üçünde de eski desen boş dönüyor, kontrol temanın değerini değil site
 * varsayılanını gösteriyordu. Burada önce bildirim ayrıştırılıyor,
 * `!important` atılıyor, `var()` zinciri çözülüyor, sonra sayı okunuyor.
 *
 * `tokens` çözüm haritası: değişken adı -> değeri.
 */
export function resolveLength(
	rule: string | undefined,
	prop: string,
	tokens: Record<string, string>,
	unit: "px" | "rem" = "px"
): number | null {
	if (!rule) return null;

	// Özellik sınırı: `width` ararken `min-width` / `max-width`,
	// `border-radius` ararken `-webkit-border-radius` eşleşmemeli.
	const escaped = prop.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
	const decl = rule.match(new RegExp(`(?:^|[;{\\s])${escaped}\\s*:\\s*([^;]+)`, "i"));
	if (!decl) return null;

	let value = decl[1].replace(/!important/gi, "").trim();

	// `var(--a, 12px)` zincirini çöz. Beş tur — Rust tarafındaki `resolve_var`
	// ile aynı sınır: döngüsel tanımlarda takılmamak için.
	for (let i = 0; i < 5 && value.includes("var("); i++) {
		value = value.replace(/var\(\s*(--[\w-]+)\s*(?:,([^()]*))?\)/i, (_, name, fallback) =>
			(tokens[name] ?? fallback ?? "").trim()
		);
	}

	const match = value.match(new RegExp(`(-?[\\d.]+)\\s*${unit}\\b`, "i"));
	if (!match) return null;
	const number = Number(match[1]);
	return Number.isFinite(number) ? number : null;
}
