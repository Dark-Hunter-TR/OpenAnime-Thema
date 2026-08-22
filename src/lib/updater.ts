/**
 * Uygulama içi güncelleyici — Rust tarafının ince sarmalayıcısı.
 *
 * Kontrol ve indirme `src-tauri/src/updater.rs` içinde yaşıyor. Sebebi orada
 * uzun uzun yazılı; kısaca: güncelleme KANALI endpoint'i çalışma anında
 * değiştirmeyi gerektiriyor ve `@tauri-apps/plugin-updater`'ın JS API'si bunu
 * yapamıyor. Bu dosya yalnızca çağrı yüzeyi ve olay akışının söze dökülmüş
 * hâli.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/** `AppSettings["updateChannel"]` ile eş; Rust'taki `Channel` ile de. */
export type UpdateChannel = "stable" | "beta" | "alpha";

/**
 * Bir kontrolün sonucu.
 *
 * `channelEmpty` ile `available: false` FARKLI: birincisi "bu kanaldan hiç
 * sürüm çıkmamış", ikincisi "zaten en günceli kullanıyorsun". Kullanıcıya
 * "stable sürüm mevcut değil" diyebilmek için ayrımın taşınması gerekiyor.
 */
export interface UpdateCheck {
	channelEmpty: boolean;
	available: boolean;
	channel: UpdateChannel;
	/** "Stable" | "Beta" | "Alpha" — metin Rust tarafında üretiliyor. */
	channelLabel: string;
	version: string | null;
	date: string | null;
	body: string | null;
	/** Kanaldaki en son sürüm; güncelleme olmasa da dolu. */
	latestVersion: string | null;
}

export interface UpdateProgress {
	status: "downloading" | "installing" | "success" | "error";
	downloaded: number;
	/** Sunucu `Content-Length` vermezse `null`. */
	total: number | null;
	percent: number;
	message: string | null;
}

const PROGRESS_EVENT = "openanime://update-progress";

/**
 * Seçili kanalda güncelleme olup olmadığını sorar.
 *
 * @param force Önbelleği atlar. Ayarlar'daki "şimdi kontrol et" için: aksi
 * hâlde düğmeye basan kullanıcı beş dakika boyunca aynı yanıtı alırdı.
 */
export function checkForUpdate(channel: UpdateChannel, force = false): Promise<UpdateCheck> {
	return invoke<UpdateCheck>("updater_check", { channel, force });
}

/**
 * Son kontrolde bulunan güncellemeyi indirir ve kurar.
 *
 * Rust tarafı kurulum biter bitmez uygulamayı yeniden başlatıyor, dolayısıyla
 * NORMAL akışta bu söz hiç çözülmez — süreç önce ölür. Yine de "success"
 * olayında çözüyoruz: yeniden başlatma bir sebeple gecikirse çağıran taraf
 * sonsuza kadar beklememeli.
 */
export async function downloadAndInstallUpdate(
	onProgress: (progress: UpdateProgress) => void
): Promise<void> {
	// Dinleyici çağrıdan önce kuruluyor: indirme Rust tarafında ayrı bir
	// görevde başlıyor ve ilk ilerleme olayı `invoke` daha dönmeden gelebilir.
	let unlisten: (() => void) | undefined;

	try {
		await new Promise<void>((resolve, reject) => {
			listen<UpdateProgress>(PROGRESS_EVENT, (event) => {
				onProgress(event.payload);
				if (event.payload.status === "success") {
					resolve();
				} else if (event.payload.status === "error") {
					reject(new Error(event.payload.message ?? "Güncelleme başarısız."));
				}
			})
				.then((fn) => {
					unlisten = fn;
					// Komutun kendisi hemen dönüyor; hata yalnızca "indirilecek
					// bir şey yok" / "zaten sürüyor" durumlarında geliyor.
					return invoke("updater_download");
				})
				.catch(reject);
		});
	} finally {
		unlisten?.();
	}
}
