<script lang="ts">
	import { onMount, tick } from "svelte";
	import {
		Button,
		ComboBox,
		ContentDialog,
		IconButton,
		SegmentedControlButton,
		Slider,
		TextArea,
		TextBlock,
		TextBox,
		ToggleSwitch,
		Tooltip
	} from "fluent-svelte-extra";

	import { getVersion } from "@tauri-apps/api/app";
	import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
	import { openPath } from "@tauri-apps/plugin-opener";

	import AdvancedSections from "$lib/AdvancedSections.svelte";
	import AboutDialog from "$lib/AboutDialog.svelte";
	import AppSettings from "$lib/AppSettings.svelte";
	import LoginDialog from "$lib/LoginDialog.svelte";
	import UpdateDialog from "$lib/UpdateDialog.svelte";
	import { checkForUpdate, type UpdateCheck } from "$lib/updater";
	import ColorField from "$lib/ColorField.svelte";
	import ColorPicker from "$lib/ColorPicker.svelte";
	import CssEditor from "$lib/CssEditor.svelte";
	import Icon from "$lib/Icon.svelte";
	import StatusBar from "$lib/StatusBar.svelte";
	import { isEnter } from "$lib/events";
	import Launcher from "$lib/Launcher.svelte";
	import NavRail from "$lib/NavRail.svelte";
	import Section from "$lib/Section.svelte";
	import SegmentedControl from "$lib/Segmented.svelte";
	import TitleBar from "$lib/TitleBar.svelte";
	import { setPresenceEnabled, updatePresence } from "$lib/discord";
	import { installEasterEgg } from "$lib/easterEgg";
	import type { NavId } from "$lib/nav";
	import {
		DEFAULT_SETTINGS,
		applyAppTheme,
		loadSettings,
		saveSettings,
		type AppSettings as AppSettingsState
	} from "$lib/settings";
	import {
		deleteProject,
		importCssText,
		isUiState,
		listProjects,
		loadProject,
		renameProject,
		saveProject,
		setPreviewVisible,
		type EditorUiState,
		type Project,
		type ProjectSummary
	} from "$lib/projects";
	import { invoke } from "@tauri-apps/api/core";
	import {
		defaultAdv,
		buildAdvImports,
		buildAdvRules,
		buildAdvTokens,
		type AdvState
	} from "$lib/advancedBuild";
	import { SITE_DEFAULTS, seedColor, seedColors } from "$lib/defaults";
	import { STARTER_MARKER, starterTemplate } from "$lib/starter";
	import {
		applyCssText,
		applyTheme,
		debounce,
		defaultDoc,
		previewLoginState,
		previewNavigate,
		readCssFile,
		readImageDataUri,
		setPreviewBounds,
		writeCssFile,
		PRESETS,
		type ThemeDoc
	} from "$lib/theme";
	import { PARAM_ROUTES, ROUTES, SITE_ORIGIN, VIEWPORTS } from "$lib/routes";
	import {
		BADGE_SELECTOR,
		BG_BODY_SELECTOR,
		BG_TRANSPARENT_SELECTOR,
		CARD_SELECTOR,
		ENHANCED_SELECTOR,
		FONT_PRESETS,
		KNOWN_SELECTORS,
		LOGO_IMAGE_HIDE_SELECTOR,
		LOGO_IMAGE_SELECTOR,
		LOGO_ROW_SELECTOR,
		LOGO_TEXT_SELECTOR,
		MASCOT_SLOTS,
		PLAYER_BAR_SELECTOR,
		PLAYER_EPISODE_SELECTOR,
		PLAYER_SLIDER_SELECTOR,
		RELEASED_BADGE_SELECTOR,
		SCROLLBAR_SELECTOR,
		SIDEBAR_SELECTOR
	} from "$lib/advanced";
	import {
		BUTTON_TEXT_SELECTOR,
		BUTTON_TOKENS,
		DURATION_TOKENS,
		EASINGS,
		EASING_TOKEN,
		HOVER_TOKENS,
		SIDEBAR_SELECTED_TOKEN,
		fromCssColor,
		hexToRgb,
		hslToRgb,
		toCssColor
	} from "$lib/customization";

	const RAMP_NAMES = [
		"accent-light-3",
		"accent-light-2",
		"accent-light-1",
		"accent-base",
		"accent-dark-1",
		"accent-dark-2",
		"accent-dark-3"
	];

	// --- Tek state -----------------------------------------------------------
	// Görsel kontroller ve kod editörü aynı `doc` üzerinde çalışır:
	//   görsel değişiklik -> apply_theme    -> css -> editöre yazılır
	//   kod değişikliği   -> apply_css_text -> doc -> kontrollere yazılır
	// Yeni bölümlerin hepsi `tokenOverrides` / `ruleOverrides` haritalarından
	// geçtiği için bu senkron onlar için de kendiliğinden çalışıyor.
	let doc: ThemeDoc = defaultDoc();
	let cssText = "";
	let ramp: string[] = [];
	let error = "";
	let copied = false;
	let lastPushed = "";

	let editMode: "visual" | "code" = "visual";
	let editorCategory: "colors" | "shape" | "motion" | "media" | "components" | "advanced" = "colors";

	// --- Üst düzey görünüm ---------------------------------------------------
	// Uygulama artık doğrudan editöre düşmüyor: açılışta ana ekran gelir.
	// SvelteKit route'u değil basit bir durum makinesi kullanmamızın gerekçesi
	// `$lib/nav.ts`'te.
	let view: NavId = "home";

	/**
	 * Ayarlar diskten BİLDİRİMDE okunuyor, `onMount`'ta değil.
	 *
	 * Aşağıdaki "her değişimde kaydet" reaktif ifadesi bileşen ilklenirken de
	 * bir kez çalışıyor ve bu, `onMount` geri çağrılarından ÖNCE oluyor.
	 * Yükleme `onMount`'ta yapılsaydı sıra şöyle işlerdi: varsayılanlarla
	 * başla -> varsayılanları `localStorage`'a YAZ -> sonra oku. Yani her
	 * açılış kullanıcının kayıtlı ayarlarını siler ve geri varsayılanları
	 * okurdu; ayarlar hiçbir zaman kalıcı olmazdı.
	 *
	 * `ssr = false` (bkz. `+layout.ts`) olduğu için `localStorage` bu noktada
	 * her zaman var; sunucuda çalışan bir kod yolu yok.
	 */
	let settings: AppSettingsState = loadSettings();
	let appVersion = "";
	let projects: ProjectSummary[] = [];
	let projectsPath = "";

	// --- Açık proje ----------------------------------------------------------
	// `projectId` boşsa henüz kaydedilmemiş bir çalışma var demektir.
	let projectId = "";
	let projectName = "Yeni tema";
	let projectSource: string | null = null;
	/**
	 * Editör sekmesi ancak ortada bir tema varken anlamlı. Açılışta hiçbir şey
	 * açık değil, dolayısıyla sekme sönük duruyor.
	 */
	let hasOpenProject = false;
	/**
	 * En son DİSKE yazılan hâlin imzası. `projectSignature` ile karşılaştırarak
	 * kaydedilmemiş değişiklik olup olmadığını anlıyoruz — ayrı bir "dirty"
	 * bayrağı tutmaktansa böylesi güvenilir, çünkü geri alınan bir değişiklik
	 * projeyi otomatik olarak yeniden "temiz" yapıyor.
	 */
	let savedSnapshot = "";
	let projectStatus = "";

	// --- Bölüm durumları -----------------------------------------------------
	// Sayısal varsayılanlar `SITE_DEFAULTS`'tan; hepsi sitenin canlı CSS'inden
	// ölçüldü (bkz. defaults.ts). Buraya elle sayı yazmayın.
	let radiusEnabled = false;
	let controlRadius = SITE_DEFAULTS.controlRadius;
	let overlayRadius = SITE_DEFAULTS.overlayRadius;

	let hoverEnabled = false;
	let hoverColors = HOVER_TOKENS.map((s) => ({ hex: "#ffffff", alpha: s.defaultAlpha }));

	let buttonsEnabled = false;
	let buttonColors = BUTTON_TOKENS.map((s) => ({ hex: "#ffffff", alpha: s.defaultAlpha }));
	let buttonTextHex = "#ffffff";

	// --- Varsayılanlar sitenin kendi değerlerinden gelir ----------------------
	// Kontrolleri beyazla başlatmak yanıltıcıydı: kullanıcı henüz hiçbir şey
	// değiştirmemişken sanki her şey beyaza ayarlıymış gibi görünüyordu.
	// Tohumlama artık `defaults.ts` üzerinden katalogdaki gerçek --fds-*
	// varsayılanlarını okuyor ve gelişmiş bölümlerle aynı yolu paylaşıyor.

	// Tohumlama modu bilerek `doc.mode`'dan AYRI tutuluyor. Doğrudan `doc`'u
	// okusaydık şöyle bir döngü oluşurdu:
	//   hoverColors -> doc -> tokenMap -> hoverColors
	// Bu değişken mod butonlarından ve adoptDoc'tan elle güncelleniyor.
	let seedMode = "dark";

	// Bölüm KAPALIYKEN varsayılanları tazele. Açıkken dokunmuyoruz, yoksa
	// kullanıcının girdiği değerler mod değişiminde silinirdi.
	$: if (!hoverEnabled) hoverColors = seedColors(HOVER_TOKENS, seedMode, ramp);
	$: if (!buttonsEnabled) buttonColors = seedColors(BUTTON_TOKENS, seedMode, ramp);
	$: if (!buttonsEnabled) {
		buttonTextHex = seedColor("--fds-text-primary", seedMode, ramp).hex;
	}

	function setMode(mode: "system" | "light" | "dark") {
		doc.mode = mode;
	}

	let motionEnabled = false;
	let motionScale = SITE_DEFAULTS.motionScale;
	let motionEasing = EASINGS[0].value;

	// Gelişmiş bölümler. Logo dahil hepsi buradan geçiyor;
	// çıktıları aşağıda mevcut haritalara katılıyor.
	let adv: AdvState = defaultAdv(seedMode, ramp);

	$: doc.controlCornerRadius = radiusEnabled ? controlRadius : null;
	$: doc.overlayCornerRadius = radiusEnabled ? overlayRadius : null;

	/** Bölüm durumlarını token haritasına çevirir. */
	function buildTokens(
		hoverOn: boolean,
		hoverList: { hex: string; alpha: number }[],
		buttonsOn: boolean,
		buttonList: { hex: string; alpha: number }[],
		motionOn: boolean,
		scale: number,
		easing: string
	): Record<string, string> {
		const map: Record<string, string> = {};

		if (hoverOn) {
			HOVER_TOKENS.forEach((spec, i) => {
				const css = toCssColor(hoverList[i].hex, spec.alpha ? hoverList[i].alpha : 100);
				if (css) map[spec.token] = css;
			});
		}

		if (buttonsOn) {
			BUTTON_TOKENS.forEach((spec, i) => {
				const css = toCssColor(buttonList[i].hex, spec.alpha ? buttonList[i].alpha : 100);
				if (css) map[spec.token] = css;
			});
		}

		if (motionOn) {
			// Sitenin kendi süre token'larını ölçekliyoruz — yeni bir animasyon
			// sistemi kurmuyoruz. 0 = anlık, 1 = varsayılan, 2 = iki kat yavaş.
			for (const { token, base } of DURATION_TOKENS) {
				map[token] = `${Math.round(base * scale)}ms`;
			}
			map[EASING_TOKEN] = easing;
		}

		return map;
	}

	/** Token karşılığı olmayan ayarları kural haritasına çevirir. */
	function buildRules(buttonsOn: boolean, textHex: string): Record<string, string> {
		const map: Record<string, string> = {};

		if (buttonsOn) {
			const css = toCssColor(textHex, 100);
			if (css) map[BUTTON_TEXT_SELECTOR] = `color: ${css};`;
		}

		return map;
	}

	$: tokenMap = {
		...buildTokens(
			hoverEnabled,
			hoverColors,
			buttonsEnabled,
			buttonColors,
			motionEnabled,
			motionScale,
			motionEasing
		),
		...buildAdvTokens(adv)
	};
	// Gelişmiş bölümlerin çıktısı mevcut haritalara katılıyor. Ayrı bir yol
	// açmıyoruz; böylece kod editörü ve harici dosya senkronu değişmeden çalışıyor.
	$: ruleMap = { ...buildRules(buttonsEnabled, buttonTextHex), ...buildAdvRules(adv) };
	$: importList = buildAdvImports(adv);

	$: if (JSON.stringify(importList) !== JSON.stringify(doc.imports)) {
		doc.imports = importList;
	}

	// Kontrollerden en son üretilen haritalar. Karşılaştırmayı `doc` yerine
	// bunlara yapıyoruz ki kod editöründen benimsenen bir değer, kontrollerin
	// yeniden üretimiyle ezilmesin.
	let lastTokenMap = "{}";
	let lastRuleMap = "{}";
	// adoptDoc çalıştığında bir sonraki yeniden üretim "kullanıcı düzenlemesi"
	// sayılmamalı; yalnızca temel alınır.
	let adopting = false;

	$: {
		const nextTokens = JSON.stringify(tokenMap);
		const nextRules = JSON.stringify(ruleMap);

		if (adopting) {
			// Kod editöründen gelen değerleri baz al, doc'a geri yazma.
			// Kontrole çözülemeyen değerler (ör. elle yazılmış hsla) böylece
			// olduğu gibi korunuyor.
			lastTokenMap = nextTokens;
			lastRuleMap = nextRules;
			adopting = false;
		} else {
			if (nextTokens !== lastTokenMap) {
				lastTokenMap = nextTokens;
				doc.tokenOverrides = { ...doc.tokenOverrides, ...tokenMap };
			}
			if (nextRules !== lastRuleMap) {
				lastRuleMap = nextRules;
				const merged = { ...doc.ruleOverrides, ...ruleMap };
				if (!adv.logo.textOn) delete merged[LOGO_TEXT_SELECTOR];
				if (!adv.logo.imageOn) delete merged[LOGO_IMAGE_SELECTOR];
				if (!adv.bg.on) delete merged[`${BG_BODY_SELECTOR}::before`];
				doc.ruleOverrides = merged;
			}
		}
	}

	/**
	 * `forceText`: kod modunda normalde kullanıcının metnini EZMİYORUZ, ama
	 * sıfırlama tam olarak bunu yapmak zorunda — yoksa kontroller sıfırlanır,
	 * editördeki metin eski hâlinde kalır ve iki mod birbirinden ayrışır.
	 */
	async function push(forceText = false) {
		try {
			const result = await applyTheme(doc);
			lastPushed = JSON.stringify(doc);
			ramp = result.ramp;
			// Kullanıcı o an kod editöründe yazıyorsa metnini ezme.
			if (editMode === "visual" || forceText) cssText = result.css;
			error = "";
		} catch (e) {
			error = String(e);
		}
	}

	const pushDebounced = debounce(push, 16);

	$: {
		const snapshot = JSON.stringify(doc);
		if (snapshot !== lastPushed) {
			lastPushed = snapshot;
			pushDebounced();
		}
	}

	/** Kod tarafından çözümlenen dokümanı kontrollere geri yansıtır. */
	function adoptDoc(next: ThemeDoc) {
		adopting = true;
		if (next.accent) {
			accentH = next.accent[0];
			accentS = next.accent[1];
			accentL = next.accent[2];
		}
		radiusEnabled = next.controlCornerRadius !== null || next.overlayCornerRadius !== null;
		if (next.controlCornerRadius !== null) controlRadius = next.controlCornerRadius;
		if (next.overlayCornerRadius !== null) overlayRadius = next.overlayCornerRadius;

		// Hover / buton renkleri: CSS değerini kontrole geri çöz.
		hoverEnabled = HOVER_TOKENS.some((s) => next.tokenOverrides[s.token] !== undefined);
		HOVER_TOKENS.forEach((spec, i) => {
			const parsed = fromCssColor(next.tokenOverrides[spec.token] ?? "");
			if (parsed) hoverColors[i] = { hex: parsed.hex, alpha: parsed.alpha };
		});
		hoverColors = hoverColors;

		buttonsEnabled =
			BUTTON_TOKENS.some((s) => next.tokenOverrides[s.token] !== undefined) ||
			next.ruleOverrides[BUTTON_TEXT_SELECTOR] !== undefined;
		BUTTON_TOKENS.forEach((spec, i) => {
			const parsed = fromCssColor(next.tokenOverrides[spec.token] ?? "");
			if (parsed) buttonColors[i] = { hex: parsed.hex, alpha: parsed.alpha };
		});
		buttonColors = buttonColors;

		const textRule = next.ruleOverrides[BUTTON_TEXT_SELECTOR];
		if (textRule) {
			const parsed = fromCssColor(textRule.replace(/^\s*color\s*:\s*/, "").replace(/;\s*$/, ""));
			if (parsed) buttonTextHex = parsed.hex;
		}
		// Animasyon: ölçeği "normal" süreden geri hesapla.
		motionEnabled = DURATION_TOKENS.some((d) => next.tokenOverrides[d.token] !== undefined);
		const normal = next.tokenOverrides["--fds-control-normal-duration"];
		if (normal) {
			const ms = Number(normal.replace(/ms\s*$/, ""));
			if (Number.isFinite(ms)) motionScale = Number((ms / 250).toFixed(2));
		}
		if (next.tokenOverrides[EASING_TOKEN]) motionEasing = next.tokenOverrides[EASING_TOKEN];

		// --- Gelişmiş bölümler: tüm gelişmiş ayarları ve görselleri çöz -----------
		const resolveUrl = (rule: string | undefined, tokens: Record<string, string>): string => {
			if (!rule) return "";
			// 1. Doğrudan url(...) eşleşmesi
			const direct = rule.match(/url\(\s*["']?([^"'\s)]+)["']?\s*\)/i);
			if (direct && direct[1] && !direct[1].startsWith("var(")) {
				return direct[1];
			}
			// 2. Kural içindeki var(--değişken) başvurusu
			const varMatches = Array.from(rule.matchAll(/var\(\s*(--[a-zA-Z0-9_-]+)/gi));
			for (const v of varMatches) {
				const varName = v[1];
				const varValue = tokens[varName];
				if (varValue) {
					const innerMatch = varValue.match(/url\(\s*["']?([^"'\s)]+)["']?\s*\)/i);
					if (innerMatch && innerMatch[1]) {
						return innerMatch[1];
					}
				}
			}
			return "";
		};

		// Logo görselini kural, gizleme seçicisi veya öncelikli değişkenlerden çöz
		let logoImage = resolveUrl(next.ruleOverrides[LOGO_IMAGE_SELECTOR], next.tokenOverrides);
		if (!logoImage) {
			logoImage = resolveUrl(next.ruleOverrides[LOGO_IMAGE_HIDE_SELECTOR], next.tokenOverrides);
		}
		if (!logoImage) {
			const priorityVars = [
				"--url-logo",
				"--logo-url",
				"--logo-image",
				"--url-icon",
				"--icon-url",
				"--logo-src",
				"--logo"
			];
			for (const pVar of priorityVars) {
				const val = next.tokenOverrides[pVar];
				if (val) {
					const m = val.match(/url\(\s*["']?([^"'\s)]+)["']?\s*\)/i);
					if (m && m[1]) {
						logoImage = m[1];
						break;
					}
				}
			}
		}
		adv.logo.dataUri = logoImage;
		adv.logo.imageOn = logoImage !== "";

		const logoTextMatch = next.ruleOverrides[LOGO_TEXT_SELECTOR]?.match(
			/content\s*:\s*['"]((?:[^'"\\]|\\.)*)['"]/i
		);
		const logoText = logoTextMatch ? logoTextMatch[1].replace(/\\(["'\\])/g, "$1") : "";
		adv.logo.text = logoText;
		adv.logo.textOn = logoText !== "";

		const logoRowRule = next.ruleOverrides[LOGO_ROW_SELECTOR] ?? "";
		const gapMatch = logoRowRule.match(/gap\s*:\s*(\d+)px/i);
		if (gapMatch) adv.logo.gap = Number(gapMatch[1]);

		const logoImgRule = next.ruleOverrides[LOGO_IMAGE_SELECTOR] ?? "";
		const sizeMatch = logoImgRule.match(/width\s*:\s*(\d+)px/i);
		if (sizeMatch) adv.logo.size = Number(sizeMatch[1]);

		// Arkaplan görselini body::before, body::after, banner veya değişkenlerden çöz
		let bgImage = resolveUrl(next.ruleOverrides[`${BG_BODY_SELECTOR}::before`], next.tokenOverrides);
		if (!bgImage) {
			bgImage =
				resolveUrl(next.ruleOverrides[`${BG_BODY_SELECTOR}::after`], next.tokenOverrides) ||
				resolveUrl(next.ruleOverrides["body::before"], next.tokenOverrides) ||
				resolveUrl(next.ruleOverrides["body::after"], next.tokenOverrides) ||
				resolveUrl(next.ruleOverrides[BG_BODY_SELECTOR], next.tokenOverrides) ||
				resolveUrl(next.ruleOverrides[BG_TRANSPARENT_SELECTOR], next.tokenOverrides) ||
				resolveUrl(
					next.ruleOverrides[
						".scene-inner-content:has(.new-playlist)>.banner.gradient-scene"
					],
					next.tokenOverrides
				) ||
				resolveUrl(next.ruleOverrides[".banner.gradient-scene"], next.tokenOverrides);
		}
		if (!bgImage) {
			const priorityVars = [
				"--url-bg",
				"--bg-url",
				"--background-url",
				"--bg-image",
				"--url-background",
				"--background-image",
				"--bg-page"
			];
			for (const pVar of priorityVars) {
				const val = next.tokenOverrides[pVar];
				if (val) {
					const m = val.match(/url\(\s*["']?([^"'\s)]+)["']?\s*\)/i);
					if (m && m[1]) {
						bgImage = m[1];
						break;
					}
				}
			}
		}
		adv.bg.dataUri = bgImage;
		adv.bg.on = bgImage !== "";

		for (const slot of MASCOT_SLOTS) {
			const image = resolveUrl(next.ruleOverrides[slot.selector], next.tokenOverrides);
			if (image) adv.mascot.images[slot.id] = image;
			else delete adv.mascot.images[slot.id];
		}

		// Kartlar
		const cardRule = next.ruleOverrides[CARD_SELECTOR];
		if (cardRule || next.tokenOverrides["--fds-card-background-default"]) {
			adv.cards.on = true;
			if (cardRule) {
				const rMatch = cardRule.match(/border-radius\s*:\s*(\d+)px/i);
				if (rMatch) adv.cards.radius = Number(rMatch[1]);
				const bMatch = cardRule.match(/border-width\s*:\s*(\d+)px/i);
				if (bMatch) adv.cards.borderWidth = Number(bMatch[1]);
			}
		}

		// Kenar çubuğu
		const sidebarRule = next.ruleOverrides[SIDEBAR_SELECTOR];
		if (sidebarRule || next.tokenOverrides[SIDEBAR_SELECTED_TOKEN.token]) {
			adv.sidebar.on = true;
			if (sidebarRule) {
				const wMatch = sidebarRule.match(/width\s*:\s*([\d.]+)rem/i);
				if (wMatch) adv.sidebar.width = Number(wMatch[1]);
			}
		}

		// Rozetler
		if (
			next.ruleOverrides[BADGE_SELECTOR] ||
			next.ruleOverrides[RELEASED_BADGE_SELECTOR] ||
			next.ruleOverrides[ENHANCED_SELECTOR]
		) {
			adv.badges.on = true;
		}

		// Kaydırma çubuğu
		if (next.ruleOverrides[SCROLLBAR_SELECTOR]) {
			adv.scrollbar.on = true;
			const sbRule = next.ruleOverrides[SCROLLBAR_SELECTOR];
			const szMatch = sbRule.match(/--os-size\s*:\s*(\d+)px/i);
			if (szMatch) adv.scrollbar.size = Number(szMatch[1]);
		}

		// Oynatıcı
		if (
			next.ruleOverrides[PLAYER_SLIDER_SELECTOR] ||
			next.ruleOverrides[PLAYER_BAR_SELECTOR] ||
			next.ruleOverrides[PLAYER_EPISODE_SELECTOR]
		) {
			adv.player.on = true;
		}

		// Tipografi / Yazı Tipi
		if (
			next.imports.length > 0 ||
			next.tokenOverrides["--fds-font-family-text"] ||
			next.tokenOverrides["--fds-font-family-display"]
		) {
			adv.typo.on = true;
			if (next.imports.length > 0) {
				const imp = next.imports[0];
				const foundIndex = FONT_PRESETS.findIndex((p) => p.importUrl === imp);
				if (foundIndex > 0) {
					adv.typo.preset = foundIndex;
				} else {
					adv.typo.preset = -1;
					adv.typo.custom = imp;
				}
			}
		}

		adv = adv;

		seedMode = next.mode;
		doc = next;
	}

	const pushCode = debounce(async (text: string) => {
		try {
			adoptDoc(await applyCssText(text, KNOWN_SELECTORS));
			error = "";
		} catch (e) {
			error = String(e);
		}
	}, 250);

	function onCodeChange(event: CustomEvent<string>) {
		cssText = event.detail;
		if (externalPath) externalDirty = true;
		pushCode(cssText);
	}

	// Görsel moda dönerken metni kanonik hâline getir.
	$: if (editMode === "visual") push();

	// --- Önizleme yerleşimi + viewport simülasyonu ---------------------------
	//
	// Görünüm seçicisi neden bir ŞERİT, "gerçek" bir overlay değil:
	// önizleme, ana pencereye `Window::add_child` ile eklenmiş NATIVE bir
	// webview. Native child webview her zaman host sayfanın içeriğinin üstüne
	// çizilir — üstüne konan bir HTML elemanı görünmez olur. Bu yüzden
	// `previewSlot` webview'in oturacağı alanı temsil ediyor ve seçici onun
	// ÜSTÜNDEKİ şeride yerleşiyor. `syncBounds` zaten `previewSlot`'u ölçtüğü
	// için webview kendiliğinden şeridin altında kalıyor.
	let previewSlot: HTMLDivElement;
	let viewport: "desktop" | "tablet" | "mobile" = "desktop";

	// Her ölçüm isteğine sıra numarası veriyoruz. Viewport değişimi, pencere
	// resize'ı ve ResizeObserver aynı anda tetiklenebiliyor; sıra numarası
	// sayesinde eskimiş bir ölçüm, yenisinin üstüne yazamıyor.
	let boundsSeq = 0;

	function syncBounds() {
		if (!previewSlot) return;
		const seq = ++boundsSeq;

		// Ölçümü bir sonraki kareye erteliyoruz: Svelte'in reaktif akışı DOM
		// yerleşimi tazelenmeden çalışabiliyor ve eski genişlik okunabiliyor.
		requestAnimationFrame(() => {
			if (seq !== boundsSeq || !previewSlot) return;

			const rect = previewSlot.getBoundingClientRect();
			const target = VIEWPORTS.find((v) => v.id === viewport)?.width ?? null;
			// Webview'i gerçekten daraltıyoruz: sitenin kendi medya sorguları ve
			// window.innerWidth kontrolleri böylece gerçekten tetikleniyor.
			const width = target === null ? rect.width : Math.min(target, rect.width);
			const x = rect.left + (rect.width - width) / 2;
			setPreviewBounds(x, rect.top, width, rect.height).catch((e) => (error = String(e)));
		});
	}

	$: viewport, syncBounds();

	// --- Sayfa gezinmesi -----------------------------------------------------
	let currentPath = "/";
	let customPath = "";

	const routeItems = ROUTES.map((r) => ({
		name: r.auth ? `${r.name} (giriş gerekir)` : r.name,
		value: r.path
	}));

	function go(path: string) {
		const clean = path.trim();
		if (!clean) return;
		currentPath = clean.startsWith("/") ? clean : `/${clean}`;
		previewNavigate(SITE_ORIGIN + currentPath).catch((e) => (error = String(e)));
	}

	function onRouteSelect(event: CustomEvent<{ value: string }>) {
		go(event.detail.value);
	}

	// --- Görsel seçici (Tauri dosya sistemi) --------------------------------
	async function pickImage(): Promise<string | null> {
		try {
			const selected = await openDialog({
				multiple: false,
				filters: [{ name: "Görsel", extensions: ["png", "svg", "jpg", "jpeg", "webp", "gif", "avif"] }]
			});
			if (typeof selected !== "string") return null;
			return await readImageDataUri(selected);
		} catch (e) {
			error = String(e);
			return null;
		}
	}

	// --- Harici .css dosyası -------------------------------------------------
	// Kod editörü ya uygulamanın kendi taslağını ya da diskteki bir dosyayı
	// düzenler. İkisinde de aynı `applyCssText` yolundan geçildiği için
	// kontrol ↔ kod senkronu bozulmuyor.
	let externalPath: string | null = null;
	let externalDirty = false;
	let fileStatus = "";

	async function openExternal() {
		try {
			const selected = await openDialog({
				multiple: false,
				filters: [{ name: "CSS", extensions: ["css"] }]
			});
			if (typeof selected !== "string") return;
			const contents = await readCssFile(selected);
			externalPath = selected;
			externalDirty = false;
			editMode = "code";
			cssText = contents;
			adoptDoc(await importCssText(contents, KNOWN_SELECTORS));
			fileStatus = `${selected} açıldı`;
			error = "";
		} catch (e) {
			error = String(e);
		}
	}

	async function saveExternal(saveAs = false) {
		try {
			let target = externalPath;
			if (saveAs || !target) {
				const chosen = await saveDialog({
					defaultPath: target ?? "tema.css",
					filters: [{ name: "CSS", extensions: ["css"] }]
				});
				if (typeof chosen !== "string") return;
				target = chosen;
			}
			await writeCssFile(target, cssText);
			externalPath = target;
			externalDirty = false;
			fileStatus = `${target} kaydedildi`;
			error = "";
		} catch (e) {
			error = String(e);
		}
	}

	function closeExternal() {
		externalPath = null;
		externalDirty = false;
		fileStatus = "";
	}

	/** Sitenin mevcut token değerlerinden düzenlenebilir bir şablon yükler. */
	function loadStarter() {
		if (cssText.includes(STARTER_MARKER)) {
			fileStatus = "Şablon zaten yüklü.";
			return;
		}
		const next = `${cssText.trimEnd()}\n\n${starterTemplate()}`;
		cssText = next;
		pushCode(next);
		fileStatus = "Site CSS şablonu eklendi.";
	}

	async function copyCss() {
		try {
			await navigator.clipboard.writeText(cssText);
			copied = true;
			setTimeout(() => (copied = false), 1800);
		} catch (e) {
			error = String(e);
		}
	}

	function usePreset(hsl: [number, number, number]) {
		accentH = hsl[0];
		accentS = hsl[1];
		accentL = hsl[2];
		doc = { ...doc, accent: [...hsl] as [number, number, number] };
	}

	// --- Vurgu rengi: palet ve HSL kaydırıcıları TEK state paylaşır ---------
	// `doc.accent` (HSL) TEK gerçek kaynak. `accentHex` ondan SALT türetilir
	// (aşağıdaki `$: accentHex = ...`) — ayrı, elle senkronlanan bir state
	// DEĞİL. Önceki sürümde `accentHex` kendi başına bir `let` idi ve
	// "uygulanmış değer" (`accentApplied`) bekçisiyle elle senkronlanıyordu;
	// bu ikili state, paletten (renk alanı/hue şeridi/hex/RGB) seçilen rengin
	// H/S/L kaydırıcılarına hiç yansımamasına yol açan hataydı — türetme tek
	// yönlü olduğundan `doc` değişse bile kaydırıcıların okuduğu değer
	// (`doc.accent`) güncellenmiyor, yalnızca palet tarafı (`accentHex`)
	// güncelleniyordu. Artık ikisi de doğrudan `doc.accent`'i okuyup
	// yazdığından hangi kontrolden değişirse değişsin diğerleri anında
	// güncellenir.

	const toHexString = (rgb: number[]) =>
		"#" + rgb.map((n) => Math.round(n).toString(16).padStart(2, "0")).join("");

	function hslToHex(hsl: [number, number, number]): string {
		return toHexString(hslToRgb(hsl[0], hsl[1], hsl[2]));
	}

	function hexToHsl(hex: string): [number, number, number] | null {
		const rgb = hexToRgb(hex);
		if (!rgb) return null;
		const [r, g, b] = rgb.map((n) => n / 255);
		const max = Math.max(r, g, b);
		const min = Math.min(r, g, b);
		const d = max - min;
		const l = (max + min) / 2;
		let h = 0;
		if (d !== 0) {
			if (max === r) h = ((g - b) / d) % 6;
			else if (max === g) h = (b - r) / d + 2;
			else h = (r - g) / d + 4;
			h *= 60;
			if (h < 0) h += 360;
		}
		const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1));
		return [
			Math.round(h * 10) / 10,
			Math.round(s * 1000) / 10,
			Math.round(l * 1000) / 10
		];
	}

	let accentH = 206;
	let accentS = 100;
	let accentL = 42;

	function extractSliderVal(e: any): number {
		if (typeof e?.detail === "number") return e.detail;
		if (Array.isArray(e?.detail) && typeof e.detail[1] === "number") return e.detail[1];
		if (typeof e?.target?.value !== "undefined") return Number(e.target.value);
		return 0;
	}

	function updateAccentFromSlider(h: number, s: number, l: number) {
		accentH = h;
		accentS = s;
		accentL = l;
		doc = { ...doc, accent: [h, s, l] };
	}

	$: accentHex = hslToHex(doc.accent);

	function onAccentPick(next: string) {
		const hsl = hexToHsl(next);
		if (!hsl) return;
		accentH = hsl[0];
		accentS = hsl[1];
		accentL = hsl[2];
		doc = { ...doc, accent: [hsl[0], hsl[1], hsl[2]] };
	}

	// --- Sıfırlama -----------------------------------------------------------
	//
	// Sıfırlamanın üç modda da tutarlı olması için tek bir kural var:
	// **her sıfırlama `doc`'u değiştirir, sonra CSS'i yeniden ürettirir ve
	// metni her modda senkronlar.** Kontroller -> doc -> CSS zinciri zaten tek
	// yol olduğu için ham CSS ve harici dosya modları kendiliğinden takip eder;
	// tek ek iş, kod modunda metni de tazelemek (`push(true)`).

	async function afterReset() {
		// Kontrol değişiklikleri `tokenMap`/`ruleMap` üzerinden `doc`'a reaktif
		// olarak akıyor; CSS'i üretmeden önce o turun bitmesini bekliyoruz.
		await tick();
		await push(true);
		// Harici dosya açıksa diskteki hâli artık eskimiştir.
		if (externalPath) externalDirty = true;
	}

	type BuiltinSection = "appearance" | "accent" | "radius" | "hover" | "motion" | "buttons" | "raw";

	function resetSection(section: BuiltinSection) {
		const base = defaultDoc();
		switch (section) {
			case "appearance":
				setMode(base.mode);
				doc = { ...doc, mode: base.mode };
				break;
			case "accent":
				accentH = base.accent[0];
				accentS = base.accent[1];
				accentL = base.accent[2];
				doc = { ...doc, accent: [...base.accent] as [number, number, number] };
				break;
			case "radius":
				radiusEnabled = false;
				controlRadius = SITE_DEFAULTS.controlRadius;
				overlayRadius = SITE_DEFAULTS.overlayRadius;
				break;
			case "hover":
				hoverEnabled = false;
				hoverColors = seedColors(HOVER_TOKENS, seedMode, ramp);
				break;
			case "motion":
				motionEnabled = false;
				motionScale = SITE_DEFAULTS.motionScale;
				motionEasing = EASINGS[0].value;
				break;
			case "buttons":
				buttonsEnabled = false;
				buttonColors = seedColors(BUTTON_TOKENS, seedMode, ramp);
				buttonTextHex = seedColor("--fds-text-primary", seedMode, ramp).hex;
				break;
			case "raw":
				doc = { ...doc, rawCss: "" };
				break;
		}
		afterReset();
	}

	/** "Tümünü sıfırla" onay diyaloğu. */
	let confirmResetAll = false;
	let aboutDialogOpen = false;

	// --- Güncelleyici ----------------------------------------------------
	//
	// Kontrol mantığı burada, gösterim `UpdateDialog.svelte`'de — ikisi ayrı
	// çünkü Ayarlar sayfasındaki "Şimdi kontrol et" düğmesi de AYNI kontrolü
	// tetikleyebilmeli, sonucu ise hep buradaki tek `updateAvailable`/
	// `updateDialogOpen` çiftine yazılıyor.
	let updateAvailable: UpdateCheck | null = null;
	let updateDialogOpen = false;
	/**
	 * Ayarlar sayfasındaki elle kontrol düğmesinin durumu.
	 *
	 * `channel-empty` ayrı bir durum: "güncelsin" ile "bu kanaldan hiç sürüm
	 * çıkmamış" kullanıcı için aynı şey değil. Stable kanalı seçen biri, o
	 * kanalda henüz yayın yoksa bunu açıkça görmeli — sessizce "güncel"
	 * demek, ön-sürüm kullanan birine yanlış bilgi verirdi.
	 */
	let updateCheckStatus:
		| "idle"
		| "checking"
		| "up-to-date"
		| "channel-empty"
		| "error" = "idle";
	let updateCheckError = "";
	/** Son kontrolün kanal adı ("Stable" / "Beta" / "Alpha"). */
	let updateChannelLabel = "";

	/**
	 * @param manual Ayarlar sayfasından elle mi tetiklendi? Otomatik açılış
	 * kontrolü sessizdir (hata olursa kullanıcıyı rahatsız etmez); elle
	 * tetiklenen kontrol ise "güncel" ya da hata durumunu görünür kılmalı —
	 * aksi hâlde düğmeye basan kullanıcı hiçbir geri bildirim almaz.
	 */
	async function runUpdateCheck(manual: boolean) {
		if (manual) {
			updateCheckStatus = "checking";
			updateCheckError = "";
		}
		try {
			// Elle tetiklenen kontrolde önbellek atlanıyor: düğmeye basan
			// kullanıcı beş dakika boyunca aynı yanıtı almamalı.
			const result = await checkForUpdate(settings.updateChannel, manual);
			updateChannelLabel = result.channelLabel;

			if (result.channelEmpty) {
				updateAvailable = null;
				if (manual) {
					updateCheckStatus = "channel-empty";
					setTimeout(() => {
						if (updateCheckStatus === "channel-empty") updateCheckStatus = "idle";
					}, 6000);
				}
			} else if (result.available && result.version !== settings.updateSkipVersion) {
				updateAvailable = result;
				updateDialogOpen = true;
				updateCheckStatus = "idle";
			} else if (manual) {
				updateCheckStatus = "up-to-date";
				setTimeout(() => {
					if (updateCheckStatus === "up-to-date") updateCheckStatus = "idle";
				}, 4000);
			}
		} catch (e) {
			if (manual) {
				updateCheckStatus = "error";
				updateCheckError = String(e);
			}
			// Otomatik kontrolde sessizce yutuluyor: ağ yoksa ya da GitHub
			// erişilemezse editör kullanılamaz hâle gelmemeli.
		}
	}

	/** Açılıştan birkaç saniye sonra, arka planda, sessizce. */
	function checkForUpdatesOnStartup() {
		if (!settings.updateAutoCheck) return;
		setTimeout(() => runUpdateCheck(false), 3500);
	}

	/** "Daha Sonra Hatırlat" — bu sürümü kalıcı olarak atlar. */
	function skipUpdate() {
		if (updateAvailable?.version) {
			settings = { ...settings, updateSkipVersion: updateAvailable.version };
		}
		updateDialogOpen = false;
	}

	/**
	 * Kanal değiştiğinde durumu hemen tazeler.
	 *
	 * Atlanan sürüm de sıfırlanıyor: `updateSkipVersion` kanal bilmiyor ve
	 * beta'da ertelenen bir sürüm numarası, Stable kanalda çıkan AYNI numaralı
	 * sürümü de sessizce gizlerdi.
	 */
	function onChannelChange() {
		settings = { ...settings, updateSkipVersion: "" };
		void runUpdateCheck(true);
	}

	/**
	 * Her şeyi varsayılana döndürür — ama CSS'i YENİDEN ÜRETMEZ.
	 *
	 * Yeniden üretim (`afterReset`) bilerek ayrı: yeni proje ve içe aktarma
	 * akışları sıfırlamanın hemen ardından kendi belgelerini yüklüyor. İkisi
	 * aynı fonksiyonda olsaydı sıfırlamanın asenkron `push`'u, sonradan
	 * yüklenen belgenin CSS'ini ezebilirdi — sonuç, kullanıcının içe aktardığı
	 * temanın bir an görünüp kaybolması olurdu.
	 */
	function resetState() {
		// Doküman: accent, mod, yarıçaplar, import'lar, tüm ezmeler ve ham CSS.
		doc = defaultDoc();
		seedMode = doc.mode;

		// Kontrol durumları.
		radiusEnabled = false;
		controlRadius = SITE_DEFAULTS.controlRadius;
		overlayRadius = SITE_DEFAULTS.overlayRadius;
		hoverEnabled = false;
		buttonsEnabled = false;
		motionEnabled = false;
		motionScale = SITE_DEFAULTS.motionScale;
		motionEasing = EASINGS[0].value;
		hoverColors = seedColors(HOVER_TOKENS, seedMode, ramp);
		buttonColors = seedColors(BUTTON_TOKENS, seedMode, ramp);
		buttonTextHex = seedColor("--fds-text-primary", seedMode, ramp).hex;

		// Gelişmiş bölümlerin tamamı (logo, maskot, kenar çubuğu, oynatıcı …).
		adv = defaultAdv(seedMode, ramp);

		// Karşılaştırma tamponlarını da temizle; aksi hâlde bir sonraki reaktif
		// tur "değişiklik yok" sanıp boş haritaları doc'a yazmayabilir.
		lastTokenMap = "{}";
		lastRuleMap = "{}";
	}

	async function resetAll() {
		confirmResetAll = false;
		resetState();

		fileStatus = externalPath
			? "Tema varsayılana döndürüldü. Diske yazmak için Kaydet'e basın."
			: "Tema varsayılana döndürüldü.";

		await afterReset();
	}

	// --- Proje: durum yakalama ve geri yükleme -------------------------------
	//
	// `doc` temanın kendisi, `uiState` ise kontrollerin durumu. İkisini AYRI
	// saklamak zorundayız: bir bölümün KAPALI olması ile açık ama varsayılan
	// değerde olması aynı CSS'i üretiyor. Yalnız `doc` kaydedilseydi proje
	// yeniden açıldığında kullanıcı bıraktığı yerde değil, "her şey kapalı"
	// hâlinde bulurdu kendini.
	//
	// Değişkenleri burada tek tek saymak şart: Svelte reaktif bloğun
	// bağımlılıklarını METİNDEN çıkarıyor, bir yardımcı fonksiyonun içine
	// gizlenselerdi `adv` değiştiğinde imza tazelenmezdi.
	$: uiState = {
		version: 1,
		editMode,
		seedMode,
		radiusEnabled,
		controlRadius,
		overlayRadius,
		hoverEnabled,
		hoverColors,
		buttonsEnabled,
		buttonColors,
		buttonTextHex,
		motionEnabled,
		motionScale,
		motionEasing,
		adv,
		viewport,
		currentPath
	} as EditorUiState;

	$: docSignature = { ...doc, mode: undefined };
	$: uiSignature = {
		...uiState,
		seedMode: undefined,
		viewport: undefined,
		currentPath: undefined,
		editMode: undefined
	};
	$: projectSignature = JSON.stringify({
		doc: docSignature,
		ui: uiSignature,
		cssText,
		externalPath
	});
	$: dirty = projectSignature !== savedSnapshot;

	/**
	 * Başlık çubuğunun orta bölgesinde duran bağlam metni.
	 * Editörde açık projeyi, diğer görünümlerde görünümün adını gösteriyor.
	 */
	$: titleContext =
		view === "editor"
			? `${projectName}${dirty ? " •" : ""}`
			: view === "settings"
				? "Ayarlar"
				: "Ana Sayfa";

	function restoreUi(ui: EditorUiState) {
		editMode = ui.editMode ?? settings.defaultEditMode;
		seedMode = ui.seedMode ?? "dark";

		radiusEnabled = ui.radiusEnabled ?? false;
		controlRadius = ui.controlRadius ?? SITE_DEFAULTS.controlRadius;
		overlayRadius = ui.overlayRadius ?? SITE_DEFAULTS.overlayRadius;

		hoverEnabled = ui.hoverEnabled ?? false;
		if (Array.isArray(ui.hoverColors) && ui.hoverColors.length === HOVER_TOKENS.length) {
			hoverColors = ui.hoverColors;
		}

		buttonsEnabled = ui.buttonsEnabled ?? false;
		if (Array.isArray(ui.buttonColors) && ui.buttonColors.length === BUTTON_TOKENS.length) {
			buttonColors = ui.buttonColors;
		}
		buttonTextHex = ui.buttonTextHex ?? buttonTextHex;

		motionEnabled = ui.motionEnabled ?? false;
		motionScale = ui.motionScale ?? SITE_DEFAULTS.motionScale;
		motionEasing = ui.motionEasing ?? EASINGS[0].value;

		if (ui.adv) adv = ui.adv;

		viewport = ui.viewport ?? settings.defaultViewport;
		currentPath = ui.currentPath ?? settings.defaultPreviewPath;
	}

	// --- Proje: açma, oluşturma, kaydetme ------------------------------------

	async function refreshProjects() {
		try {
			projects = await listProjects();
		} catch (e) {
			error = String(e);
		}
	}

	/** Editörü, kaydedilmiş bir projenin tam durumuna getirir. */
	async function applyProject(project: Project) {
		projectId = project.id;
		projectName = project.name;
		projectSource = project.source;
		externalPath = project.externalPath;
		externalDirty = false;
		cssText = project.cssText;

		// Kontrol durumu bozuksa (elle düzenlenmiş ya da eski sürüm) temayı yine
		// de yüklüyoruz — kullanıcı çalışmasını asla kaybetmemeli.
		if (isUiState(project.ui)) {
			restoreUi(project.ui);
		} else {
			projectStatus = "Kontrol durumu okunamadı; tema yüklendi, kontroller varsayılana döndü.";
		}

		// `adoptDoc`'takiyle aynı gerekçe: kaydedilen belgeyi TEMEL al, bir
		// sonraki reaktif turda kontrollerden yeniden üretip ezme.
		adopting = true;
		doc = project.doc;
		lastPushed = JSON.stringify(project.doc);

		await tick();
		await push(true);
		go(currentPath);
	}

	async function openProject(id: string) {
		try {
			const project = await loadProject(id);
			await applyProject(project);
			hasOpenProject = true;
			view = "editor";

			// İmzayı bir tur SONRA alıyoruz: `applyProject` içindeki yeniden
			// üretim kontrollerden bir tur daha reaktif akış tetikleyebiliyor ve
			// erken alınan imza projeyi hemen "değişmiş" gösterirdi.
			await tick();
			savedSnapshot = projectSignature;
			error = "";
		} catch (e) {
			error = String(e);
		}
	}

	/** Sıfırdan yeni bir tema. Henüz diskte yok; ilk kaydetmede oluşur. */
	async function createProject() {
		resetState();
		projectId = "";
		projectName = "Yeni tema";
		projectSource = null;
		externalPath = null;
		externalDirty = false;
		fileStatus = "";
		projectStatus = "";

		editMode = settings.defaultEditMode;
		viewport = settings.defaultViewport;
		currentPath = settings.defaultPreviewPath;

		// CSS'i burada üretiyoruz ki `cssText` yerine oturmuş olsun; aksi hâlde
		// aşağıdaki imza henüz boş metinle alınır ve proje daha ilk karede
		// "kaydedilmemiş değişiklik" gibi görünürdü.
		await afterReset();
		go(currentPath);

		hasOpenProject = true;
		view = "editor";
		await tick();
		// Dokunulmamış yeni bir tema "kaydedilmemiş değişiklik" sayılmasın.
		savedSnapshot = projectSignature;
	}

	let namingOpen = false;
	let nameValue = "";
	/** Ad sorulduktan sonra yapılacak iş (ör. ana ekrana dönmek). */
	let afterSave: (() => void) | null = null;

	function askName(next: (() => void) | null = null) {
		nameValue = projectName;
		afterSave = next;
		namingOpen = true;
	}

	async function confirmName() {
		const name = nameValue.trim();
		if (!name) return;
		projectName = name;
		namingOpen = false;
		await persistProject();
		const next = afterSave;
		afterSave = null;
		next?.();
	}

	/** Projeyi diske yazar. Ad yoksa çağıran taraf önce `askName` demeli. */
	async function persistProject() {
		try {
			const saved = await saveProject({
				id: projectId,
				name: projectName,
				createdAt: 0,
				updatedAt: 0,
				doc,
				ui: uiState,
				cssText,
				externalPath,
				source: projectSource
			});
			projectId = saved.id;
			projectName = saved.name;
			savedSnapshot = projectSignature;
			projectStatus = `"${saved.name}" kaydedildi.`;
			error = "";
			await refreshProjects();
		} catch (e) {
			error = String(e);
		}
	}

	/** Editördeki "Projeyi kaydet" düğmesi. */
	function saveCurrentProject() {
		if (!projectId) askName();
		else persistProject();
	}

	// --- Görünümler arası geçiş ---------------------------------------------

	let confirmLeave = false;
	let pendingView: NavId = "home";

	/**
	 * Editörden ayrılırken kaydedilmemiş işi korur.
	 *
	 * Üç yol var ve hangisinin seçileceği ayarlara bağlı:
	 *   · otomatik kaydet açık + proje diskte  -> sessizce kaydet, geç
	 *   · otomatik kaydet açık + yeni proje    -> ad sor (adsız kaydedilemez)
	 *   · otomatik kaydet kapalı               -> ne yapılacağını sor
	 */
	async function navigate(next: NavId) {
		if (next === view) return;

		if (next === "editor" && !hasOpenProject) {
			if (projects.length > 0) {
				await openProject(projects[0].id);
			} else {
				await createProject();
			}
		}

		if (view === "editor" && dirty) {
			// Tek sessiz yol: proje zaten diskte VE otomatik kaydetme açık.
			// Ancak o zaman kullanıcıya sormadan kaydedip çıkabiliriz.
			if (settings.autoSaveOnLeave && projectId) {
				persistProject().then(() => (view = next));
				return;
			}

			// Diğer her durumda soruyoruz. Henüz kaydedilmemiş bir projede
			// doğrudan ad kutusunu açmak yanıltıcıydı: kullanıcı neden ad
			// sorulduğunu anlamadan bir diyalogla karşılaşıyordu. Önce ne
			// yapmak istediğini soruyoruz; "Kaydet" derse ad kutusu geliyor.
			pendingView = next;
			confirmLeave = true;
			return;
		}

		view = next;
	}

	function leaveWithoutSaving() {
		confirmLeave = false;
		savedSnapshot = projectSignature; // bir daha sormasın
		view = pendingView;
	}

	function saveAndLeave() {
		confirmLeave = false;
		if (projectId) persistProject().then(() => (view = pendingView));
		else askName(() => (view = pendingView));
	}

	// --- Ana ekran eylemleri -------------------------------------------------

	async function handleRename(id: string, name: string) {
		try {
			await renameProject(id, name);
			if (id === projectId) projectName = name;
			await refreshProjects();
		} catch (e) {
			error = String(e);
		}
	}

	async function handleDelete(id: string) {
		try {
			await deleteProject(id);
			// Açık proje silindiyse artık diskte karşılığı yok; kaydedilmemiş
			// çalışmaya dönüştürüyoruz ki bir sonraki kaydetme yeni dosya açsın.
			if (id === projectId) {
				projectId = "";
				savedSnapshot = "";
			}
			await refreshProjects();
		} catch (e) {
			error = String(e);
		}
	}

	/**
	 * GitHub'dan çekilen CSS'i içe aktarır.
	 *
	 * Eşlemeyi Rust yapıyor (`import_css_text` -> `parse_foreign_css`):
	 * `--fds-*` token'ları ve tanınan selector'lar kontrollere bağlanıyor,
	 * geri kalan her kural ham CSS'te olduğu gibi korunuyor.
	 */
	async function handleImport(payload: { css: string; name: string; source: string }) {
		try {
			// `resetAll` değil: onun asenkron yeniden üretimi, aşağıda yüklenen
			// temanın CSS'ini ezebilirdi (bkz. `resetState` başlığındaki not).
			resetState();
			await tick();

			const next = await importCssText(payload.css, KNOWN_SELECTORS);
			adoptDoc(next);

			projectId = "";
			projectName = payload.name;
			projectSource = payload.source;
			externalPath = null;
			externalDirty = false;
			editMode = "visual";
			viewport = settings.defaultViewport;
			currentPath = settings.defaultPreviewPath;

			await tick();
			await push(true);
			go(currentPath);

			hasOpenProject = true;
			view = "editor";

			// İçe aktarılan tema doğrudan bir proje olarak kaydedilir; kullanıcı
			// adını zaten diyalogda onayladı.
			await persistProject();
			projectStatus = `"${payload.name}" içe aktarıldı ve proje olarak kaydedildi.`;
			error = "";
		} catch (e) {
			error = String(e);
		}
	}

	/** Ana ekrandan doğrudan bir .css dosyası açmak. */
	async function openFileFromHome() {
		await openExternal();
		if (!externalPath) return;
		projectId = "";
		projectName = externalPath.split(/[\\/]/).pop()?.replace(/\.css$/i, "") || "Harici tema";
		projectSource = null;
		editMode = settings.defaultEditMode;
		viewport = settings.defaultViewport;
		currentPath = settings.defaultPreviewPath;
		hasOpenProject = true;
		view = "editor";
		await tick();
		savedSnapshot = projectSignature;
	}

	/**
	 * Uygulama içi giriş diyaloğunu açar.
	 *
	 * Görünümü DEĞİŞTİRMİYOR: kullanıcı hangi ekrandaysa diyalog orada açılır.
	 * Önceki sürüm editöre atlayıp önizlemeyi gösteriyordu; hem gereksizdi hem
	 * de ekranın görünen kısmını bozuyordu (önizleme diyaloğun üstüne çizilir).
	 *
	 * Kullanıcı açısından önizlemenin girişle bir ilgisi yok. Teknik olarak
	 * yine gerekli ama görünmeden: `POST /user/auth` Vanguard geçidinin
	 * arkasında ve `Gateway-Token` başlığı yalnızca sitenin `/osc.wasm` ile
	 * imzalanan yamalı `fetch`'i tarafından eklenebiliyor — doğrudan istek
	 * 401 + "Unauthorized access is denied by OpenAnime Vanguard." dönüyor
	 * (curl ile doğrulandı). Yani önizleme sayfası isteğin "posta kutusu";
	 * yüklü olması yeter, görünür olması gerekmiyor. Hesap kartı da aynı
	 * köprüyü aynı şekilde, önizleme gizliyken kullanıyor.
	 */
	function openLoginDialog() {
		loginDialogOpen = true;
	}

	let loginDialogOpen = false;

	/** Giriş başarılı: oturum durumunu hemen tazele, 3 sn'lik anketi bekleme. */
	async function onLoginSuccess() {
		loginDialogOpen = false;
		await refreshLoginState();
	}

	/**
	 * Çıkış tamamlandı: oturum durumunu hemen tazele.
	 *
	 * Kısa bir bekleme var, çünkü köprü çerezleri sildikten SONRA önizlemeyi
	 * yeniliyor (bkz. `preview_init.js` -> `__OA_API_LOGOUT__`). Hemen
	 * sorsaydık `preview_login_state` çerez kavanozunu yenileme başlamadan
	 * okuyabilir ve kullanıcıyı hâlâ giriş yapmış gösterebilirdi. 3 sn'lik
	 * anket zaten yakalardı ama arayüzün o kadar geç tepki vermesi yanlış
	 * görünürdü.
	 */
	async function onLoggedOut() {
		await new Promise((resolve) => setTimeout(resolve, 250));
		await refreshLoginState();
	}

	/**
	 * Kayıt / parola sıfırlama / QR / e-posta doğrulama için sitenin kendisine
	 * bırakır. Giriş diyaloğunun aksine BURADA önizlemeye geçiyoruz — kullanıcı
	 * o akışları sitede kendisi tamamlayacak, yani önizlemeyi görmesi gerekiyor.
	 */
	function openSiteForAuth() {
		loginDialogOpen = false;
		hasOpenProject = true;
		view = "editor";
		go("/");
	}

	// --- Önizlemedeki oturum durumu ------------------------------------------
	//
	// Giriş, önizleme webview'inin İÇİNDE yapılıyor. Durumu sayfadan mesajla
	// öğrenmiyoruz; Rust, webview'in çerez kavanozunu doğrudan okuyor — bu
	// yolun hiçbir IPC'ye ihtiyacı yok.
	//
	// Not: o webview'e artık ÇOK DAR bir IPC yüzeyi açıldı (yalnızca
	// `core:event:allow-emit`, yalnızca https://openani.me — bkz.
	// `src-tauri/capabilities/preview-bridge.json`). PLAN.md §2.4'teki uyarı
	// hiçbir şey açmamayı öneriyordu, ama hesap bilgisi başka türlü
	// alınamıyor: api.openani.me'nin "Vanguard" geçidi `Gateway-Token`
	// başlığı olmayan her isteği 401'liyor ve o başlık yalnızca sayfanın
	// kendi wasm imzasıyla üretilebiliyor. Bu yüzden bu durum sorgusu
	// özellikle eski (çerez okuyan) yolunda bırakıldı.
	let loggedIn = false;
	let loginTimer: ReturnType<typeof setInterval> | null = null;

	async function refreshLoginState() {
		try {
			loggedIn = (await previewLoginState()).loggedIn;
		} catch {
			// Önizleme henüz kurulmadıysa ya da çerez okunamadıysa durumu
			// değiştirmiyoruz; bu bir hata değil, "henüz bilmiyoruz".
		}
	}

	/**
	 * Oturum durumunu yalnızca GÖRÜNDÜĞÜ yerlerde yokluyoruz.
	 *
	 * Hesap kutusu ana ekranda ve ayarlarda; editördeyken ekranda karşılığı
	 * yok. Kullanıcı zaten girişi editördeki önizlemede yapıyor, oradan
	 * çıkarken bu yoklama kendiliğinden tetikleniyor.
	 */
	$: watchLoginState(view);

	function watchLoginState(current: NavId) {
		if (loginTimer) {
			clearInterval(loginTimer);
			loginTimer = null;
		}
		if (current === "editor") return;

		refreshLoginState();
		// Kullanıcı önizlemeyi başka bir sekmede açık bırakıp giriş yapmış
		// olabilir; birkaç saniyede bir tazelemek yeterli, sürekli yoklama
		// gereksiz IPC trafiği olurdu.
		loginTimer = setInterval(refreshLoginState, 3000);
	}

	/**
	 * Ana ekrandaki "Hesap" düğmesi.
	 *
	 * Önceden editörü açıp önizlemeyi openani.me'nin gerçek `/settings`
	 * sayfasına yönlendiriyordu. Artık editöre hiç girmiyor: doğrudan
	 * uygulamanın kendi native ayarlar sayfasına gidiyor — orada hesap
	 * bilgileri zaten uygulamanın kendi kartında gösteriliyor (bkz.
	 * `AccountCard.svelte`).
	 */
	function openAccountSettings() {
		navigate("settings");
	}

	async function openProjectsFolder() {
		try {
			if (projectsPath) await openPath(projectsPath);
		} catch (e) {
			error = String(e);
		}
	}

	// --- Önizlemenin görünürlüğü --------------------------------------------
	//
	// Native child webview HER ZAMAN host sayfanın içeriğinin üstüne çiziliyor,
	// dolayısıyla ana ekranda onu bir HTML katmanıyla örtmek imkânsız —
	// gerçekten gizlemek gerekiyor.
	//
	// Aynı sebep bir hataya yol açıyordu: `ContentDialog` pencerede ORTALANIYOR,
	// yani editördeyken tam olarak önizlemenin durduğu alana düşüyor ve
	// webview'in altında kalıp GÖRÜNMÜYORDU. Kullanıcı "Ana ekran"a tıkladığında
	// onay diyaloğu açılıyor ama ekranda hiçbir şey belirmiyor, uygulama
	// kilitlenmiş gibi hissettiriyordu. z-index'in bu soruna çaresi yok:
	// webview host sayfanın DOM'unda değil.
	//
	// Çözüm, diyalog açıkken önizlemeyi geçici olarak gizlemek. Zaten var olan
	// görünürlük yolunu kullanıyoruz; yeni bir mekanizma eklenmiyor.
	// Önizleme `add_child` ile eklenmiş NATİF bir webview ve her zaman host
	// sayfanın İÇERİĞİNİN ÜSTÜNE çiziliyor (bkz. `preview.rs`) — z-index onu
	// etkilemiyor. Bu yüzden ekranı kaplayan her diyalog burada listelenmek
	// ZORUNDA; listelenmezse diyalog açılır ama önizlemenin altında kalır.
	$: modalOpen =
		confirmLeave ||
		namingOpen ||
		confirmResetAll ||
		aboutDialogOpen ||
		updateDialogOpen ||
		loginDialogOpen;
	$: syncPreviewVisibility(view, modalOpen);

	async function syncPreviewVisibility(current: NavId, blocked: boolean) {
		const shouldShow = current === "editor" && !blocked;
		try {
			await setPreviewVisible(shouldShow);
			if (shouldShow) {
				await tick();
				syncBounds();
			}
		} catch (e) {
			error = String(e);
		}
	}

	// Editör görünümü kurulup `previewSlot` DOM'a girdiğinde ölçümü başlat,
	// çıktığında bırak. Eskiden bu `onMount`'taydı; artık slot her zaman var
	// olmadığı için yaşam döngüsüne değil elemanın varlığına bağlıyoruz.
	let slotObserver: ResizeObserver | null = null;

	$: if (previewSlot && !slotObserver) {
		slotObserver = new ResizeObserver(syncBounds);
		slotObserver.observe(previewSlot);
		syncBounds();
	} else if (!previewSlot && slotObserver) {
		slotObserver.disconnect();
		slotObserver = null;
	}

	// Ayarlar her değiştiğinde diske yaz ve arayüz temasını uygula.
	//
	// Tetikleyici `settings`'in YENİDEN ATANMASI; alan mutasyonu değil. Ayarlar
	// sayfası zaten her değişimde `change` olayını yollayıp aşağıdaki
	// işleyicide nesneyi yeniden atıyor; dolayısıyla buraya derin bir
	// karşılaştırma (ör. `JSON.stringify`) koymanın bir etkisi olmaz.
	$: if (settings) {
		saveSettings(settings);
	}
	$: if (settings?.appTheme) {
		applyAppTheme(settings.appTheme);
	}

	// --- Discord Rich Presence ---------------------------------------------
	//
	// Kapsam kararı BURADA veriliyor, Rust'ta değil: "sadece düzenlerken"
	// seçeneği `view`'a bakıyor ve `view` zaten burada yaşıyor. Rust tarafına
	// taşımak, arayüz durumunun ikinci bir kopyasını oradaki duruma da
	// yansıtmayı gerektirirdi.
	$: presenceVisible =
		settings.discordRpc &&
		(settings.discordRpcScope === "always" || view === "editor");

	// İki ayrı reaktif ifade, çünkü tetikleyicileri farklı: birincisi yalnızca
	// görünürlük değiştiğinde, ikincisi kullanıcı uygulamada gezindikçe
	// çalışıyor. Tek ifadede birleştirilseydi her sekme değişimi
	// `discord_set_enabled`'ı da gereksiz yere çağırırdı.
	$: setPresenceEnabled(presenceVisible);

	// `projectName` editörde açık olan temanın adı; diğer görünümlerde
	// Discord'a gönderilmiyor (bkz. `discord.rs` -> `describe`), ama burada
	// filtrelemeye gerek yok — Rust tarafı görünüme göre zaten yok sayıyor.
	$: if (presenceVisible) {
		updatePresence(view, projectName, editMode, settings.discordRpcThemeName);
	}

	onMount(() => {
		// Ayarlar bildirimde yüklendi (yukarı bkz.); burada yalnızca onlara
		// bağlı açılış durumu kuruluyor.
		applyAppTheme(settings.appTheme);
		viewport = settings.defaultViewport;
		currentPath = settings.defaultPreviewPath;

		refreshProjects();
		getVersion()
			.then((v) => (appVersion = `v${v}`))
			.catch(() => {});
		invoke<string>("projects_dir_path")
			.then((path) => (projectsPath = path))
			.catch(() => {});
		checkForUpdatesOnStartup();

		window.addEventListener("resize", syncBounds);
		const removeEasterEgg = installEasterEgg();
		push();

		return () => {
			slotObserver?.disconnect();
			if (loginTimer) clearInterval(loginTimer);
			window.removeEventListener("resize", syncBounds);
			removeEasterEgg();
		};
	});
</script>

<div class="app">
	<TitleBar title={titleContext} />

	<div class="body">
		<NavRail
			current={view}
			editorEnabled={hasOpenProject}
			aboutOpen={aboutDialogOpen}
			onNavigate={navigate}
			onOpenAbout={() => (aboutDialogOpen = true)}
		/>

		<div class="view">
		{#if view === "home"}
			<Launcher
				{projects}
				onOpen={openProject}
				onCreate={createProject}
				onOpenFile={openFileFromHome}
				onImport={handleImport}
				onRename={handleRename}
				onDelete={handleDelete}
				onLogin={openLoginDialog}
				{loggedIn}
				onOpenAccount={openAccountSettings}
			/>
		{:else if view === "settings"}
			<AppSettings
				bind:settings
				on:change={() => {
					settings = { ...settings };
					saveSettings(settings);
					if (settings.appTheme) applyAppTheme(settings.appTheme);
				}}
				projectCount={projects.length}
				{projectsPath}
				{appVersion}
				onOpenProjectsFolder={openProjectsFolder}
				onLogin={openLoginDialog}
				{onLoggedOut}
				{loggedIn}
				onCheckForUpdates={() => runUpdateCheck(true)}
				{updateCheckStatus}
				{updateCheckError}
				{updateChannelLabel}
				{onChannelChange}
			/>
		{:else}
			<!-- Editör: mevcut panel + önizleme yerleşimi olduğu gibi korunuyor. -->
			<div class="shell">
	<div class="panel">
		<header>
			<div class="row-between">
				<TextBlock variant="subtitle">{projectName}{dirty ? " •" : ""}</TextBlock>
				<Tooltip text="Tüm temayı sitenin varsayılanlarına döndür">
					<Button on:click={() => (confirmResetAll = true)}>
						<Icon name="resetAll" size={14} /><span class="gap">Tümünü sıfırla</span>
					</Button>
				</Tooltip>
			</div>
			<TextBlock variant="caption" class="text-secondary">
				Değişiklikler sağdaki siteye anında yansır.
			</TextBlock>

			<div class="chips">
				<Button variant={dirty ? "accent" : "standard"} on:click={saveCurrentProject}>
					<Icon name="save" size={14} /><span class="gap">
						{projectId ? "Projeyi kaydet" : "Proje olarak kaydet…"}
					</span>
				</Button>
				<Button on:click={() => askName()}>Farklı adla kaydet…</Button>
				<Button variant="hyperlink" on:click={() => navigate("home")}>Ana ekran</Button>
			</div>

			{#if projectSource}
				<Tooltip text={projectSource}>
					<TextBlock variant="caption">
						<Icon name="github" size={12} /><span class="gap">GitHub'dan içe aktarıldı</span>
					</TextBlock>
				</Tooltip>
			{/if}
		</header>

		{#if error}
			<StatusBar severity="critical" title="Hata" message={error} closable={false} />
		{/if}

		{#if projectStatus}
			<StatusBar severity="success" title="" message={projectStatus} closable={true} />
		{/if}

		<!--
			`on:click` burada gereksiz görünebilir: SegmentedControl zaten
			bind:value ile değeri kendisi yazıyor. Ama o mekanizma DOM'da
			`closest("[data-segment-id]")` araması ve modül düzeyinde paylaşılan
			bir sözlük üzerinden çalışıyor; kritik yolda tek dayanak olmasın diye
			doğrudan atama da yapıyoruz. İkisi aynı değeri yazdığı için çakışmaz.
		-->
		<div class="mode-switch">
			<SegmentedControl bind:value={editMode}>
				<SegmentedControlButton value="visual" on:click={() => (editMode = "visual")}>
					Görsel
				</SegmentedControlButton>
				<SegmentedControlButton value="code" on:click={() => (editMode = "code")}>
					Kod
				</SegmentedControlButton>
			</SegmentedControl>
		</div>

		{#if editMode === "visual"}
			<div class="category-switch">
				<SegmentedControl bind:value={editorCategory}>
					<SegmentedControlButton value="colors" on:click={() => (editorCategory = "colors")}>
						Renkler
					</SegmentedControlButton>
					<SegmentedControlButton value="shape" on:click={() => (editorCategory = "shape")}>
						Şekil
					</SegmentedControlButton>
					<SegmentedControlButton value="motion" on:click={() => (editorCategory = "motion")}>
						Efekt
					</SegmentedControlButton>
					<SegmentedControlButton value="media" on:click={() => (editorCategory = "media")}>
						Medya
					</SegmentedControlButton>
					<SegmentedControlButton value="components" on:click={() => (editorCategory = "components")}>
						Bileşen
					</SegmentedControlButton>
					<SegmentedControlButton value="advanced" on:click={() => (editorCategory = "advanced")}>
						Gelişmiş
					</SegmentedControlButton>
				</SegmentedControl>
			</div>

			<div class="sections">
				{#if editorCategory === "colors"}
					<Section
						icon="accent"
						title="Vurgu rengi"
						expanded
						onReset={() => resetSection("accent")}
					>
						<div class="swatches">
							{#each ramp as step, i}
								<Tooltip text="--fds-{RAMP_NAMES[i]}: {step}">
									<div class="swatch" style="background: hsl({step})"></div>
								</Tooltip>
							{/each}
						</div>

						<!-- svelte-ignore a11y-label-has-associated-control -->
						<label>
							<TextBlock variant="caption">Ton (H) — {Math.round(accentH)}°</TextBlock>
							<Slider
								bind:value={accentH}
								min={0}
								max={360}
								step={1}
								suffix="°"
								on:input={(e) => updateAccentFromSlider(extractSliderVal(e), accentS, accentL)}
								on:change={(e) => updateAccentFromSlider(extractSliderVal(e), accentS, accentL)}
							/>
						</label>
						<!-- svelte-ignore a11y-label-has-associated-control -->
						<label>
							<TextBlock variant="caption">Doygunluk (S) — {Math.round(accentS)}%</TextBlock>
							<Slider
								bind:value={accentS}
								min={0}
								max={100}
								step={1}
								suffix="%"
								on:input={(e) => updateAccentFromSlider(accentH, extractSliderVal(e), accentL)}
								on:change={(e) => updateAccentFromSlider(accentH, extractSliderVal(e), accentL)}
							/>
						</label>
						<!-- svelte-ignore a11y-label-has-associated-control -->
						<label>
							<TextBlock variant="caption">Işıklılık (L) — {Math.round(accentL)}%</TextBlock>
							<Slider
								bind:value={accentL}
								min={0}
								max={100}
								step={1}
								suffix="%"
								on:input={(e) => updateAccentFromSlider(accentH, accentS, extractSliderVal(e))}
								on:change={(e) => updateAccentFromSlider(accentH, accentS, extractSliderVal(e))}
							/>
						</label>

						<div class="picker">
							<TextBlock variant="caption">Palet, hex ve RGB</TextBlock>
							<ColorPicker hex={accentHex} on:change={(e) => onAccentPick(e.detail)} />
						</div>

						<div class="chips">
							{#each PRESETS as preset}
								<Button on:click={() => usePreset(preset.hsl)}>
									<span
										class="dot"
										style="background: hsl({preset.hsl[0]}, {preset.hsl[1]}%, {preset.hsl[2]}%)"
									></span>
									{preset.name}
								</Button>
							{/each}
						</div>
					</Section>

					<Section icon="hover" title="Hover ve tıklama renkleri" onReset={() => resetSection("hover")}>
						<ToggleSwitch bind:checked={hoverEnabled}>Etkileşim renklerini özelleştir</ToggleSwitch>
						<TextBlock variant="caption">
							Varsayılanlar sitenin kendi opaklık değerleridir. Siyah/beyaz geçişleri sitenin karanlık
							veya aydınlık modda olmasına göre otomatik seçilir.
						</TextBlock>

						{#each HOVER_TOKENS as spec, i}
							<ColorField
								{spec}
								bind:hex={hoverColors[i].hex}
								bind:alpha={hoverColors[i].alpha}
								disabled={!hoverEnabled}
							/>
						{/each}
					</Section>

					<Section icon="button" title="Buton renkleri" onReset={() => resetSection("buttons")}>
						<ToggleSwitch bind:checked={buttonsEnabled}>Buton renklerini özelleştir</ToggleSwitch>
						{#each BUTTON_TOKENS as spec, i}
							<ColorField
								{spec}
								bind:hex={buttonColors[i].hex}
								bind:alpha={buttonColors[i].alpha}
								disabled={!buttonsEnabled}
							/>
						{/each}
						<div class="field">
							<TextBlock variant="caption">Standart buton metni</TextBlock>
							<TextBox bind:value={buttonTextHex} disabled={!buttonsEnabled} clearButton={false} />
							<TextBlock variant="caption">
								Bunun için sitede ayrı bir token yok; <code>.button</code> kuralı üzerinden ezilir.
							</TextBlock>
						</div>
					</Section>
					<AdvancedSections bind:adv {pickImage} mode={seedMode} {ramp} category="colors" />

				{:else if editorCategory === "shape"}
					<Section icon="corner" title="Köşe yuvarlaklığı" expanded onReset={() => resetSection("radius")}>
						<ToggleSwitch bind:checked={radiusEnabled}>Yarıçapları özelleştir</ToggleSwitch>
						<!-- svelte-ignore a11y-label-has-associated-control -->
						<label>
							<TextBlock variant="caption">Kontroller — {controlRadius}px</TextBlock>
							<Slider
								bind:value={controlRadius}
								min={0}
								max={16}
								step={1}
								disabled={!radiusEnabled}
								suffix="px"
							/>
						</label>
						<!-- svelte-ignore a11y-label-has-associated-control -->
						<label>
							<TextBlock variant="caption">Katmanlar (flyout, dialog) — {overlayRadius}px</TextBlock>
							<Slider
								bind:value={overlayRadius}
								min={0}
								max={24}
								step={1}
								disabled={!radiusEnabled}
								suffix="px"
							/>
						</label>
					</Section>

					<AdvancedSections bind:adv {pickImage} mode={seedMode} {ramp} category="shape" />

				{:else if editorCategory === "motion"}
					<Section icon="motion" title="Animasyon ve geçişler" expanded onReset={() => resetSection("motion")}>
						<ToggleSwitch bind:checked={motionEnabled}>Geçişleri özelleştir</ToggleSwitch>
						<TextBlock variant="caption">
							Sitenin kendi süre token'ları ölçeklenir; yeni bir animasyon sistemi eklenmez.
						</TextBlock>
						<!-- svelte-ignore a11y-label-has-associated-control -->
						<label>
							<TextBlock variant="caption">
								Hız — {motionScale === 0 ? "anlık" : `${motionScale.toFixed(2)}×`}
							</TextBlock>
							<Slider
								bind:value={motionScale}
								min={0}
								max={3}
								step={0.05}
								disabled={!motionEnabled}
								suffix="×"
							/>
						</label>
						<TextBlock variant="caption">Yumuşatma eğrisi</TextBlock>
						<ComboBox
							items={EASINGS.map((e) => ({ name: e.name, value: e.value }))}
							bind:value={motionEasing}
							disabled={!motionEnabled}
						/>
						<TextBlock variant="caption">
							{#each DURATION_TOKENS as d, i}{i > 0 ? " · " : ""}{d.label}: {Math.round(
									d.base * motionScale
								)}ms{/each}
						</TextBlock>
					</Section>

					<AdvancedSections bind:adv {pickImage} mode={seedMode} {ramp} category="motion" />

				{:else if editorCategory === "media"}
					<AdvancedSections bind:adv {pickImage} mode={seedMode} {ramp} category="media" />

				{:else if editorCategory === "components"}
					<AdvancedSections bind:adv {pickImage} mode={seedMode} {ramp} category="components" />

				{:else if editorCategory === "advanced"}
					<AdvancedSections bind:adv {pickImage} mode={seedMode} {ramp} category="advanced" />

					<Section icon="code" title="Ham CSS" onReset={() => resetSection("raw")}>
						<StatusBar
							severity="caution"
							title="Dikkat"
							message="Buraya yazdığınız CSS resmi tema token'ları dışına çıkar; site güncellemelerinde bozulabilir."
							closable={false}
						/>
						<TextArea bind:value={doc.rawCss} placeholder="body &lbrace; letter-spacing: .2px; &rbrace;" />
					</Section>
				{/if}
			</div>
		{:else}
			<div class="sections">
				<div class="row-between">
					<TextBlock variant="bodyStrong">
						{externalPath ? "Harici dosya" : "CSS"}{externalDirty ? " •" : ""}
					</TextBlock>
					<TextBlock variant="caption">{cssText.length} karakter</TextBlock>
				</div>

				{#if externalPath}
					<Tooltip text={externalPath}>
						<TextBlock variant="caption">
							<Icon name="file" size={12} />
							<span class="gap">{externalPath.split(/[\\/]/).pop()}</span>
						</TextBlock>
					</Tooltip>
				{/if}

				<div class="chips">
					<Button on:click={openExternal}>
						<Icon name="open" size={14} /><span class="gap">Dosya aç…</span>
					</Button>
					<Button variant={externalDirty ? "accent" : "standard"} on:click={() => saveExternal(false)}>
						<Icon name="save" size={14} /><span class="gap">Kaydet</span>
					</Button>
					<Button on:click={() => saveExternal(true)}>Farklı kaydet…</Button>
					{#if externalPath}
						<Button variant="hyperlink" on:click={closeExternal}>Dosyayı bırak</Button>
					{/if}
				</div>

				<div class="chips">
					<Button on:click={loadStarter}>
						<Icon name="code" size={14} /><span class="gap">Site CSS şablonunu ekle</span>
					</Button>
					<Button variant="hyperlink" on:click={copyCss}>
						{copied ? "Kopyalandı ✓" : "Panoya kopyala"}
					</Button>
				</div>

				{#if fileStatus}
					<StatusBar severity="success" title="" message={fileStatus} closable={false} />
				{/if}

				<TextBlock variant="caption">
					Ctrl+Boşluk ile tamamlama. İki tire yazınca tema token'ları, nokta ve diyez yazınca
					sitenin gerçek class/id'leri listelenir.
				</TextBlock>
				<CssEditor value={cssText} on:change={onCodeChange} />
				<StatusBar
					severity="information"
					title="İki yönlü senkron"
					message="İşaretli blok içindeki değerleri elle değiştirirseniz soldaki kontroller de güncellenir. Altı accent basamağı --fds-accent-base'ten türetildiği için elle düzenlenirse yeniden üretilir. Harici dosya modunda da aynı senkron çalışır."
					closable={false}
				/>
			</div>
		{/if}

		<div class="sections">
			<Section icon="page" title="Önizleme sayfası">
				<ComboBox items={routeItems} bind:value={currentPath} on:select={onRouteSelect} />
				<div class="row">
					<TextBox bind:value={customPath} placeholder={PARAM_ROUTES[0]} clearButton={false} />
					<Button on:click={() => go(customPath)}>Git</Button>
				</div>
				<TextBlock variant="caption">
					Parametreli sayfalar için: {PARAM_ROUTES.join("  ·  ")}
				</TextBlock>
			</Section>

		</div>
	</div>

	<div class="preview-area">
		<!--
			Görünüm seçici: ayrı bir liste öğesi değil, önizlemenin üstünde duran
			floating bir kontrol. Sağ üst köşede, segmented control.
		-->
		<div class="viewport-bar">
			<div class="viewport-pill">
				<SegmentedControl bind:value={doc.mode}>
					<SegmentedControlButton value="system" on:click={() => setMode("system")}>
						Sistem
					</SegmentedControlButton>
					<SegmentedControlButton value="light" on:click={() => setMode("light")}>
						Açık
					</SegmentedControlButton>
					<SegmentedControlButton value="dark" on:click={() => setMode("dark")}>
						Koyu
					</SegmentedControlButton>
				</SegmentedControl>
			</div>

			<div class="viewport-pill">
				<SegmentedControl bind:value={viewport}>
					{#each VIEWPORTS as vp}
						<SegmentedControlButton value={vp.id} on:click={() => (viewport = vp.id)}>
							{vp.name}
						</SegmentedControlButton>
					{/each}
				</SegmentedControl>
			</div>
		</div>

		<!-- Önizleme webview'i bu boşluğun üstüne native olarak oturur. -->
		<div class="preview-slot" bind:this={previewSlot}></div>
	</div>
			</div>
		{/if}
		</div>
	</div>
</div>

<!-- --- Proje adı ------------------------------------------------------- -->
<ContentDialog bind:open={namingOpen} title="Projeyi kaydet" size="standard">
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Proje adı</TextBlock>
		<TextBox
			bind:value={nameValue}
			clearButton={false}
			on:keydown={(e) => isEnter(e) && confirmName()}
		/>
	</label>
	<TextBlock variant="caption">
		Kontrol değerleri, ham CSS ve varsa logo/maskot görselleri tek bir dosyada saklanır.
	</TextBlock>

	<svelte:fragment slot="footer">
		<Button variant="accent" disabled={!nameValue.trim()} on:click={confirmName}>Kaydet</Button>
		<Button
			on:click={() => {
				namingOpen = false;
				afterSave = null;
			}}
		>
			Vazgeç
		</Button>
	</svelte:fragment>
</ContentDialog>

<!-- --- Kaydedilmemiş değişikliklerle ayrılma ---------------------------- -->
<ContentDialog bind:open={confirmLeave} title="Kaydedilmemiş değişiklikler var" size="standard">
	<TextBlock>
		<strong>{projectName}</strong> üzerinde kaydedilmemiş değişiklikler var. Ne yapmak
		istersiniz?
	</TextBlock>
	<TextBlock variant="caption">
		"Ana ekrana dönerken otomatik kaydet" ayarını açarsanız bu soru bir daha sorulmaz.
	</TextBlock>

	<svelte:fragment slot="footer">
		<Button variant="accent" on:click={saveAndLeave}>
			<Icon name="save" size={14} /><span class="gap">Kaydet ve çık</span>
		</Button>
		<Button on:click={leaveWithoutSaving}>Kaydetmeden çık</Button>
		<Button on:click={() => (confirmLeave = false)}>Vazgeç</Button>
	</svelte:fragment>
</ContentDialog>

<ContentDialog bind:open={confirmResetAll} title="Tüm tema sıfırlansın mı?" size="standard">
	<TextBlock>
		Vurgu rengi, köşe yuvarlaklığı, renkler, animasyonlar, logo, maskot, arkaplan
		görseli ve ham CSS dahil <strong>bütün ayarlar</strong> sitenin orijinal
		varsayılanlarına döner. Bu işlem geri alınamaz.
	</TextBlock>
	{#if externalPath}
		<StatusBar
			severity="caution"
			title="Harici dosya açık"
			message="Dosya diskte değişmez; sıfırlanmış tema ancak Kaydet'e bastığınızda yazılır."
			closable={false}
		/>
	{/if}

	<svelte:fragment slot="footer">
		<Button variant="accent" on:click={resetAll}>
			<Icon name="resetAll" size={14} /><span class="gap">Evet, tümünü sıfırla</span>
		</Button>
		<Button on:click={() => (confirmResetAll = false)}>Vazgeç</Button>
	</svelte:fragment>
</ContentDialog>

<AboutDialog open={aboutDialogOpen} {appVersion} onClose={() => (aboutDialogOpen = false)} />

<UpdateDialog
	open={updateDialogOpen}
	update={updateAvailable}
	onClose={() => (updateDialogOpen = false)}
	onSkip={skipUpdate}
/>

<LoginDialog
	open={loginDialogOpen}
	onClose={() => (loginDialogOpen = false)}
	onSuccess={onLoginSuccess}
	onOpenSite={openSiteForAuth}
/>

<style>
	/* Kısıt gereği burada GÖRSEL karar yok — yalnızca yerleşim iskeleti ve
	   tutarlı boşluklandırma. Tüm renk / kenarlık / yarıçap / tipografi
	   değerleri --fds-* token'larından geliyor. */
	/* Üç katmanlı iskelet:
	     .app  — başlık çubuğu + gövde (dikey)
	     .body — gezinme şeridi + görünüm (yatay)
	     .view — o an seçili görünüm (ana ekran / editör / ayarlar)
	   Başlık çubuğu artık ızgara yerine dikey akışın ilk elemanı; şerit her
	   görünümde aynı yerde durduğu için pencere düzeni sabit kalıyor. */
	.app {
		display: flex;
		flex-direction: column;
		height: 100vh;
		overflow: hidden;
	}

	.body {
		flex: 1 1 auto;
		/* Olmazsa içerik uzayınca kendi kaydırması yerine pencereyi taşırıyor. */
		min-height: 0;
		display: flex;
	}

	.view {
		flex: 1 1 auto;
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}

	/* Ana ekran ve ayarlar sayfası kalan alanın tamamını kaplasın. */
	.view > :global(*) {
		flex: 1 1 auto;
		min-height: 0;
	}

	/* Editör: panel + önizleme. Mevcut yerleşim korunuyor, yalnızca başlık
	   çubuğu satırı artık burada değil (yukarı, .app'e taşındı). */
	.shell {
		display: grid;
		grid-template-columns: 420px 1fr;
		min-height: 0;
		overflow: hidden;
	}

	.panel {
		/* Izgara satırı taşmasın diye min-height: 0 şart; yoksa panel
		   içeriği uzayınca kendi kaydırması yerine pencereyi taşırıyor. */
		min-height: 0;
		overflow-y: auto;
		box-sizing: border-box;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 16px;
		background-color: var(--fds-solid-background-tertiary);
		border-right: 1px solid var(--fds-divider-stroke-default);
	}

	/* Panel bir dikey flex konteyneri ve içeriği taşınca tarayıcı çocukları
	   küçültüyor. fluent'in `.segmented-control`'ü `overflow: hidden` taşıdığı
	   için min-height'ı `auto` yerine 0 hesaplanıyor ve sıfır yüksekliğe kadar
	   eziliyordu — Kod modunda Görsel/Kod anahtarı görünmez oluyordu.
	   Hiçbir doğrudan çocuk küçülmesin; panel bunun yerine kaysın.
	   `:global` şart, çünkü çocukların çoğu bileşen ve kök elemanları bu
	   bileşenin scope sınıfını almıyor. */
	.panel > :global(*),
	.sections > :global(*) {
		flex: 0 0 auto;
	}

	/* Panel kaydığında mod anahtarı yukarıda görünür kalsın. */
	.mode-switch {
		position: sticky;
		top: 0;
		z-index: 1;
		background-color: var(--fds-solid-background-tertiary);
		padding-bottom: 8px;
	}

	/* Sağ sütun: üstte floating görünüm seçici şeridi, altta webview'in
	   oturacağı boşluk. Şerit `auto`, boşluk kalan her şeyi alır. */
	.preview-area {
		display: grid;
		grid-template-rows: auto 1fr;
		min-height: 0;
		min-width: 0;
		background-color: var(--fds-solid-background-base);
	}

	.viewport-bar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		/* Şerit sürüklenebilir olmasın diye title bar'dan bağımsız. */
		box-sizing: border-box;
	}

	/* Yüzen his: kendi zemini, kenarlığı ve gölgesi olan bir hap.
	   Tüm değerler --fds-* token'larından. */
	.viewport-pill {
		border-radius: var(--fds-control-corner-radius);
		background-color: var(--fds-card-background-default);
		border: 1px solid var(--fds-card-stroke-default);
		box-shadow: var(--fds-card-shadow);
		padding: 2px;
	}

	.preview-slot {
		min-height: 0;
	}

	/* Tek bir boşluk ölçeği: bölümler arası 8, bölüm içi alanlar 4. */
	.sections {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	header,
	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.category-switch {
		padding: 0 1rem 0.5rem 1rem;
	}

	.category-switch :global(.segmented-control) {
		width: 100%;
		display: flex;
	}

	.category-switch :global(.segmented-control-item),
	.category-switch :global(.segmented-control-button),
	.category-switch :global(button) {
		flex: 1 1 0%;
		text-align: center;
		justify-content: center;
	}

	.row {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.row-between {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.swatches {
		display: flex;
		gap: 4px;
	}

	.swatch {
		flex: 1 1 0;
		height: 28px;
		border-radius: var(--fds-control-corner-radius);
		border: 1px solid var(--fds-control-stroke-default);
	}

	.dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		display: inline-block;
		margin-right: 6px;
	}

	.gap {
		margin-left: 6px;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	:global(.combo-box-dropdown) {
		scrollbar-width: none !important;
		-ms-overflow-style: none !important;
	}

	:global(.combo-box-dropdown::-webkit-scrollbar) {
		display: none !important;
		width: 0 !important;
		height: 0 !important;
		background: transparent !important;
	}
</style>
