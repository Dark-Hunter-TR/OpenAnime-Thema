/**
 * GitHub bağlantısından tema (.css) çekme.
 *
 * Ağ çağrısı neden Rust'ta DEĞİL: `raw.githubusercontent.com` ve
 * `api.github.com` ikisi de `Access-Control-Allow-Origin: *` gönderiyor ve
 * uygulamanın CSP'si kapalı (`tauri.conf.json` -> `security.csp: null`), yani
 * webview'den doğrudan `fetch` etmenin önünde engel yok. Rust tarafında yapmak
 * `reqwest`'e bir TLS arka ucu (rustls/ring ya da native-tls) eklemeyi
 * gerektirirdi — Cargo.lock'taki `reqwest` şu an TLS'siz geliyor. Mimarinin
 * gerçek değeri olan kısım, yani çekilen CSS'in kontrollere EŞLENMESİ, zaten
 * Rust'ta (`parse_foreign_css`).
 */

const API = "https://api.github.com";
const RAW_HOST = "raw.githubusercontent.com";

/** Depoda bulunan aday tema dosyası. */
export interface GithubFile {
	/** Depo köküne göre yol — kullanıcıya bu gösteriliyor. */
	path: string;
	rawUrl: string;
	/** Bilinmiyorsa 0 (ham/tek dosya bağlantılarında boyut önceden bilinmez). */
	size: number;
}

export interface ResolveResult {
	/** En olası tema dosyası başta olacak şekilde sıralı adaylar. */
	files: GithubFile[];
	/** Kullanıcının girdiği bağlantının normalize hâli — projeye köken olarak yazılır. */
	source: string;
}

/** Ağ hatalarını kullanıcıya gösterilebilir tek bir tipte topluyoruz. */
export class ImportError extends Error {}

/** 8 MB üstü bir dosya tema değildir; indirmeden önce durduruyoruz. */
const MAX_BYTES = 8 * 1024 * 1024;

const TIMEOUT_MS = 20_000;

async function request(url: string, accept?: string): Promise<Response> {
	const controller = new AbortController();
	const timer = setTimeout(() => controller.abort(), TIMEOUT_MS);

	let response: Response;
	try {
		response = await fetch(url, {
			signal: controller.signal,
			headers: accept ? { Accept: accept } : undefined
		});
	} catch (e) {
		if (controller.signal.aborted) {
			throw new ImportError("GitHub yanıt vermedi (zaman aşımı). İnternet bağlantınızı kontrol edin.");
		}
		throw new ImportError(`GitHub'a bağlanılamadı: ${e instanceof Error ? e.message : String(e)}`);
	} finally {
		clearTimeout(timer);
	}

	if (response.ok) return response;

	if (response.status === 404) {
		throw new ImportError(
			"Bulunamadı (404). Bağlantının doğru ve deponun herkese açık olduğundan emin olun — özel depolar okunamaz."
		);
	}
	if (response.status === 403 || response.status === 429) {
		// Kimliksiz istekler için GitHub saatte 60 istek veriyor.
		const remaining = response.headers.get("x-ratelimit-remaining");
		if (remaining === "0") {
			throw new ImportError(
				"GitHub API saatlik istek sınırına takıldı (kimliksiz istekler için 60/saat). " +
					"Bir süre sonra tekrar deneyin ya da doğrudan dosya bağlantısı girin " +
					"(ör. .../blob/main/tema.css)."
			);
		}
		throw new ImportError("GitHub erişimi reddetti (403). Depo özel olabilir.");
	}

	throw new ImportError(`GitHub beklenmedik bir yanıt verdi: ${response.status} ${response.statusText}`);
}

/**
 * Girilen bağlantıyı çözümleyip aday tema dosyalarını döndürür.
 *
 * Desteklenen biçimler:
 *   - `https://raw.githubusercontent.com/<sahip>/<depo>/<dal>/<yol>.css`
 *   - `https://github.com/<sahip>/<depo>/blob/<dal>/<yol>.css`
 *   - `https://github.com/<sahip>/<depo>/tree/<dal>/<klasör>`
 *   - `https://github.com/<sahip>/<depo>`
 *   - `https://gist.github.com/<kullanıcı>/<id>`
 *   - `<sahip>/<depo>`
 */
export async function resolveThemeFiles(input: string): Promise<ResolveResult> {
	const raw = input.trim();
	if (!raw) throw new ImportError("Bir GitHub bağlantısı girin.");

	// Protokolsüz `sahip/depo` kısayolu.
	if (!/^https?:\/\//i.test(raw)) {
		const short = raw.replace(/^github\.com\//i, "").replace(/\.git$/i, "");
		if (/^[\w.-]+\/[\w.-]+$/.test(short)) {
			const [owner, repo] = short.split("/");
			return { files: await listRepoCss(owner, repo, null, ""), source: `https://github.com/${owner}/${repo}` };
		}
		throw new ImportError(
			"Bağlantı anlaşılamadı. Bir GitHub adresi ya da `sahip/depo` biçiminde bir ad girin."
		);
	}

	let url: URL;
	try {
		url = new URL(raw);
	} catch {
		throw new ImportError("Geçersiz bağlantı. Tam bir adres girin (https://github.com/...).");
	}

	const host = url.hostname.replace(/^www\./, "");
	const parts = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);

	if (host === RAW_HOST) {
		const path = parts.slice(3).join("/") || parts.join("/");
		return { files: [{ path, rawUrl: url.toString(), size: 0 }], source: url.toString() };
	}

	if (host === "gist.github.com") {
		const id = parts[parts.length - 1];
		if (!id) throw new ImportError("Gist kimliği okunamadı.");
		return { files: await listGistCss(id), source: url.toString() };
	}

	if (host !== "github.com") {
		throw new ImportError(`Bu adres GitHub'a ait değil: ${host}`);
	}

	const [owner, repoRaw, kind, ...rest] = parts;
	if (!owner || !repoRaw) {
		throw new ImportError("Bağlantıda depo adı yok. Örnek: https://github.com/sahip/depo");
	}
	const repo = repoRaw.replace(/\.git$/i, "");
	const source = `https://github.com/${owner}/${repo}`;

	// Tek dosyaya doğrudan bağlantı.
	if ((kind === "blob" || kind === "raw") && rest.length >= 2) {
		const [ref, ...pathParts] = rest;
		const path = pathParts.join("/");
		return {
			files: [{ path, rawUrl: rawUrlFor(owner, repo, ref, path), size: 0 }],
			source: url.toString()
		};
	}

	// Klasöre bağlantı.
	if (kind === "tree" && rest.length >= 1) {
		const [ref, ...pathParts] = rest;
		return { files: await listRepoCss(owner, repo, ref, pathParts.join("/")), source: url.toString() };
	}

	// Depo kökü.
	return { files: await listRepoCss(owner, repo, null, ""), source };
}

function rawUrlFor(owner: string, repo: string, ref: string, path: string): string {
	const encoded = path.split("/").map(encodeURIComponent).join("/");
	return `https://${RAW_HOST}/${owner}/${repo}/${encodeURIComponent(ref)}/${encoded}`;
}

async function defaultBranch(owner: string, repo: string): Promise<string> {
	const response = await request(`${API}/repos/${owner}/${repo}`, "application/vnd.github+json");
	const info = (await response.json()) as { default_branch?: string };
	return info.default_branch || "main";
}

/** Depodaki (ya da bir alt klasördeki) tüm `.css` dosyalarını bulur. */
async function listRepoCss(
	owner: string,
	repo: string,
	ref: string | null,
	prefix: string
): Promise<GithubFile[]> {
	const branch = ref ?? (await defaultBranch(owner, repo));

	// Tek çağrıda tüm ağaç: klasör klasör gezinmekten hem hızlı hem de saatlik
	// istek sınırına çok daha az yükleniyor.
	const response = await request(
		`${API}/repos/${owner}/${repo}/git/trees/${encodeURIComponent(branch)}?recursive=1`,
		"application/vnd.github+json"
	);
	const tree = (await response.json()) as {
		tree?: { path: string; type: string; size?: number }[];
		truncated?: boolean;
	};

	const normalizedPrefix = prefix ? `${prefix.replace(/\/+$/, "")}/` : "";
	const files = (tree.tree ?? [])
		.filter((node) => node.type === "blob" && node.path.toLowerCase().endsWith(".css"))
		.filter((node) => !normalizedPrefix || node.path.startsWith(normalizedPrefix))
		.map((node) => ({
			path: node.path,
			rawUrl: rawUrlFor(owner, repo, branch, node.path),
			size: node.size ?? 0
		}));

	if (files.length === 0) {
		throw new ImportError(
			tree.truncated
				? "Depo listelenemeyecek kadar büyük. Doğrudan CSS dosyasının bağlantısını girin."
				: "Bu depoda .css dosyası bulunamadı."
		);
	}

	return rankThemeFiles(files);
}

async function listGistCss(id: string): Promise<GithubFile[]> {
	const response = await request(`${API}/gists/${id}`, "application/vnd.github+json");
	const gist = (await response.json()) as {
		files?: Record<string, { filename: string; raw_url: string; size?: number } | null>;
	};

	const files = Object.values(gist.files ?? {})
		.filter((file): file is { filename: string; raw_url: string; size?: number } => !!file)
		.filter((file) => file.filename.toLowerCase().endsWith(".css"))
		.map((file) => ({ path: file.filename, rawUrl: file.raw_url, size: file.size ?? 0 }));

	if (files.length === 0) throw new ImportError("Bu gist'te .css dosyası yok.");
	return rankThemeFiles(files);
}

/**
 * Adayları "en olası tema dosyası" sırasına dizer.
 *
 * Sıralama tahmin değil, gözleme dayanıyor: tema depolarında dosya ya
 * `theme.css` / `tema.css` gibi adlandırılıyor ya da kökte duruyor. Derinlerde
 * duran `dist/`, `node_modules/` gibi yollar neredeyse hiç tema olmuyor.
 */
function rankThemeFiles(files: GithubFile[]): GithubFile[] {
	const score = (file: GithubFile) => {
		const path = file.path.toLowerCase();
		const name = path.split("/").pop() ?? path;
		let points = 0;

		if (/(^|[^a-z])(theme|tema)([^a-z]|$)/.test(name)) points += 100;
		if (name.includes("openanime")) points += 60;
		// Derinlik cezası: kökteki dosya daha olası.
		points -= (path.split("/").length - 1) * 10;
		// Üretim çıktıları ve bağımlılıklar tema olmaz.
		if (/(^|\/)(node_modules|dist|build|vendor)\//.test(path)) points -= 200;
		if (name.endsWith(".min.css")) points -= 30;

		return points;
	};

	return [...files].sort((a, b) => score(b) - score(a) || a.path.localeCompare(b.path));
}

/** Seçilen dosyanın içeriğini indirir ve gerçekten CSS olduğunu doğrular. */
export async function fetchCss(file: GithubFile): Promise<string> {
	if (file.size > MAX_BYTES) {
		throw new ImportError(
			`Dosya çok büyük (${Math.round(file.size / 1024)} KB). Tema dosyaları en fazla ${MAX_BYTES / 1024 / 1024} MB olabilir.`
		);
	}

	const response = await request(file.rawUrl);
	const text = await response.text();

	if (text.length > MAX_BYTES) {
		throw new ImportError("Dosya çok büyük; tema olarak açılamadı.");
	}

	assertLooksLikeCss(text, file.path);
	return text;
}

/**
 * "CSS içermeyen dosya" durumunu erkenden yakalar.
 *
 * En sık karşılaşılan hata, ham bağlantı yerine GitHub'ın HTML sayfasının
 * çekilmesi; o durumda kullanıcıya "CSS değil" demek yerine ne yapması
 * gerektiğini söylüyoruz.
 */
function assertLooksLikeCss(text: string, where: string): void {
	const trimmed = text.trim();
	if (!trimmed) throw new ImportError(`${where} boş.`);

	const head = trimmed.slice(0, 500).toLowerCase();
	if (head.startsWith("<!doctype") || head.startsWith("<html") || head.includes("<head>")) {
		throw new ImportError(
			`${where} bir CSS dosyası değil, HTML sayfası. Dosyanın "Raw" bağlantısını kullanmayı deneyin.`
		);
	}

	const hasRule = /\{[^}]*\}/.test(trimmed);
	const hasCustomProperty = /--[\w-]+\s*:/.test(trimmed);
	if (!hasRule && !hasCustomProperty) {
		throw new ImportError(`${where} içinde CSS kuralı bulunamadı.`);
	}
}

/** Dosya yolundan proje adı önerir: `themes/sakura.css` -> `sakura`. */
export function suggestProjectName(file: GithubFile): string {
	const name = (file.path.split("/").pop() ?? file.path).replace(/\.css$/i, "");
	const cleaned = name.replace(/[-_]+/g, " ").trim();
	if (!cleaned) return "İçe aktarılan tema";
	return cleaned.charAt(0).toLocaleUpperCase("tr-TR") + cleaned.slice(1);
}
