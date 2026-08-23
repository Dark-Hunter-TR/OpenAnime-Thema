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
		ToggleSwitch
	} from "fluent-svelte-extra";
	import Tooltip from "$lib/Tooltip.svelte";
	import { unclip } from "$lib/unclip";

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
	import GithubImportDialog from "$lib/GithubImportDialog.svelte";
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
		clampPanelWidth,
		loadSettings,
		saveSettings,
		type AppSettings as AppSettingsState
	} from "$lib/settings";
	import {
		deleteProject,
		importCssText,
		formatUpdated,
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
		controlledRuleProps,
		ADV_TOKEN_KEYS,
		type AdvState
	} from "$lib/advancedBuild";
	import { extractUrl, mergeRuleOverrides, resolveLength } from "$lib/cssDecls";
	import { SITE_DEFAULTS, seedColor, seedColors } from "$lib/defaults";
	import { STARTER_MARKER, starterTemplate } from "$lib/starter";
	import {
		applyCssText,
		applyTheme,
		debounce,
		defaultDoc,
		previewClearData,
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
		BADGE_TEXT_SELECTOR,
		BG_BODY_SELECTOR,
		BG_TRANSPARENT_SELECTOR,
		CARD_SELECTOR,
		ENHANCED_SELECTOR,
		ENHANCED_TEXT_SELECTOR,
		AVATAR_SELECTOR,
		BANNER_PROGRESS_SELECTOR,
		BANNER_SELECTED_SELECTOR,
		COMMENT_INPUT_SELECTOR,
		COMMENT_SELECTOR,
		FONT_PRESETS,
		KNOWN_SELECTORS,
		TEXT_TOKENS,
		LOGO_IMAGE_HIDE_SELECTOR,
		LOGO_IMAGE_SELECTOR,
		LOGO_BADGE_SELECTOR,
		LOGO_ROW_SELECTOR,
		LOGO_TEXT_SELECTOR,
		MASCOT_SLOTS,
		PLAYER_BAR_SELECTOR,
		PLAYER_EPISODE_SELECTOR,
		PLAYER_SLIDER_SELECTOR,
		RELEASED_BADGE_SELECTOR,
		RELEASED_TEXT_SELECTOR,
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
		LINK_TOKENS,
		SIDEBAR_SELECTED_TOKEN,
		SURFACE_TOKENS,
		SYSTEM_TOKENS,
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

	// Kontrollerin üretebileceği HER token/kural anahtarı — bir bölüm
	// kapatıldığında `doc.tokenOverrides`/`doc.ruleOverrides`'tan bunlar
	// silinip ÖYLE yeniden birleştiriliyor (bkz. aşağıdaki reaktif blok).
	// `KNOWN_SELECTORS` zaten bu amaçla bakımı yapılan tam liste (advanced.ts);
	// burada yalnız temel (Gelişmiş dışı) buton metni selector'ı ekleniyor.
	const TOKEN_KEY_UNIVERSE = [...ADV_TOKEN_KEYS, ...HOVER_TOKENS, ...BUTTON_TOKENS]
		.map((t) => (typeof t === "string" ? t : t.token))
		.concat(DURATION_TOKENS.map((t) => t.token), EASING_TOKEN);
	const RULE_KEY_UNIVERSE = [...KNOWN_SELECTORS, BUTTON_TEXT_SELECTOR];

	// Kontrollerin bir kuralda SAHİP OLDUĞU özellikler.
	//
	// `TOKEN_KEY_UNIVERSE` zaten özellik düzeyinde çalışıyor (bir token = bir
	// bildirim), ama kurallar öyle değil: tek bir selector'ın gövdesinde hem
	// kontrollerin ürettiği hem de kullanıcının dosyasından gelen bildirimler
	// yan yana durabiliyor. Aşağıdaki harita, yeniden birleştirmede hangi
	// bildirimlere dokunulacağını söylüyor; gerisi olduğu gibi korunuyor
	// (bkz. `cssDecls.ts` -> `mergeRuleOverrides`).
	//
	// `BUTTON_TEXT_SELECTOR` gelişmiş bölümlerin değil `buildRules`'ın çıktısı,
	// bu yüzden elle katılıyor.
	const CONTROLLED_RULE_PROPS: Record<string, Set<string>> = {
		...controlledRuleProps(),
		[BUTTON_TEXT_SELECTOR]: new Set(["color"])
	};

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

	/**
	 * Editör panelinin sekmesi.
	 *
	 * Önceden İKİ ayrı anahtar vardı: Görsel/Kod ve onun altında altı kategori
	 * (Renkler · Şekil · Efekt · Medya · Bileşen · Gelişmiş). O bölme üç ayrı
	 * sorun üretiyordu: "Bileşen" sekmesi hiçbir bölüm içermiyordu (boş açılıyordu),
	 * "Şekil" ve "Efekt" tek bir bölüm için birer sekme harcıyordu, "Gelişmiş" ise
	 * geri kalan on üç bölümün toplandığı bir çekmeceye dönüşmüştü. Üstelik altı
	 * sekme panele sığmadığı için şeridin iki yanına kaydırma okları koymak
	 * gerekmişti.
	 *
	 * Yerine tek bir anahtar geldi ve sekmeler kullanıcının niyetine göre
	 * ayrıldı, CSS kavramlarına göre değil:
	 *
	 *   Temel — en çok istenen beş ayar. Teknik bilgisi olmayan biri buraya
	 *           girip işini bitirebilmeli.
	 *   Tümü  — bütün bölümler, gruplandırılmış tek akış.
	 *   Kod   — CSS metni.
	 */
	let editorTab: "basic" | "all" | "code" = "basic";

	/**
	 * `editMode` artık türetilmiş: davranışı (kod modunda kullanıcının metnini
	 * ezmemek, Discord durumunu yazmak) hâlâ bu ikili ayrım belirliyor, ama
	 * arayüzün tek doğru kaynağı `editorTab`.
	 */
	let editMode: "visual" | "code" = "visual";
	$: editMode = editorTab === "code" ? "code" : "visual";

	/** Kaydedilmiş bir kip değerinden sekmeye. */
	const tabFromMode = (mode: "visual" | "code") => (mode === "code" ? "code" : "basic");

	/**
	 * "Temel" sekmesinde görünen bölümler.
	 *
	 * Beşi de "temayı açan biri ilk olarak neyi değiştirmek ister" sorusunun
	 * karşılığı: rengi, arka planı, logoyu, köşeleri, yazı tipini. Liste burada
	 * duruyor çünkü hem bu dosyanın kendi bölümlerini (vurgu, köşe) hem de
	 * `AdvancedSections`'ınkileri (arka plan, logo, yazı tipi) kapsıyor.
	 */
	const BASIC_SECTIONS = ["accent", "bg", "logo", "radius", "typo"];

	$: showSection = (section: string) =>
		editorTab === "all" || BASIC_SECTIONS.includes(section);


	// --- Üst düzey görünüm ---------------------------------------------------
	// Uygulama artık doğrudan editöre düşmüyor: açılışta ana ekran gelir.
	// SvelteKit route'u değil basit bir durum makinesi kullanmamızın gerekçesi
	// `$lib/nav.ts`'te.
	let view: NavId = "home";

	/**
	 * Ayarlar diskten BİLDİRİMDE okunuyor, `onMount`'ta değil.
	 *
	 * Aşağıdaki "her değişimde kaydet" reaktif ifadesi bileşen ilklenirken de
	 * bir kez çalışıyor ve bu, `onMount` geri çağrılarından önce oluyor.
	 * Yükleme `onMount`'ta yapılsaydı sıra şöyle işlerdi: varsayılanlarla
	 * başla -> varsayılanları `localStorage`'a YAZ -> sonra oku. Yani her
	 * açılış kullanıcının kayıtlı ayarlarını siler ve geri varsayılanları
	 * okurdu; ayarlar hiçbir zaman kalıcı olmazdı.
	 *
	 * `ssr = false` (bkz. `+layout.ts`) olduğu için `localStorage` bu noktada
	 * her zaman var; sunucuda çalışan bir kod yolu yok.
	 */
	let settings: AppSettingsState = loadSettings();

	// Sol panelin genişliği — ayraçtan sürüklenip `settings.panelWidth`e
	// yazılıyor. Önizleme webview'i BAŞKA bir mekanizmadan (previewSlot'u
	// izleyen ResizeObserver -> syncBounds) kendiliğinden yeniden konumlanıyor;
	// burada webview'e dair hiçbir şey yapmıyoruz.
	let panelWidth = clampPanelWidth(settings.panelWidth);
	let panelResizing = false;

	function startPanelResize(e: PointerEvent) {
		e.preventDefault();
		panelResizing = true;
		const startX = e.clientX;
		const startWidth = panelWidth;

		function onMove(ev: PointerEvent) {
			panelWidth = clampPanelWidth(startWidth + (ev.clientX - startX));
		}
		function onUp() {
			panelResizing = false;
			window.removeEventListener("pointermove", onMove);
			window.removeEventListener("pointerup", onUp);
			settings = { ...settings, panelWidth };
		}
		window.addEventListener("pointermove", onMove);
		window.addEventListener("pointerup", onUp);
	}

	function resetPanelWidth() {
		panelWidth = DEFAULT_SETTINGS.panelWidth;
		settings = { ...settings, panelWidth };
	}

	let appVersion = "";
	let projects: ProjectSummary[] = [];
	let projectsPath = "";

	// --- Açık proje ----------------------------------------------------------
	// `projectId` boşsa henüz kaydedilmemiş bir çalışma var demektir.
	let projectId = "";
	let projectName = "Yeni tema";
	let projectSource: string | null = null;
	/** Ana ekrandaki kart için kapak görseli — yalnızca ana ekranın "⋮" menüsünden değişir. */
	let coverImage: string | null = null;
	/**
	 * Editör'de o an gösterilecek bir tema var mı.
	 *
	 * `navigate()`'in "Editör'e açık tema olmadan girildi" dalını (Ayarlar'daki
	 * hızlı başlama seçicisi buradan tetikleniyor) tetikler mi diye bakıyor.
	 * Ana ekrana dönüldüğünde AŞAĞIDA `false`'a çekiliyor — aksi hâlde bir kez
	 * proje açıldıktan sonra bu bayrak oturum boyunca `true` kalır ve Editör'e
	 * her tıklama sessizce o projeyi geri açar; hızlı başlama ayarını
	 * DEĞİŞTİRSENİZ BİLE hiç devreye girmez, çünkü kapı zaten `!hasOpenProject`
	 * şartında kapalı kalır.
	 */
	let hasOpenProject = false;

	// Ana ekran = "açık tema yok" ekranı. Oraya her dönüşte bayrağı indiriyoruz
	// ki Editör'e bir sonraki giriş `settings.editorQuickStart` /
	// `settings.editorStartupAction`'ı YENİDEN değerlendirsin. `view`'ın hangi
	// yoldan "home" olduğu önemsiz (doğrudan `navigate`, kaydetmeden çık,
	// kaydedip çık) — hepsi buradan geçiyor.
	$: if (view === "home") hasOpenProject = false;
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

	// Tohumlama modu bilerek `doc.mode`'dan ayrı tutuluyor. Doğrudan `doc`'u
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

	/**
	 * Sıfırlamanın TABANI: düzenlenen şeyin "orijinali".
	 *
	 * Kural iki durumlu:
	 *   * Yeni tema oluşturuluyorsa taban SİTENİN kendi değerleri.
	 *   * Bir `.css` dosyası ya da GitHub içeriği açıldıysa taban O TEMANIN
	 *     içe aktarma anındaki değerleri.
	 *
	 * `null` iken taban site varsayılanı; içe aktarma sonrası bir anlık görüntü
	 * konuyor. Eskiden böyle bir kavram yoktu: kapalı bölümler ve sıfırlama
	 * düğmesi HER ZAMAN site varsayılanına dönüyordu, yani içe aktarılmış bir
	 * temada bir bölümü kapatıp açmak temanın değerlerini siliyordu. Kullanıcı
	 * yalnızca aç-kapa yapıyor, logosunun yazı tipi ve rengi gidiyordu.
	 */
	let advBaseline: AdvState | null = null;

	$: advBaselineOrSite = advBaseline ?? defaultAdv(seedMode, ramp);

	/** İçe aktarma sonrası tabanı, o anki kontrol durumundan sabitler. */
	function captureBaseline() {
		advBaseline = structuredClone(adv);
	}

	/** Yeni temaya dönüldüğünde taban yeniden sitenin kendi değerleri olur. */
	function clearBaseline() {
		advBaseline = null;
	}

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
			// Basit spread ({...eski, ...yeni}) eksik bir anahtarı SİLMEZ; bir
			// bölüm kapatılınca o anahtar `tokenMap`/`ruleMap`'ten kaybolur ama
			// spread eski değeri kalıcı bırakırdı ("kapat" hiçbir şey yapmamış
			// gibi görünürdü). Bu yüzden önce kontrollü anahtarların TAMAMI
			// siliniyor, sonra güncel harita üstüne yazılıyor — kapalı bir
			// bölümün anahtarı yeniden eklenmez, açık olanınki eklenir.
			if (nextTokens !== lastTokenMap) {
				lastTokenMap = nextTokens;
				const cleaned = { ...doc.tokenOverrides };
				for (const key of TOKEN_KEY_UNIVERSE) delete cleaned[key];
				doc.tokenOverrides = { ...cleaned, ...tokenMap };
			}
			// Kurallarda anahtarı komple silip yenisini yazmak DEĞİL, bildirim
			// bildirim birleştirme yapılıyor. Sebebi: bir selector'ın gövdesi
			// yalnızca kontrollerin ürettiklerinden ibaret olmayabilir. Harici
			// bir tema `.slider.orientation-horizontal`a `color`, `position`,
			// `block-size` gibi kontrollerde karşılığı olmayan bildirimler
			// yazıyor; gövdeyi komple değiştiren eski yol, kullanıcı hiçbir
			// şeye dokunmasa bile ilk yeniden üretimde onları siliyordu.
			if (nextRules !== lastRuleMap) {
				lastRuleMap = nextRules;
				doc.ruleOverrides = mergeRuleOverrides(
					doc.ruleOverrides,
					ruleMap,
					RULE_KEY_UNIVERSE,
					CONTROLLED_RULE_PROPS
				);
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

		// Kontroller, CSS'e YAZILAN token'ların yanında yalnızca gösterim için
		// taşınanları da okuyor (bkz. `theme.ts` -> `seedTokens`). Yazılanlar
		// üstte: bir değer ikisinde birden varsa geçerli olan odur.
		const seededTokens = { ...(next.seedTokens ?? {}), ...next.tokenOverrides };
		if (next.accent) {
			accentH = next.accent[0];
			accentS = next.accent[1];
			accentL = next.accent[2];
		}
		radiusEnabled = next.controlCornerRadius !== null || next.overlayCornerRadius !== null;
		if (next.controlCornerRadius !== null) controlRadius = next.controlCornerRadius;
		if (next.overlayCornerRadius !== null) overlayRadius = next.overlayCornerRadius;

		// Hover / buton renkleri: CSS değerini kontrole geri çöz.
		hoverEnabled = HOVER_TOKENS.some((s) => seededTokens[s.token] !== undefined);
		HOVER_TOKENS.forEach((spec, i) => {
			const parsed = fromCssColor(seededTokens[spec.token] ?? "");
			if (parsed) hoverColors[i] = { hex: parsed.hex, alpha: parsed.alpha };
		});
		hoverColors = hoverColors;

		buttonsEnabled =
			BUTTON_TOKENS.some((s) => seededTokens[s.token] !== undefined) ||
			next.ruleOverrides[BUTTON_TEXT_SELECTOR] !== undefined;
		BUTTON_TOKENS.forEach((spec, i) => {
			const parsed = fromCssColor(seededTokens[spec.token] ?? "");
			if (parsed) buttonColors[i] = { hex: parsed.hex, alpha: parsed.alpha };
		});
		buttonColors = buttonColors;

		const textRule = next.ruleOverrides[BUTTON_TEXT_SELECTOR];
		if (textRule) {
			const parsed = fromCssColor(textRule.replace(/^\s*color\s*:\s*/, "").replace(/;\s*$/, ""));
			if (parsed) buttonTextHex = parsed.hex;
		}
		// Animasyon: ölçeği "normal" süreden geri hesapla.
		motionEnabled = DURATION_TOKENS.some((d) => seededTokens[d.token] !== undefined);
		const normal = seededTokens["--fds-control-normal-duration"];
		if (normal) {
			const ms = Number(normal.replace(/ms\s*$/, ""));
			if (Number.isFinite(ms)) motionScale = Number((ms / 250).toFixed(2));
		}
		if (seededTokens[EASING_TOKEN]) motionEasing = seededTokens[EASING_TOKEN];

		// --- Renk bölümleri: temanın değerleriyle doldur ve AÇ -------------------
		//
		// Bunlar daha önce `adoptDoc`ta hiç ele alınmıyordu ve sonucu ikiydi:
		//
		//   1. Bölüm kapalı kaldığı için kontrolleri `disabled`'dı ve
		//      `buildAdvTokens` onlardan hiçbir şey yazmıyordu — kullanıcının
		//      "ayarı değiştiriyorum, önizleme ve CSS güncellenmiyor" dediği
		//      durum tam olarak buydu.
		//   2. Kapalıyken renkleri sürekli SİTE VARSAYILANINA geri yazılıyor
		//      (`AdvancedSections.svelte` -> `if (!adv.text.on) …`). Yani
		//      kullanıcı bölümü elle açtığında temanın renkleri değil site
		//      varsayılanları uygulanıyor, tema gözle görülür biçimde
		//      bozuluyordu.
		//
		// Ölçüt hover/buton bölümleriyle aynı: tema bu token'lardan en az
		// birini tanımlıyorsa bölüm açılır. Tanımlamıyorsa dokunulmaz —
		// temanın hiç ilgilenmediği bir bölümü açmak, site varsayılanlarını
		// gereksiz yere CSS'e yazmak olurdu.
		const seedColorSection = (
			specs: { token: string }[],
			section: { on: boolean; colors: { hex: string; alpha: number }[] }
		) => {
			if (!specs.some((spec) => seededTokens[spec.token] !== undefined)) return;

			let seeded = false;
			specs.forEach((spec, i) => {
				const parsed = fromCssColor(seededTokens[spec.token] ?? "");
				if (!parsed) return;
				section.colors[i] = { hex: parsed.hex, alpha: parsed.alpha };
				seeded = true;
			});

			// Hiçbir değer okunamadıysa bölümü AÇMIYORUZ. Açsaydık, kontroller
			// site varsayılanlarını taşıyor olurdu ve o varsayılanlar temanın
			// okuyamadığımız değerlerinin üstüne yazılırdı — yardım etmek
			// yerine temayı bozardık.
			if (seeded) section.on = true;
		};

		seedColorSection(TEXT_TOKENS, adv.text);
		seedColorSection(SURFACE_TOKENS, adv.surface);
		seedColorSection(LINK_TOKENS, adv.links);
		seedColorSection(SYSTEM_TOKENS, adv.system);

		// --- Avatar / banner / yorumlar ------------------------------------------
		// Bunlar token değil KURAL yazıyor; değerleri kuralın gövdesinden
		// çözülüyor. Aynı gerekçeyle bölüm yalnızca gerçekten okunabildiğinde
		// açılıyor.
		// Sayısal okumalar `resolveLength` üzerinden geçiyor: temalar bu
		// değerleri `var()` ardında ve `!important` ile veriyor, düz metin
		// araması hiçbirini yakalamıyordu (bkz. `cssDecls.ts`).
		const px = (rule: string | undefined, prop: string): number | null =>
			resolveLength(rule, prop, seededTokens);

		const avatarRule = next.ruleOverrides[AVATAR_SELECTOR];
		const avatarSize = px(avatarRule, "--fds-person-picture-size");
		if (avatarSize !== null) {
			adv.avatar.size = avatarSize;
			adv.avatar.on = true;
		}

		const bannerOutline = next.ruleOverrides[BANNER_SELECTED_SELECTOR]?.match(
			/outline-color\s*:\s*([^;!]+)/i
		);
		const bannerProgress = next.ruleOverrides[BANNER_PROGRESS_SELECTOR];
		if (bannerOutline || bannerProgress) {
			if (bannerOutline) adv.banner.outlineColor = bannerOutline[1].trim();
			const height = px(bannerProgress, "height");
			if (height !== null) adv.banner.progressHeight = height;
			const radius = px(bannerProgress, "border-radius");
			if (radius !== null) adv.banner.progressRadius = radius;
			const background = bannerProgress?.match(/background\s*:\s*([^;!]+)/i);
			if (background) adv.banner.progressColor = background[1].trim();
			adv.banner.on = true;
		}

		const commentRule = next.ruleOverrides[COMMENT_SELECTOR];
		const commentFocus =
			next.ruleOverrides[`${COMMENT_INPUT_SELECTOR}:focus-within`] ??
			next.ruleOverrides[COMMENT_INPUT_SELECTOR];
		if (commentRule || commentFocus) {
			const background = commentRule?.match(/background-color\s*:\s*([^;!]+)/i);
			const parsed = background ? fromCssColor(background[1]) : null;
			if (parsed) adv.comments.bg = { hex: parsed.hex, alpha: parsed.alpha };
			const radius = px(commentRule, "border-radius");
			if (radius !== null) adv.comments.radius = radius;
			const outline = commentFocus?.match(/outline\s*:\s*[^;]*?solid\s+([^;!]+)/i);
			if (outline) adv.comments.focusColor = outline[1].trim();
			adv.comments.on = true;
		}

		// --- Gelişmiş bölümler: tüm gelişmiş ayarları ve görselleri çöz -----------
		// Adres okuma `extractUrl` üzerinden: temalar logoyu gömülü bir SVG
		// olarak veriyor ve o değerin içinde boşluk da tırnak da var —
		// karakter sınıfına dayanan eski desen hiç eşleşmiyordu
		// (bkz. `cssDecls.ts` -> `extractUrl`).
		const resolveUrl = (rule: string | undefined, tokens: Record<string, string>): string => {
			if (!rule) return "";
			// 1. Doğrudan url(...) eşleşmesi
			const direct = extractUrl(rule);
			if (direct && !direct.startsWith("var(")) return direct;

			// 2. Kural içindeki var(--değişken) başvurusu
			const varMatches = Array.from(rule.matchAll(/var\(\s*(--[a-zA-Z0-9_-]+)/gi));
			for (const v of varMatches) {
				const varValue = tokens[v[1]];
				if (!varValue) continue;
				const inner = extractUrl(varValue);
				if (inner) return inner;
			}
			return "";
		};

		// Logo görselini kural, gizleme seçicisi veya öncelikli değişkenlerden çöz
		let logoImage = resolveUrl(next.ruleOverrides[LOGO_IMAGE_SELECTOR], seededTokens);
		if (!logoImage) {
			logoImage = resolveUrl(next.ruleOverrides[LOGO_IMAGE_HIDE_SELECTOR], seededTokens);
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
				const found = extractUrl(seededTokens[pVar] ?? "");
				if (found) {
					logoImage = found;
					break;
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

		// NEXT-GEN rozeti gizlenmiş mi? Temalar bunu rutin olarak yapıyor;
		// algılanmazsa kontrol kapalı görünür ve kullanıcı bir şeye
		// dokunduğunda rozet aniden geri gelirdi.
		adv.logo.badgeHidden = /display\s*:\s*none/i.test(
			next.ruleOverrides[LOGO_BADGE_SELECTOR] ?? ""
		);

		const logoRowRule = next.ruleOverrides[LOGO_ROW_SELECTOR] ?? "";
		const logoGap = resolveLength(logoRowRule, "gap", seededTokens);
		if (logoGap !== null) adv.logo.gap = logoGap;

		const logoImgRule = next.ruleOverrides[LOGO_IMAGE_SELECTOR] ?? "";
		const logoSize = resolveLength(logoImgRule, "width", seededTokens);
		if (logoSize !== null) adv.logo.size = logoSize;

		// Arkaplan görselini body::before, body::after, banner veya değişkenlerden çöz
		let bgImage = resolveUrl(next.ruleOverrides[`${BG_BODY_SELECTOR}::before`], seededTokens);
		if (!bgImage) {
			bgImage =
				resolveUrl(next.ruleOverrides[`${BG_BODY_SELECTOR}::after`], seededTokens) ||
				resolveUrl(next.ruleOverrides["body::before"], seededTokens) ||
				resolveUrl(next.ruleOverrides["body::after"], seededTokens) ||
				resolveUrl(next.ruleOverrides[BG_BODY_SELECTOR], seededTokens) ||
				resolveUrl(next.ruleOverrides[BG_TRANSPARENT_SELECTOR], seededTokens) ||
				resolveUrl(
					next.ruleOverrides[
						".scene-inner-content:has(.new-playlist)>.banner.gradient-scene"
					],
					seededTokens
				) ||
				resolveUrl(next.ruleOverrides[".banner.gradient-scene"], seededTokens);
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
				const found = extractUrl(seededTokens[pVar] ?? "");
				if (found) {
					bgImage = found;
					break;
				}
			}
		}
		adv.bg.dataUri = bgImage;
		adv.bg.on = bgImage !== "";

		for (const slot of MASCOT_SLOTS) {
			const image = resolveUrl(next.ruleOverrides[slot.selector], seededTokens);
			if (image) adv.mascot.images[slot.id] = image;
			else delete adv.mascot.images[slot.id];
		}

		// Kartlar
		const cardRule = next.ruleOverrides[CARD_SELECTOR];
		if (cardRule || seededTokens["--fds-card-background-default"]) {
			adv.cards.on = true;
			if (cardRule) {
				const cardRadius = resolveLength(cardRule, "border-radius", seededTokens);
				if (cardRadius !== null) adv.cards.radius = cardRadius;
				const cardBorder = resolveLength(cardRule, "border-width", seededTokens);
				if (cardBorder !== null) adv.cards.borderWidth = cardBorder;
			}
		}

		// Kenar çubuğu
		const sidebarRule = next.ruleOverrides[SIDEBAR_SELECTOR];
		if (sidebarRule || seededTokens[SIDEBAR_SELECTED_TOKEN.token]) {
			adv.sidebar.on = true;
			if (sidebarRule) {
				// Kenar çubuğu genişliği `rem`; temalar bunu da değişkenden
				// verebiliyor.
				const sidebarWidth = resolveLength(sidebarRule, "width", seededTokens, "rem");
				if (sidebarWidth !== null) adv.sidebar.width = sidebarWidth;
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

		// Gizleme ve yazı, renk anahtarından bağımsız algılanıyor: temalar
		// rozeti çoğunlukla renklerine dokunmadan kaldırıyor.
		const hidden = (selector: string) =>
			/display\s*:\s*none/i.test(next.ruleOverrides[selector] ?? "");

		adv.badges.badgeHidden = hidden(BADGE_SELECTOR);
		adv.badges.releasedHidden = hidden(RELEASED_BADGE_SELECTOR);
		adv.badges.enhancedHidden = hidden(ENHANCED_SELECTOR);

		/** `::after` kuralındaki `content: "..."` değerini geri okur. */
		const injectedText = (selector: string) => {
			const match = next.ruleOverrides[selector]?.match(
				/content\s*:\s*['"]((?:[^'"\\]|\\.)*)['"]/i
			);
			return match ? match[1].replace(/\\(["'\\])/g, "$1") : "";
		};

		adv.badges.badgeText = injectedText(BADGE_TEXT_SELECTOR);
		adv.badges.releasedText = injectedText(RELEASED_TEXT_SELECTOR);
		adv.badges.enhancedText = injectedText(ENHANCED_TEXT_SELECTOR);

		// Kaydırma çubuğu
		if (next.ruleOverrides[SCROLLBAR_SELECTOR]) {
			adv.scrollbar.on = true;
			const sbRule = next.ruleOverrides[SCROLLBAR_SELECTOR];
			const scrollbarSize = resolveLength(sbRule, "--os-size", seededTokens);
			if (scrollbarSize !== null) adv.scrollbar.size = scrollbarSize;
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
			seededTokens["--fds-font-family-text"] ||
			seededTokens["--fds-font-family-display"]
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

	let clearingPreviewData = false;
	let previewClearedStatus = "";

	/** Önizlemenin çerezlerini ve site verisini sıfırlar (bkz. `previewClearData`). */
	async function clearPreviewData() {
		clearingPreviewData = true;
		previewClearedStatus = "";
		try {
			await previewClearData();
			currentPath = settings.defaultPreviewPath;
			await refreshLoginState();
			previewClearedStatus = "Temizlendi.";
		} catch (e) {
			error = String(e);
		} finally {
			clearingPreviewData = false;
		}
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
			editorTab = "code";
			cssText = contents;
			adoptDoc(await importCssText(contents, KNOWN_SELECTORS));

			// Editöre dosyanın HAM metnini bırakmıyoruz; sistemin gerçekten
			// kullandığı metni yazıyoruz.
			//
			// Bırakıldığında üç şey birbirinden ayrışıyordu: editörde dosyanın
			// kendisi, `doc`ta çözümlenmiş belge, önizlemede o belgeden üretilen
			// CSS. Kullanıcının gördüğü "açtığım dosyanın bilgileri yanlış
			// geliyor, değiştirsem de işlenmiyor" tam olarak buydu — kod
			// editöründeki metin hiçbir şeyin kaynağı değildi.
			//
			// `push(true)` zorunlu: kod sekmesindeyken normal `push` kullanıcının
			// metnini KASTEN ezmiyor (yazarken imleç zıplamasın diye). Burada
			// ezmesi gerekiyor, çünkü henüz kullanıcının yazdığı bir şey yok.
			// GitHub'dan içe aktarma da aynı yoldan geçiyor (`handleImport`).
			await tick();
			await push(true);

			// Bu dosya artık "orijinal": kapalı bölümler ve sıfırlama düğmesi
			// site varsayılanına değil buraya dönecek.
			captureBaseline();

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

	// --- Vurgu rengi: palet ve HSL kaydırıcıları tek state paylaşır ---------
	// `doc.accent` (HSL) tek gerçek kaynak. `accentHex` ondan SALT türetilir
	// (aşağıdaki `$: accentHex = ...`) — ayrı, elle senkronlanan bir state
	// değil. Önceki sürümde `accentHex` kendi başına bir `let` idi ve
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
	 *
	 * `preserveMode`: `doc.mode`'u sıfırlamadan ÖNCEKİ hâliyle korur.
	 *
	 * Neden gerekli: Ayarlar -> "Düzenlenen temanın modu" kontrolü (bkz.
	 * `AppSettings.svelte` -> `onThemeModeChange`) açık proje olmadan da
	 * çalışıyor — kullanıcı bunu proje bazlı değil, önizlemenin genel bir
	 * tercihi olarak görüyor. Ama `createProject`/`handleImport` bu
	 * fonksiyonu ÇAĞIRIYOR ve `defaultDoc()` modu her zaman "dark" veriyor;
	 * `preserveMode` olmasaydı Ayarlar'da seçilen mod, kullanıcı Editör'e
	 * girip yeni/içe aktarılan tema oluşturulduğu an sessizce "Koyu"ya
	 * dönerdi. "Tümünü sıfırla" (`resetAll`) bilerek bunu kullanmıyor —
	 * o buton adı üstünde, modu da dahil HER ŞEYİ varsayılana döndürmeli.
	 */
	function resetState(preserveMode = false) {
		const mode = preserveMode ? doc.mode : defaultDoc().mode;
		// Doküman: accent, mod, yarıçaplar, import'lar, tüm ezmeler ve ham CSS.
		doc = { ...defaultDoc(), mode };
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

		// Artık düzenlenen bir tema yok: sıfırlamanın tabanı yeniden SİTENİN
		// kendi değerleri. İçe aktarma bu çağrıdan SONRA kendi tabanını
		// koyuyor (`captureBaseline`).
		clearBaseline();

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
	// `doc` temanın kendisi, `uiState` ise kontrollerin durumu. İkisini ayrı
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
		editorTab,
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
		advBaseline,
		viewport,
		currentPath
	} as EditorUiState;

	$: docSignature = { ...doc, mode: undefined };
	$: uiSignature = {
		...uiState,
		seedMode: undefined,
		viewport: undefined,
		currentPath: undefined,
		// Sekme değiştirmek projeyi "kaydedilmemiş" yapmamalı.
		editMode: undefined,
		editorTab: undefined
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
		// Eski projelerde `editorTab` yok; kaydedilmiş kipten türetiliyor.
		editorTab = ui.editorTab ?? tabFromMode(ui.editMode ?? settings.defaultEditMode);
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

		// Sıfırlamanın tabanı da projeyle birlikte geliyor. Taşınmasaydı
		// kaydedilmiş bir tema yeniden açıldığında taban sitenin
		// varsayılanlarına düşer ve ilk aç-kapada temanın değerleri silinirdi.
		// Alan yoksa (bu özellikten önce kaydedilmiş proje) taban sitedir.
		advBaseline = ui.advBaseline ?? null;

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
		coverImage = project.coverImage;
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

			// İmzayı bir tur sonra alıyoruz: `applyProject` içindeki yeniden
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
		// `true`: Ayarlar'da seçilen önizleme modu (bkz. `resetState` başlığı)
		// yeni temaya taşınsın, sessizce "Koyu"ya dönmesin.
		resetState(true);
		projectId = "";
		projectName = "Yeni tema";
		projectSource = null;
		coverImage = null;
		externalPath = null;
		externalDirty = false;
		fileStatus = "";
		projectStatus = "";

		editorTab = tabFromMode(settings.defaultEditMode);
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
				source: projectSource,
				coverImage
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
	// --- Editöre başlama seçicisi -------------------------------------------
	//
	// Açık tema yokken Editör'e girildiğinde çıkar. Dört yol da ana ekrandakiyle
	// (ya da, mevcut temada, ana ekranın "Kayıtlı temalar" listesiyle) AYNI
	// işleyicilere bağlı; burada ikinci bir uygulama yolu yok.
	let starterOpen = false;
	let githubImportOpen = false;

	/**
	 * "Mevcut bir temayı düzenle" — seçiciyi kaydırma listesine çeviren adım.
	 *
	 * Diyalog her kapandığında (hangi yoldan olursa olsun: bir eylem seçildi,
	 * "Geri" ya da "Vazgeç") sıfırlanıyor; aksi hâlde bir sonraki açılışta
	 * kullanıcı üç düğme yerine bıraktığı yerdeki proje listesini görürdü.
	 */
	let pickingExisting = false;
	$: if (!starterOpen) pickingExisting = false;

	function startNew() {
		starterOpen = false;
		createProject();
	}

	/**
	 * GitHub'ın kendi içe aktarma diyaloğunu doğrudan açar.
	 *
	 * Araya "bağlantıyı gir" diye ikinci bir diyalog konmuyor: o diyalog zaten
	 * bağlantıyı, dosya seçimini ve proje adını soruyor
	 * (`GithubImportDialog.svelte`). İkincisi yalnızca bir tıklama fazlası
	 * olurdu.
	 */
	function startFromGithub() {
		starterOpen = false;
		githubImportOpen = true;
	}

	function startFromFile() {
		starterOpen = false;
		openFileFromHome();
	}

	function startFromExisting(id: string) {
		starterOpen = false;
		openProject(id);
	}

	/** "Oluşturma" tarihi — `formatUpdated` gibi göreli değil, MUTLAK tarih.
	 * Ne zaman oluşturulduğunu bilmek isteyen biri için "3 ay önce" değil
	 * gerçek tarih anlamlı; göreli biçim zaten "En son güncelleme" satırında
	 * kullanılıyor, ikisi aynı olsaydı satırlar birbirinin tekrarı olurdu. */
	function formatCreated(ms: number): string {
		if (!ms) return "";
		return new Date(ms).toLocaleDateString("tr-TR", { day: "numeric", month: "short", year: "numeric" });
	}

	async function navigate(next: NavId) {
		if (next === view) return;

		// Editör açık bir tema gerektirir. Hangi eylemin çalışacağı
		// `settings.editorStartupAction`'a bağlı (bkz. Ayarlar -> Tema editörü);
		// "ask" dışındaki üç seçenek seçiciyi hiç göstermeden doğrudan o eylemi
		// başlatıyor.
		if (next === "editor" && !hasOpenProject) {
			// Hızlı başlama kapalıysa (varsayılan) ya da açık olup "Her zaman
			// sor" seçiliyse davranış aynı: seçici çıkar.
			const action = settings.editorQuickStart ? settings.editorStartupAction : "ask";
			switch (action) {
				case "new":
					startNew();
					break;
				case "github":
					startFromGithub();
					break;
				case "file":
					startFromFile();
					break;
				default:
					starterOpen = true;
			}
			return;
		}

		if (view === "editor" && dirty) {
			// Otomatik kaydetme açıksa hiç sormuyoruz — proje diskteyse
			// sessizce kaydedip geçiyoruz, henüz hiç kaydedilmemişse (ad yok)
			// tek eksik bilgiyi (adı) soruyoruz ve devam ediyoruz. İkisinde de
			// "kaydedilsin mi?" sorusu çıkmıyor; kullanıcı bu ayarı zaten o
			// soruyu istemediği için açtı.
			if (settings.autoSaveOnLeave) {
				if (projectId) {
					persistProject().then(() => (view = next));
				} else {
					askName(() => (view = next));
				}
				return;
			}

			// Kapalıyken ne yapılacağını soruyoruz; "Kaydet" derse ad kutusu
			// (gerekiyorsa) oradan açılır.
			pendingView = next;
			confirmLeave = true;
			return;
		}

		view = next;
	}

	/**
	 * Başlık çubuğundaki geri oku (yalnızca Editör'de görünür — bkz.
	 * `TitleBar` -> `onBack`). `navigate("home")` ile aynı: kaydedilmemiş
	 * değişiklik varsa kaydet/çık sorusu çıkar, otomatik kaydetme açıksa
	 * sessizce kaydedip döner.
	 */
	function backToHome() {
		navigate("home");
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

	/** Ana ekrandaki "⋮" menüsünden: kapak görseli seç/değiştir. */
	async function handleSetCover(id: string) {
		try {
			const dataUri = await pickImage();
			if (!dataUri) return;
			const project = await loadProject(id);
			project.coverImage = dataUri;
			await saveProject(project);
			if (id === projectId) coverImage = dataUri;
			await refreshProjects();
		} catch (e) {
			error = String(e);
		}
	}

	/** Ana ekrandaki "⋮" menüsünden: kapak görselini kaldırıp varsayılan önizlemeye dön. */
	async function handleRemoveCover(id: string) {
		try {
			const project = await loadProject(id);
			project.coverImage = null;
			await saveProject(project);
			if (id === projectId) coverImage = null;
			await refreshProjects();
		} catch (e) {
			error = String(e);
		}
	}

	/** Ana ekrandaki "⋮" menüsünden: projeyi editöre girmeden tek seferlik .css olarak dışa aktarır. */
	async function handleExportCss(id: string) {
		try {
			const project = await loadProject(id);
			const chosen = await saveDialog({
				defaultPath: `${project.name}.css`,
				filters: [{ name: "CSS", extensions: ["css"] }]
			});
			if (typeof chosen !== "string") return;
			await writeCssFile(chosen, project.cssText);
			projectStatus = `"${chosen}" kaydedildi.`;
			error = "";
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
			// `true`: Ayarlar'daki önizleme modu, içe aktarılan CSS modu
			// belirtmiyorsa korunsun.
			resetState(true);
			await tick();

			const next = await importCssText(payload.css, KNOWN_SELECTORS);
			adoptDoc(next);

			projectId = "";
			projectName = payload.name;
			projectSource = payload.source;
			coverImage = null;
			externalPath = null;
			externalDirty = false;
			editorTab = "basic";
			viewport = settings.defaultViewport;
			currentPath = settings.defaultPreviewPath;

			await tick();
			await push(true);

			// İçe aktarılan tema artık "orijinal": kapalı bölümler ve sıfırlama
			// düğmesi site varsayılanına değil buraya döner.
			captureBaseline();

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
		coverImage = null;
		editorTab = tabFromMode(settings.defaultEditMode);
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
	 * Kısa bir bekleme var, çünkü köprü çerezleri sildikten sonra önizlemeyi
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
	// zorunda; listelenmezse diyalog açılır ama önizlemenin altında kalır.
	$: modalOpen =
		confirmLeave ||
		namingOpen ||
		confirmResetAll ||
		aboutDialogOpen ||
		updateDialogOpen ||
		loginDialogOpen ||
		starterOpen ||
		githubImportOpen;
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
	// çıktığında bırak. Slot yalnızca Editör'deyken var olduğu için bileşenin
	// yaşam döngüsüne değil elemanın varlığına bağlanıyor.
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
	<TitleBar title={titleContext} onBack={view === "editor" ? backToHome : null} />

	<div class="body">
		<NavRail
			current={view}
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
				onSetCover={handleSetCover}
				onRemoveCover={handleRemoveCover}
				onExportCss={handleExportCss}
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
				themeMode={doc.mode}
				onThemeModeChange={setMode}
				onCheckForUpdates={() => runUpdateCheck(true)}
				{updateCheckStatus}
				{updateCheckError}
				{updateChannelLabel}
				{onChannelChange}
			/>
		{:else}
			<!-- Editör: mevcut panel + önizleme yerleşimi olduğu gibi korunuyor. -->
			<div class="shell" style="grid-template-columns: {panelWidth}px 6px 1fr">
	<div class="panel" use:unclip>
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
				<Tooltip text="Temayı ayrı bir .css dosyası olarak diske yazar — proje kaydından bağımsız.">
					<Button
						on:click={async () => {
							await saveExternal(true);
							// `fileStatus`'un kendi StatusBar'ı yalnızca Kod modunda görünür;
							// Görsel'den tetiklendiğinde onay burada gösterilsin diye
							// `projectStatus`'a da yansıtıyoruz (ikisi de her modda görünür).
							if (fileStatus) projectStatus = fileStatus;
						}}
					>
						<Icon name="code" size={14} /><span class="gap">CSS olarak kaydet…</span>
					</Button>
				</Tooltip>
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
			Tek anahtar, üç sekme. Gerekçesi `editorTab`'in başındaki notta.

			`on:click` burada gereksiz görünebilir: SegmentedControl zaten
			bind:value ile değeri kendisi yazıyor. Ama o mekanizma DOM'da
			`closest("[data-segment-id]")` araması ve modül düzeyinde paylaşılan
			bir sözlük üzerinden çalışıyor; kritik yolda tek dayanak olmasın diye
			doğrudan atama da yapıyoruz. İkisi aynı değeri yazdığı için çakışmaz.
		-->
		<div class="mode-switch">
			<SegmentedControl bind:value={editorTab}>
				<SegmentedControlButton value="basic" on:click={() => (editorTab = "basic")}>
					Temel
				</SegmentedControlButton>
				<SegmentedControlButton value="all" on:click={() => (editorTab = "all")}>
					Tümü
				</SegmentedControlButton>
				<SegmentedControlButton value="code" on:click={() => (editorTab = "code")}>
					Kod
				</SegmentedControlButton>
			</SegmentedControl>
		</div>

		{#if editorTab !== "code"}
			<div class="sections">
				<!--
					Bölümlerin sırası ve grup başlıkları `AdvancedSections`'ta;
					bu dosyanın kendi bölümleri oraya yuvalardan giriyor. Tek bir
					`AdvancedSections` örneği var ve olmak zorunda: bileşen, kapalı
					bölümlerin varsayılanlarını tazeleyen reaktif ifadeler taşıyor
					ve iki örnek aynı `adv`ye yazsaydı birbirlerini tetikleyip
					döngüye girerlerdi.
				-->
				<AdvancedSections
					bind:adv
					{pickImage}
					baseline={advBaselineOrSite}
					show={showSection}
					grouped={editorTab === "all"}
				>
					<svelte:fragment slot="accent">
						{#if showSection("accent")}
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
						{/if}
					</svelte:fragment>
					<svelte:fragment slot="shape">
						{#if showSection("radius")}
							<Section icon="corner" title="Köşe yumuşaklığı" expanded onReset={() => resetSection("radius")}>
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
						{/if}
						{#if showSection("motion")}
							<Section icon="motion" title="Hareket ve geçişler" expanded onReset={() => resetSection("motion")}>
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
						{/if}
					</svelte:fragment>
					<svelte:fragment slot="interaction">
						{#if showSection("buttons")}
							<Section icon="button" title="Düğme renkleri" onReset={() => resetSection("buttons")}>
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
						{/if}
						{#if showSection("hover")}
							<Section icon="hover" title="Fare üzerindeyken" onReset={() => resetSection("hover")}>
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
						{/if}
					</svelte:fragment>
					<svelte:fragment slot="raw">
						{#if showSection("raw")}
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
					</svelte:fragment>
				</AdvancedSections>

				{#if editorTab === "basic"}
					<TextBlock variant="caption" class="text-secondary">
						Burada en çok kullanılan beş ayar var. Geri kalan her şey için “Tümü”
						sekmesine geçin.
					</TextBlock>
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

				<Tooltip
					text="Önizlemedeki çerezleri (oturum dâhil) ve site verilerini (localStorage, önbellek) siler, sayfayı baştan yükler. Temanız etkilenmez."
				>
					<Button on:click={clearPreviewData} disabled={clearingPreviewData}>
						<Icon name="reset" size={14} /><span class="gap">
							{clearingPreviewData ? "Temizleniyor…" : "Çerezleri ve verileri sıfırla"}
						</span>
					</Button>
				</Tooltip>
				{#if previewClearedStatus}
					<TextBlock variant="caption" class="text-secondary">{previewClearedStatus}</TextBlock>
				{/if}
			</Section>

		</div>
	</div>

	<!-- svelte-ignore a11y-no-static-element-interactions -->
	<div
		class="panel-resize-handle"
		class:dragging={panelResizing}
		role="separator"
		aria-orientation="vertical"
		aria-label="Paneli yeniden boyutlandır"
		on:pointerdown={startPanelResize}
		on:dblclick={resetPanelWidth}
	></div>

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
<!-- --- Editöre başlama seçicisi ----------------------------------------- -->
<ContentDialog
	bind:open={starterOpen}
	title={pickingExisting ? "Hangi tema?" : "Ne yapmak istersiniz?"}
	size="standard"
>
	<div class="starter">
		{#if !pickingExisting}
			<TextBlock variant="caption" class="text-secondary">
				Devam etmek için bir tema gerekiyor. Nasıl başlamak istersiniz?
			</TextBlock>

			<Button variant="accent" on:click={startNew}>
				<Icon name="add" size={16} /><span class="gap">Yeni tema oluştur</span>
			</Button>
			<Button on:click={startFromGithub}>
				<Icon name="github" size={16} /><span class="gap">GitHub'dan içe aktar</span>
			</Button>
			<Button on:click={startFromFile}>
				<Icon name="open" size={16} /><span class="gap">CSS dosyası aç…</span>
			</Button>

			{#if projects.length}
				<Button on:click={() => (pickingExisting = true)}>
					<Icon name="navEditor" size={16} /><span class="gap"
						>Mevcut bir temayı düzenle ({projects.length})</span
					>
				</Button>
			{/if}
		{:else}
			<!-- Ana ekrandaki kart ızgarasının küçültülmüş hâli değil, tek sütunlu
			     bir liste: bu diyalog dar (size="standard") ve kartların önizleme
			     şeridi burada yer kaplayan, katkısı düşük bir ayrıntı olurdu. -->
			<div class="starter-projects">
				{#each projects as project (project.id)}
					<button
						type="button"
						class="starter-project"
						on:click={() => startFromExisting(project.id)}
					>
						<TextBlock variant="bodyStrong">{project.name}</TextBlock>
						<TextBlock variant="caption" class="text-secondary">
							Oluşturma: {formatCreated(project.createdAt)} · En son güncelleme: {formatUpdated(
								project.updatedAt
							)}
						</TextBlock>
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<svelte:fragment slot="footer">
		{#if pickingExisting}
			<Button on:click={() => (pickingExisting = false)}>Geri</Button>
		{/if}
		<Button on:click={() => (starterOpen = false)}>Vazgeç</Button>
	</svelte:fragment>
</ContentDialog>

<GithubImportDialog bind:open={githubImportOpen} onImport={handleImport} />

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
				// `askName` bir yolculuk sırasında (ör. "Ana ekran"a dönerken) adı
				// sormak için açılmış olabilir — o zaman `afterSave` bekleyen
				// navigasyonu taşır. Yalnızca diyaloğu kapatıp kullanıcıyı editörde
				// TAKILI bırakmak yerine (autoSaveOnLeave açıkken tekrar "Ana
				// ekran"a basınca aynı diyalog sonsuza dek yeniden açılırdı), o
				// navigasyonu kaydetmeden çalıştırıyoruz: "Vazgeç" gerçekten
				// vazgeçsin.
				const next = afterSave;
				afterSave = null;
				next?.();
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
		/* `grid-template-columns` satır içi stille (bkz. yukarısı) `panelWidth`e
		   bağlanıyor; buradaki değer yalnızca stil uygulanmadan önceki karede
		   görünecek geçici bir varsayılan. */
		grid-template-columns: 420px 6px 1fr;
		min-height: 0;
		overflow: hidden;
	}

	/* Panel/önizleme arasındaki sürüklenebilir ayraç. Genişlik `panelWidth`e
	   yazılıyor; webview'in yeniden konumlanması zaten `previewSlot`'u izleyen
	   `ResizeObserver`'dan (syncBounds) kendiliğinden geliyor, burada ekstra
	   bir şey tetiklemeye gerek yok. */
	.panel-resize-handle {
		position: relative;
		cursor: col-resize;
		background-color: var(--fds-divider-stroke-default);
		touch-action: none;
	}

	.panel-resize-handle::after {
		/* Görünen çizgi 1px kalsın ama tıklama/sürükleme hedefi tüm 6px'lik
		   sütunu kaplasın — 1px'e nişan almak zorunda bırakmıyoruz. */
		content: "";
		position: absolute;
		inset: 0 -3px;
	}

	.panel-resize-handle:hover,
	.panel-resize-handle.dragging {
		background-color: var(--fds-accent-default);
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

	/* Seçici: üç yol alt alta ve eşit genişlikte. Yan yana dizilseydi en uzun
	   etiket ("CSS dosyası aç…") diğerlerini ezer, hangisinin birincil yol
	   olduğu da kaybolurdu. */
	.starter {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.starter :global(.button) {
		justify-content: flex-start;
	}

	/* Proje listesi: diyalog dar olduğu için ana ekrandaki kart ızgarası değil,
	   satır satır bir liste — her satır tek başına tıklanabilir bir düğme. */
	.starter-projects {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 320px;
		overflow-y: auto;
	}

	.starter-project {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		width: 100%;
		box-sizing: border-box;
		padding: 8px 12px;
		border: none;
		border-radius: var(--fds-control-corner-radius);
		background: transparent;
		cursor: pointer;
		text-align: left;
		transition: background-color var(--fds-control-fast-duration) ease;
	}

	.starter-project:hover {
		background-color: var(--fds-subtle-fill-secondary);
	}

	.starter-project:active {
		background-color: var(--fds-subtle-fill-tertiary);
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
