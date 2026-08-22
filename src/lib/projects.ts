/**
 * Yerel tema projeleri — IPC sarmalayıcıları ve editör durumunun şekli.
 *
 * Kalıcılık Rust tarafında (`src-tauri/src/projects.rs`), dosya başına bir
 * proje olacak şekilde `app_data_dir()/projects/` altında. Buradaki tipler o
 * dosyaların şemasıdır.
 *
 * Kritik ayrım: `doc` temanın kendisi (CSS ondan üretilir), `ui` ise görsel
 * kontrollerin durumu. İkisini ayrı tutmak zorundayız çünkü kontrollerin bir
 * kısmı CSS'e geri çözülemiyor — ör. bir bölümün KAPALI olması ile açık ama
 * varsayılan değerde olması aynı CSS'i üretir. `ui` olmadan proje geri
 * yüklendiğinde kullanıcı bıraktığı yerde değil, "her şey kapalı" halinde
 * bulurdu kendini.
 */

import { invoke } from "@tauri-apps/api/core";

import type { AdvState } from "$lib/advancedBuild";
import type { ColorState } from "$lib/defaults";
import type { ThemeDoc, ThemeMode } from "$lib/theme";

/** Ana ekrandaki kart için okunan hafif özet. */
export interface ProjectSummary {
	id: string;
	name: string;
	createdAt: number;
	updatedAt: number;
	/** Kart şeridini çizmek için — projenin tamamını okumaya gerek kalmasın. */
	accent: [number, number, number];
	mode: ThemeMode;
	source: string | null;
	/** Kullanıcının seçtiği kapak görseli (`data:` URI). Yoksa `null`. */
	coverImage: string | null;
}

/**
 * Görsel kontrollerin tam durumu.
 *
 * `version` ileride şema değişirse geçiş yapabilmek için. Şu an tek sürüm var;
 * okurken uyuşmayan sürümler görmezden geliniyor (bkz. `restoreUi`).
 */
export interface EditorUiState {
	version: 1;
	editMode: "visual" | "code";
	seedMode: ThemeMode;

	radiusEnabled: boolean;
	controlRadius: number;
	overlayRadius: number;

	hoverEnabled: boolean;
	hoverColors: ColorState[];

	buttonsEnabled: boolean;
	buttonColors: ColorState[];
	buttonTextHex: string;

	motionEnabled: boolean;
	motionScale: number;
	motionEasing: string;

	/** Gelişmiş bölümlerin tamamı (logo, maskot, oynatıcı, kartlar …). */
	adv: AdvState;

	viewport: "desktop" | "tablet" | "mobile";
	currentPath: string;
}

export interface Project {
	id: string;
	name: string;
	createdAt: number;
	updatedAt: number;
	doc: ThemeDoc;
	ui: EditorUiState;
	/** Kod editöründeki metin — işaretleyici blok dışına yazılanlar burada. */
	cssText: string;
	externalPath: string | null;
	/** İçe aktarıldıysa kaynağı (GitHub bağlantısı). */
	source: string | null;
	/** Ana ekrandaki kart için kullanıcının seçtiği kapak görseli (`data:` URI). */
	coverImage: string | null;
}

/** Kaydedilmemiş yeni proje için taslak. Kimliği Rust üretir. */
export type ProjectDraft = Omit<Project, "id" | "createdAt" | "updatedAt"> & {
	id: string;
	createdAt: number;
	updatedAt: number;
};

export const listProjects = () => invoke<ProjectSummary[]>("list_projects");

export const loadProject = (id: string) => invoke<Project>("load_project", { id });

export const saveProject = (project: ProjectDraft) =>
	invoke<Project>("save_project", { project });

export const deleteProject = (id: string) => invoke<void>("delete_project", { id });

export const renameProject = (id: string, name: string) =>
	invoke<Project>("rename_project", { id, name });

/**
 * Dışarıdan gelen bir CSS'i kontrollere eşler.
 *
 * `applyCssText`'ten ayrı: o, editörün kendi işaretleyici bloğuna güvenen
 * gidiş-geliş yolu; bu ise işaretleyicisi olmayan yabancı bir temayı ilk kez
 * içeri alıyor. Eşlenemeyen her kural ham CSS'te korunur.
 */
export const importCssText = (text: string, knownSelectors: string[]) =>
	invoke<ThemeDoc>("import_css_text", { text, knownSelectors });

/** Ana ekran ve ayarlar görünümünde önizleme webview'ini gizler. */
export const setPreviewVisible = (visible: boolean) =>
	invoke<void>("set_preview_visible", { visible });

/**
 * Kaydedilmiş bir `ui` bloğunun bu sürümle uyumlu olup olmadığını söyler.
 *
 * Elle düzenlenmiş ya da eski sürümden kalma bir dosya editörü bozuk bir
 * duruma sokmasın diye: uymuyorsa kontroller varsayılanlarıyla açılır, tema
 * (`doc`) yine de yüklenir — yani kullanıcı temasını asla kaybetmez.
 */
export function isUiState(value: unknown): value is EditorUiState {
	if (!value || typeof value !== "object") return false;
	const ui = value as Partial<EditorUiState>;
	return ui.version === 1 && typeof ui.adv === "object" && ui.adv !== null;
}

/** Tarihi kart altındaki kısa etikete çevirir. */
export function formatUpdated(ms: number): string {
	if (!ms) return "";
	const date = new Date(ms);
	const diff = Date.now() - ms;
	const minute = 60_000;
	const hour = 60 * minute;
	const day = 24 * hour;

	if (diff < minute) return "az önce";
	if (diff < hour) return `${Math.floor(diff / minute)} dk önce`;
	if (diff < day) return `${Math.floor(diff / hour)} sa önce`;
	if (diff < 7 * day) return `${Math.floor(diff / day)} gün önce`;

	return date.toLocaleDateString("tr-TR", { day: "numeric", month: "short", year: "numeric" });
}
