import { FDS_TOKENS, type TokenInfo } from "$lib/catalog.generated";

/**
 * Kod editörü için başlangıç şablonu üretir.
 *
 * Kullanıcı boş bir sayfayla karşılaşmasın diye sitenin KENDİ mevcut token
 * değerleri, düzenlenebilir hâlde yazılır. Önce moddan bağımsız `:root`, sonra
 * `html.fds-theme-light` ve `html.fds-theme-dark` blokları yer alır.
 *
 * Katalogda kısaltılmış (`…` içeren) uzun data URI'leri atlanır — aksi hâlde
 * geçersiz CSS üretirdik.
 */

const TRUNCATED = "…";

function groupBy(tokens: TokenInfo[], key: "root" | "light" | "dark") {
	const groups = new Map<string, { name: string; value: string }[]>();
	for (const token of tokens) {
		const value = token[key];
		if (!value || value.includes(TRUNCATED)) continue;
		if (!groups.has(token.group)) groups.set(token.group, []);
		groups.get(token.group)!.push({ name: token.name, value });
	}
	return groups;
}

function block(selector: string, title: string, groups: Map<string, { name: string; value: string }[]>) {
	if (groups.size === 0) return "";
	const lines: string[] = [`/* ---------- ${title} ---------- */`, `${selector} {`];
	for (const [group, entries] of groups) {
		lines.push(`\t/* ${group} */`);
		for (const { name, value } of entries) lines.push(`\t${name}: ${value};`);
		lines.push("");
	}
	// son boş satırı at
	if (lines[lines.length - 1] === "") lines.pop();
	lines.push("}");
	return lines.join("\n");
}

export function starterTemplate(): string {
	const header = [
		"/* ============================================================",
		"   OpenAnime tema şablonu",
		"",
		"   Aşağıdaki değerler sitenin ŞU ANKİ varsayılanlarıdır — hepsini",
		"   doğrudan düzenleyebilirsiniz.",
		"",
		"   Not: soldaki görsel kontroller yukarıdaki <oa:tokens> bloğunu",
		"   yönetir. Buradaki değerler o bloğun DIŞINDA olduğu için",
		"   kontrollerle çakışmaz; ikisi de aynı CSS'e katkı verir ve",
		"   sonra gelen (bu blok) kazanır.",
		"   ============================================================ */"
	].join("\n");

	const parts = [
		header,
		block(":root", "Moddan bağımsız", groupBy(FDS_TOKENS, "root")),
		block("html.fds-theme-light", "Açık tema", groupBy(FDS_TOKENS, "light")),
		block("html.fds-theme-dark", "Koyu tema", groupBy(FDS_TOKENS, "dark"))
	].filter(Boolean);

	return parts.join("\n\n") + "\n";
}

/** Şablonun zaten yüklü olup olmadığını anlamak için kullanılan imza. */
export const STARTER_MARKER = "OpenAnime tema şablonu";
