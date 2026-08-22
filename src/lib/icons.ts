// Fluent System Icons (@fluentui/svg-icons) — fluent-svelte-extra'nın kendi
// dokümantasyon sitesinde kullandığı ikon paketi, yani tasarım dili birebir
// uyumlu. Paket 20 000'den fazla SVG içeriyor; burada yalnızca kullandığımız
// birkaç tanesini `?raw` ile statik olarak alıyoruz ki bundle şişmesin.
//
// SVG'lerde `fill` özniteliği yok — dolayısıyla `currentColor` miras alıyorlar
// ve --fds-* metin renkleriyle kendiliğinden uyumlu oluyorlar.

import appearance from "@fluentui/svg-icons/icons/dark_theme_24_regular.svg?raw";
import accent from "@fluentui/svg-icons/icons/color_24_regular.svg?raw";
import corner from "@fluentui/svg-icons/icons/border_all_24_regular.svg?raw";
import hover from "@fluentui/svg-icons/icons/cursor_hover_24_regular.svg?raw";
import motion from "@fluentui/svg-icons/icons/top_speed_24_regular.svg?raw";
import brand from "@fluentui/svg-icons/icons/image_24_regular.svg?raw";
import button from "@fluentui/svg-icons/icons/cursor_click_24_regular.svg?raw";
import code from "@fluentui/svg-icons/icons/code_24_regular.svg?raw";
import page from "@fluentui/svg-icons/icons/globe_24_regular.svg?raw";
import viewport from "@fluentui/svg-icons/icons/phone_desktop_24_regular.svg?raw";
import reset from "@fluentui/svg-icons/icons/arrow_reset_24_regular.svg?raw";
import background from "@fluentui/svg-icons/icons/wallpaper_24_regular.svg?raw";
import card from "@fluentui/svg-icons/icons/card_ui_24_regular.svg?raw";
import typography from "@fluentui/svg-icons/icons/text_font_size_24_regular.svg?raw";
import mascot from "@fluentui/svg-icons/icons/emoji_24_regular.svg?raw";
import player from "@fluentui/svg-icons/icons/play_circle_24_regular.svg?raw";
import focus from "@fluentui/svg-icons/icons/square_hint_24_regular.svg?raw";
import file from "@fluentui/svg-icons/icons/document_24_regular.svg?raw";
import open from "@fluentui/svg-icons/icons/folder_open_24_regular.svg?raw";
import openExternal from "@fluentui/svg-icons/icons/open_24_regular.svg?raw";
// Sitenin başlık çubuğundaki "geri" oku — `fluent:arrow-left-24-regular`
// (bundle'da doğrulandı, bkz. TitleBar.svelte -> `onBack`).
import back from "@fluentui/svg-icons/icons/arrow_left_24_regular.svg?raw";
import save from "@fluentui/svg-icons/icons/save_24_regular.svg?raw";
import update from "@fluentui/svg-icons/icons/arrow_sync_24_regular.svg?raw";
import refresh from "@fluentui/svg-icons/icons/arrow_clockwise_24_regular.svg?raw";
// Discord'un kendi logosu. Fluent setinde marka glifleri yok ve yerine konan
// jenerik bir ikon (denenen: "topluluk") ayarın hangi servise ait olduğunu
// anlatmıyordu. Bu SVG, OpenAnime-Desktops'ın Discord ayar arayüzünde
// kullanılanın aynısı — iki uygulama aynı ikonu göstersin diye.
//
// `fill="currentColor"` içeride tanımlı, yani diğer ikonlar gibi --fds-*
// metin rengini miras alıyor (bkz. `Icon.svelte`).
import discord from "$lib/assets/discord.svg?raw";

// Gelişmiş özelleştirme seçenekleri ikon tanımları.
import sidebar from "@fluentui/svg-icons/icons/panel_left_24_regular.svg?raw";
import surface from "@fluentui/svg-icons/icons/layer_24_regular.svg?raw";
import link from "@fluentui/svg-icons/icons/link_24_regular.svg?raw";
import scrollbar from "@fluentui/svg-icons/icons/line_horizontal_1_24_regular.svg?raw";
import badge from "@fluentui/svg-icons/icons/ribbon_24_regular.svg?raw";
import avatar from "@fluentui/svg-icons/icons/person_circle_24_regular.svg?raw";
import banner from "@fluentui/svg-icons/icons/image_multiple_24_regular.svg?raw";
import comments from "@fluentui/svg-icons/icons/comment_24_regular.svg?raw";
import system from "@fluentui/svg-icons/icons/warning_24_regular.svg?raw";
import palette from "@fluentui/svg-icons/icons/color_fill_24_regular.svg?raw";
import size from "@fluentui/svg-icons/icons/resize_24_regular.svg?raw";

import weatherSunny from "@fluentui/svg-icons/icons/weather_sunny_24_regular.svg?raw";
import weatherMoon from "@fluentui/svg-icons/icons/weather_moon_24_regular.svg?raw";
import settingsSystem from "@fluentui/svg-icons/icons/system_24_regular.svg?raw";

// Sıfırlama: bölüm başlıklarındaki tekil sıfırlama ile "tümünü sıfırla"
// bilerek FARKLI glifler — biri geri alır, diğeri her şeyi siler.
import resetAll from "@fluentui/svg-icons/icons/arrow_counterclockwise_24_regular.svg?raw";

// --- Ana ekran gezinmesi ----------------------------------------------------
//
// Bu ikonlar ve regular/filled ikilisi keyfi değil: openani.me'nin kendi
// kenar çubuğu tam olarak bu seti kullanıyor ve seçili öğede son eki
// çalışma anında değiştiriyor (`icon + (selected ? "filled" : "regular")`,
// canlı bundle'dan çıkarıldı). Aynı kuralı burada da uyguluyoruz ki uygulama
// sitenin devamı gibi hissettirsin.
import navHome from "@fluentui/svg-icons/icons/home_24_regular.svg?raw";
import navHomeOn from "@fluentui/svg-icons/icons/home_24_filled.svg?raw";
import navLibrary from "@fluentui/svg-icons/icons/library_24_regular.svg?raw";
import navLibraryOn from "@fluentui/svg-icons/icons/library_24_filled.svg?raw";
import navEditor from "@fluentui/svg-icons/icons/paint_brush_24_regular.svg?raw";
import navEditorOn from "@fluentui/svg-icons/icons/paint_brush_24_filled.svg?raw";
import navSettings from "@fluentui/svg-icons/icons/settings_24_regular.svg?raw";
import navSettingsOn from "@fluentui/svg-icons/icons/settings_24_filled.svg?raw";
import navAbout from "@fluentui/svg-icons/icons/info_24_regular.svg?raw";
import navAboutOn from "@fluentui/svg-icons/icons/info_24_filled.svg?raw";

// --- Durum kutuları ---------------------------------------------------------
//
// InfoBar'ın kendi varsayılan ikonu `InfoBadge`: 16px'lik renkli bir hap ve
// içinde 8px'lik çıplak bir glif (düz onay işareti, çarpı, ünlem). Bunlar
// Fluent System Icons setinden değil. Sitenin durum kutularında kullandığı
// 20px'lik daireli ikonların karşılıkları bunlar; `StatusBar` her önem
// derecesine doğru olanı bağlıyor.
//
// 20'lik ızgaradan geliyorlar (24'lük değil): bu ikonlar 20px çizilecek ve
// Fluent'te her ölçünün kendi optik olarak düzeltilmiş çizimi var.
import statusSuccess from "@fluentui/svg-icons/icons/checkmark_circle_20_regular.svg?raw";
import statusInfo from "@fluentui/svg-icons/icons/info_20_regular.svg?raw";
import statusCaution from "@fluentui/svg-icons/icons/warning_20_regular.svg?raw";
import statusCritical from "@fluentui/svg-icons/icons/error_circle_20_regular.svg?raw";

// Boş durum ekranı. 48'lik ızgara: boş durum ikonları 48–64px çiziliyor ve
// 24'lük glifi büyütmek çizgi kalınlığını orantısız bırakırdı.
import emptyThemes from "@fluentui/svg-icons/icons/folder_48_regular.svg?raw";

// --- Proje yönetimi ---------------------------------------------------------
import add from "@fluentui/svg-icons/icons/add_24_regular.svg?raw";
import github from "@fluentui/svg-icons/icons/branch_fork_24_regular.svg?raw";
import download from "@fluentui/svg-icons/icons/cloud_arrow_down_24_regular.svg?raw";
import remove from "@fluentui/svg-icons/icons/delete_24_regular.svg?raw";
import rename from "@fluentui/svg-icons/icons/rename_24_regular.svg?raw";
import more from "@fluentui/svg-icons/icons/more_vertical_24_regular.svg?raw";
import person from "@fluentui/svg-icons/icons/person_24_regular.svg?raw";
import plus from "@fluentui/svg-icons/icons/star_24_regular.svg?raw";

// Görünüm (viewport) seçicisi — önizlemenin üstündeki floating kontrol.
import mobile from "@fluentui/svg-icons/icons/phone_24_regular.svg?raw";
import tablet from "@fluentui/svg-icons/icons/tablet_24_regular.svg?raw";
import desktop from "@fluentui/svg-icons/icons/desktop_24_regular.svg?raw";

// Başlık çubuğu kontrolleri. Bunlar 16'lık ızgaradan geliyor (yukarıdakiler
// 24'lük): pencere düğmeleri küçük çizilir ve Fluent'in kendi başlık çubuğu
// glifleri de 16'lık sette. `restore` için ayrı bir dosya yok; Fluent'in
// "geri al" gliflerinin karşılığı üst üste iki kare olan `square_multiple`.
import minimize from "@fluentui/svg-icons/icons/subtract_16_regular.svg?raw";
import maximize from "@fluentui/svg-icons/icons/maximize_16_regular.svg?raw";
import restore from "@fluentui/svg-icons/icons/square_multiple_16_regular.svg?raw";
import close from "@fluentui/svg-icons/icons/dismiss_16_regular.svg?raw";

export const ICONS = {
	appearance,
	accent,
	corner,
	hover,
	motion,
	brand,
	button,
	code,
	page,
	viewport,
	reset,
	background,
	card,
	typography,
	mascot,
	player,
	focus,
	file,
	open,
	openExternal,
	back,
	save,
	update,
	refresh,
	sidebar,
	surface,
	link,
	scrollbar,
	badge,
	avatar,
	banner,
	comments,
	system,
	palette,
	size,
	weatherSunny,
	weatherMoon,
	settingsSystem,
	resetAll,
	mobile,
	tablet,
	desktop,
	minimize,
	maximize,
	restore,
	close,
	navHome,
	navHomeOn,
	navLibrary,
	navLibraryOn,
	navEditor,
	navEditorOn,
	navSettings,
	navSettingsOn,
	navAbout,
	navAboutOn,
	statusSuccess,
	statusInfo,
	statusCaution,
	statusCritical,
	emptyThemes,
	add,
	github,
	download,
	remove,
	rename,
	more,
	person,
	plus,
	discord
} as const;

export type IconName = keyof typeof ICONS;
