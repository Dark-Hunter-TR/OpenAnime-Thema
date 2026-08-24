import { invoke } from "@tauri-apps/api/core";

/**
 * Uygulamayı kaldırma köprüsü (yalnızca Windows).
 *
 * Silme işini uygulama YAPMIYOR; Windows kurulum paketinin kendi
 * kaldırıcısını (`uninstall.exe`) çalıştırıyor. Gerekçesi ve neyin silindiği
 * `src-tauri/src/lib.rs` -> `run_uninstaller` yorumunda.
 */

/**
 * Kaldırıcı bu kopyanın yanında duruyor mu?
 *
 * `false` dönerse ayarlardaki satır hiç gösterilmiyor: geliştirme derlemesi,
 * taşınabilir kopya ya da Windows dışı bir platformdayız demektir ve
 * kaldırılacak bir kurulum yok.
 */
export const uninstallerAvailable = () => invoke<boolean>("uninstaller_available");

/**
 * Kaldırıcıyı başlatır ve uygulamadan çıkar.
 *
 * DÖNEN SÖZ ÇÖZÜLMEYEBİLİR: Rust komutu kaldırıcıyı başlattıktan hemen sonra
 * uygulamayı kapatıyor, dolayısıyla IPC yanıtı ön yüze ulaşamadan süreç
 * ölebiliyor. Çağıran taraf bu çağrıyı bekleyip arayüzü kilitlememeli —
 * yalnızca REDDİ (kaldırıcı bulunamadı, başlatılamadı) ele almalı.
 */
export const runUninstaller = () => invoke<void>("run_uninstaller");
