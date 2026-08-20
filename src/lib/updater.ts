/**
 * Uygulama içi güncelleyici — ince bir sarmalayıcı.
 *
 * Mantığın tamamı `@tauri-apps/plugin-updater`'ın kendisinde: sürüm
 * karşılaştırması, imza doğrulaması (bkz. `tauri.conf.json` ->
 * `plugins.updater.pubkey`) ve indirme. Bu dosya yalnızca proje genelinde
 * tek bir tip/adlandırma sağlıyor; ayrı bir Rust komutu YOK — eklentinin
 * kendi `check`/`downloadAndInstall` çağrıları IPC'yi zaten kapsıyor.
 */
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";
import { invoke } from "@tauri-apps/api/core";

export type { Update, DownloadEvent };

/**
 * Güncelleme olup olmadığını sorar.
 *
 * Sonuç `null` ise güncel demektir. Ağ hatası ya da imza doğrulaması
 * başarısızlığında `check()` reddediyor — çağıran taraf bunu sessizce
 * yutmalı (bkz. `+page.svelte` -> `checkForUpdateOnStartup`): bir güncelleme
 * kontrolünün başarısız olması editörü kullanılamaz hâle getirmemeli.
 */
export const checkForUpdate = () => check();

/**
 * Güncellemeyi indirir ve kurar; başarılı olursa uygulamayı yeniden başlatır.
 *
 * `update.downloadAndInstall` NSIS installer'ı indirip çalıştırır (Windows'ta
 * mevcut süreç installer tarafından üzerine yazılamaz), bu yüzden kurulumdan
 * sonra biz kendimiz kapanıp yeniden açılmamız gerekiyor — `restart_app`
 * (bkz. `lib.rs`) tam olarak bunu yapıyor. Kısa bir bekleme koyuyoruz ki
 * `onEvent`'in son "Finished" olayı arayüze ulaşıp görünsün; `restart`
 * hiç dönmeden süreci sonlandırıyor.
 */
export async function downloadAndInstallUpdate(
	update: Update,
	onEvent: (event: DownloadEvent) => void
): Promise<void> {
	await update.downloadAndInstall(onEvent);
	await new Promise((resolve) => setTimeout(resolve, 400));
	await invoke("restart_app");
}
