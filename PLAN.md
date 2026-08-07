# OpenAnime Tema Editörü — Mimari ve Uygulama Planı

> Bu plandaki her teknik iddia, openani.me'nin canlı CSS/JS bundle'ları ve
> fluent-svelte-extra kaynak kodu incelenerek **doğrulanmıştır**. Varsayım yoktur.
> Doğrulama tarihi: 2026-08-03.

---

## 0. Doğrulanmış Bulgular

Planın tamamı bu bulgulara dayanıyor. En kritik olanı 0.3'tür — site zaten bizim
için tasarlanmış resmi bir tema giriş noktası expose ediyor.

### 0.1 Site, fluent-svelte-extra'nın tema sistemini birebir kullanıyor

`openani.me` 52 adet CSS chunk yüklüyor. Bu chunk'lardaki custom property
tanımları tarandığında, fluent-svelte-extra'nın **hem `theme.css` hem
`switchable.css`** dosyalarının yüklendiği görülüyor. Her tema token'ı tam
4 yerde tanımlı:

| # | Selector | Kaynak |
|---|----------|--------|
| 1 | `@media (prefers-color-scheme: light) :root` | `theme.css` |
| 2 | `@media (prefers-color-scheme: dark) :root`  | `theme.css` |
| 3 | `.fds-theme-light`                            | `switchable.css` |
| 4 | `.fds-theme-dark`                             | `switchable.css` |

Mod-bağımsız token'lar (`--fds-accent-light-3` … `--fds-accent-dark-3`,
font aileleri, font boyutları, `--fds-control-corner-radius`,
`--fds-overlay-corner-radius`, süreler, easing, acrylic asset'leri) yalnızca
`:root` içinde **bir kez** tanımlı.

**Sonuç:** Site kendi tema değişkeni icat etmemiş. `--fds-*` seti bizim resmi
giriş noktamız. Bu, "kullanıcıya sadece resmi giriş noktaları sun" kısıtını
doğrudan karşılıyor.

### 0.2 Site-özel (fds dışı) değişkenler — resmi API değil

Bundle'larda şu fds-dışı token'lar da var: `--gray1`…`--gray12`
(`[data-sonner-toaster]` scope'unda), `--explore-columns` / `--explore-card-width`
(`#page.svelte-1dl8t1s` — **Svelte hash'li**, her deploy'da değişir),
`--os-*` (overlayscrollbars), `--normal/info/success/warning/error-*` (toast),
`--glow-rotation`, `--lift`, `--mask-*`.

**Sonuç:** Bunlar Svelte scoped-class hash'lerine bağlı olduğu için kararlı API
değil. Editörde varsayılan olarak **gösterilmeyecek**; en fazla "Gelişmiş
(kırılgan)" başlığı altında, uyarı ile.

### 0.3 ⭐ Sitenin resmi tema enjeksiyon noktası: `localStorage.theme_content`

`openanime-C4wWGNpH.js` chunk'ında, root layout boot kodunda (deminified):

```js
const themeMode    = +localStorage.getItem("theme");          // 0=system, 1=light, 2=dark
const themeContent =  localStorage.getItem("theme_content");  // ham CSS string

if (!isNative) {
    if      (themeMode == 0) document.documentElement.className = "";
    else if (themeMode == 1) document.documentElement.className = "fds-theme-light";
    else if (themeMode == 2) document.documentElement.className = "fds-theme-dark";
} else {
    window.NativeApp.setTheme(themeMode == 0 ? "system" : themeMode == 1 ? "light" : "dark");
}

if (themeContent) {
    let s = document.createElement("style");
    s.type = "text/css";
    s.setAttribute("themeStyle", "true");
    s.innerText = themeContent;
    document.head.appendChild(s);   // ← TÜM bundle stylesheet'lerinden SONRA
}
```

Bu kod `theme_content`'i **yalnızca okuyor**. Sitenin hiçbir yerinde
`setItem("theme_content")` çağrısı yok — yani bu, harici araçlar için bırakılmış
bir uzantı noktası.

**Sonuç — projenin tüm mimarisi buradan çıkıyor:**
- **Dışa aktarma formatı belli:** düz bir CSS string. Kullanıcı bunu
  `localStorage.theme_content`'e yazınca site temalı açılır. `.css` dosyası
  olarak da aynı içerik.
- **Canlı önizleme yolu belli:** aynı `<style themeStyle="true">` elementinin
  `textContent`'ini güncellemek. Reload yok, Svelte re-render yok.
- **WYSIWYG garantisi:** önizleme ve export **aynı CSS string'i** kullanır.
  Tek kod yolu ⇒ önizlemede gördüğün ile export ettiğin arasında sapma imkânsız.

### 0.4 ⭐ İkinci resmi giriş noktası: `window.NativeApp` köprüsü

`openanime-kOXkfm7I.js` içinde (deminified):

```js
try {
    if (window.NativeApp.fetchInfo().isNative) {
        const info = window.NativeApp.fetchInfo();
        isNativeStore.set(true);
        osStore.set(info.os);
        document.body.classList.add("native-app");
        for (const key in info.theme)
            if (key.includes("accent"))
                document.documentElement.style.setProperty(
                    `--fds-${kebab(key)}`, hslTriplet(info.theme[key])
                );
    } else { isNativeStore.set(false); initMobileDetect(); }
} catch { isNativeStore.set(false); initMobileDetect(); }

// kebab:      accentLight3   -> "accent-light-3"
// hslTriplet: [191, 98, 80]  -> "191, 98%, 80%"

setInterval(syncThemeStore, 100);   // html.classList -> dahili tema store'u
```

Yani bir native shell şu sözleşmeyi sağlarsa site onu tanır:

```ts
window.NativeApp = {
    fetchInfo: () => ({
        isNative: true,
        os: { platform, release, type, isWindows11 },
        theme: { accentLight3: [h,s,l], accentLight2: [...], accentLight1: [...],
                 accentBase: [...], accentDark1: [...], accentDark2: [...], accentDark3: [...] }
    }),
    setTheme: (mode /* "system" | "light" | "dark" */) => {}
};
```

**Sonuç:** Tauri önizleme webview'imiz istenirse *native app* rolüne girebilir.
Ama `body.native-app` sınıfı layout'u değiştirir (site native modda farklı
görünür). Bu yüzden **varsayılan KAPALI**, editörde "Native app modunu emüle et"
toggle'ı olarak sunulacak. Ana yol 0.3'teki `theme_content`.

Ayrıca `setInterval(syncThemeStore, 100)` sayesinde `<html>` class'ını dışarıdan
değiştirmemiz sitenin kendi tema store'una ≤100ms içinde yansır — light/dark
geçişi için ekstra bir şey yapmamıza gerek yok.

### 0.5 CSP ve iframe durumu

`https://openani.me/` yanıt başlıkları:

```
content-security-policy: object-src 'self';
                         script-src 'strict-dynamic' https: 'unsafe-inline' 'wasm-unsafe-eval' 'nonce-…';
                         base-uri 'none'
x-frame-options: SAMEORIGIN
```

| Gözlem | Sonuç |
|---|---|
| **`style-src` direktifi YOK** | `<style>` enjeksiyonu ve `adoptedStyleSheets` tamamen serbest. Tema enjeksiyonumuz CSP'ye takılmaz. |
| `script-src` `'strict-dynamic'` + nonce | Sayfaya elle `<script>` eklemek **bloklanır** (`'strict-dynamic'`, CSP3 tarayıcılarda `'unsafe-inline'`i geçersiz kılar). |
| Ama `webview.eval()` / `initialization_script` | Bunlar host (WebView2) seviyesinde çalışır, sayfa CSP'sine tabi değildir. **JS enjeksiyonu için tek doğru yol budur.** |
| `x-frame-options: SAMEORIGIN` | **iframe kesinlikle imkânsız.** Gerçek bir webview şart. |

### 0.6 ⚠️ Svelte 5 fluent-svelte-extra'yı çalıştıramaz — kanıtlandı

npm'deki `fluent-svelte-extra@2.2.5` → `internal.js` ilk satırı:

```js
import { bubble, listen } from "svelte/internal";
```

`createEventForwarder` bu import'a dayanıyor ve neredeyse **tüm** bileşenler
`createEventForwarder` kullanıyor.

Svelte 5.56.8'de `./internal` export'u hâlâ var, ama içeriği şu:

```js
throw new Error(
  `Your application, or one of its dependencies, imported from 'svelte/internal',
   which was a private module used by Svelte 4 components that no longer exists in Svelte 5. …`
);
```

Scratchpad'de iki gerçek build ile doğrulandı:

| Kurulum | `vite build` | Runtime import | DOM mount |
|---|---|---|---|
| svelte@5.56.8 + vite-plugin-svelte@5 | ✅ geçti (yanıltıcı!) | ❌ **`ERR: imported from 'svelte/internal'`** | — |
| svelte@4.2.20 + vite-plugin-svelte@3 | ✅ geçti | ✅ `IMPORT OK` | ✅ 16 bileşen mount oldu (10 272 char DOM) |

Svelte 4 testinde mount edilip doğrulanan bileşenler: `TextBox`, `Slider`,
`RangeSlider`, `NumberBox`, `TextArea`, `ToggleSwitch`, `ComboBox`,
`SegmentedControl` + `SegmentedControlButton`, `Expander`, `InfoBar`,
`TextBlock`, `ContentDialog`, `Button`, `MenuFlyout`, `TeachingTip`.

> **Dikkat:** Svelte 5'te build'in *geçmesi* tuzak. Hata ancak uygulama tarayıcıda
> açılınca beyaz ekran olarak ortaya çıkar. Bu yüzden erken karar vermek şart.

**Karar: `svelte@^4.2` + `@sveltejs/vite-plugin-svelte@^3`.** SvelteKit 2 Svelte
4'ü tam destekler. Ayrıca openani.me'nin kendisi de Svelte 4 ile derlenmiş
(bundle'lardaki `$$scope` / `ctx` imzaları) — sürüm paritesi tema tutarlılığı
açısından da doğru olan.

Ek bulgu: `internal.ts`'deki `getCSSDuration()` modül seviyesinde `window`'a
dokunuyor, yani kütüphane **SSR-unsafe**. Mevcut iskelette
`src/routes/+layout.ts` zaten `export const ssr = false` içeriyor — sorun yok,
ama bu satır asla silinmemeli.

### 0.7 Tauri 2 API doğrulaması

| API | Durum |
|---|---|
| `Window::add_child(WebviewBuilder, position, size) -> Result<Webview>` | `unstable` + `desktop` feature gate'i arkasında |
| `Window::webviews() -> Vec<Webview>` | aynı gate |
| `Webview::eval(js: impl Into<String>) -> Result<()>` | ✅ stabil (desktop) |
| `Webview::set_position` / `set_size` / `navigate` / `url` | ✅ stabil (desktop) |
| `WebviewWindowBuilder::initialization_script()` | ✅ stabil — "global object oluşturulduktan sonra, HTML parse edilmeden önce" çalışır |
| `WebviewUrl::External(url)` | ✅ stabil |

`Webview::eval` **fire-and-forget**'tir (`Result<()>` döner, sayfadan değer
getirmez). Sayfa→host yönü ayrı ele alınmalı (bkz. §2.4).

---

## 1. Mimari Plan

### 1.1 Proses / pencere topolojisi

Tek pencere, iki webview (seçilen yaklaşım):

```
┌─ WebviewWindow "main" ────────────────────────────────────────────┐
│                                                                   │
│  ┌─ ana webview (frontendDist) ──┐  ┌─ child webview "preview" ─┐  │
│  │  SvelteKit + Svelte 4         │  │  WebviewUrl::External(    │  │
│  │  fluent-svelte-extra          │  │    "https://openani.me")  │  │
│  │  Editör paneli                │  │                           │  │
│  │  · Tauri IPC ✓                │  │  · Tauri IPC ✗ (bilerek)  │  │
│  │  · capability: "default"      │  │  · sadece eval ile beslenir│  │
│  └───────────────┬───────────────┘  └─────────────▲─────────────┘  │
│                  │                                │                │
└──────────────────┼────────────────────────────────┼────────────────┘
                   │ invoke("apply_theme", {css})   │ webview.eval(...)
                   ▼                                │
         ┌─────────────────────────────────────────┴┐
         │  Rust çekirdeği (src-tauri)               │
         │  · ThemeState (Mutex<ThemeDoc>)           │
         │  · css_emitter  (ThemeDoc -> CSS string)  │
         │  · preview_bridge (eval gönderir)         │
         │  · persist (proje kaydet/yükle, export)   │
         └───────────────────────────────────────────┘
```

`unstable` feature gerektiği için `Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = ["unstable"] }
```

### 1.2 Neden CSS'i Rust tarafında üretiyoruz?

CSS emitter'ı frontend'e koyup string'i IPC'den geçirmek de mümkündü. Rust'ta
tutmanın gerekçesi: **export ve önizleme aynı fonksiyondan çıksın.** Dosyaya
yazılan CSS ile webview'e enjekte edilen CSS byte-byte aynı olur; "önizlemede
farklı görünüyordu" sınıfı bugların tamamı yapısal olarak ortadan kalkar.

### 1.3 Tek yönlü veri akışı

```
kullanıcı bir slider'ı oynatır
   → Svelte store (draft ThemeDoc)  [debounce ~16ms]
   → invoke("set_token", { path, value })
   → Rust: state.apply(patch)
   → Rust: css = emit(state)                    ← TEK KAYNAK
   → Rust: preview.eval("__OA_THEME_APPLY__(" + json(css) + ")")
   → önizleme yeniden boyanır (reflow yok, reload yok)
```

Editör UI'ı kendi optimistic state'ini gösterir; Rust'tan geri okuma yapmaz.
`invoke` sırasında yaşanacak birkaç ms'lik gecikme kullanıcıya yansımaz.

### 1.4 Pencere boyutlandırma

`add_child` ile eklenen webview **otomatik resize olmaz**. Ana pencerenin
`WindowEvent::Resized` olayında child webview'in `set_position`/`set_size`
metodları elle çağrılmalı. Ayrıca editör panelinin genişliği kullanıcı
tarafından sürüklenebilir olacaksa (splitter), aynı hesap frontend'den de
tetiklenmeli — `invoke("set_preview_bounds", { x, y, w, h })`.

DPI notu: `add_child` fiziksel piksel bekler; frontend `window.devicePixelRatio`
ile mantıksal→fiziksel dönüşümü yapmalı ya da Rust tarafında
`LogicalPosition`/`LogicalSize` kullanılmalı (tercih: logical, DPI'ı Tauri
halletsin).

---

## 2. CSS Enjeksiyon Stratejisi

### 2.1 Seçilen yöntem ve reddedilenler

| Yöntem | Karar | Gerekçe |
|---|---|---|
| iframe + `contentDocument` | ❌ | `x-frame-options: SAMEORIGIN` (§0.5) |
| Sayfaya `<script>` enjekte etmek | ❌ | `script-src 'strict-dynamic'` bloklar (§0.5) |
| `documentElement.style.setProperty` (token bazlı) | ⚠️ kısmen | Light/dark çift-mod bloklarını ifade edemez; export formatıyla uyuşmaz |
| **`style[themeStyle]`'ın `textContent`'ini `eval` ile güncellemek** | ✅ **ANA YOL** | Sitenin resmi hook'u (§0.3); `style-src` yok; export ile aynı string |
| `window.NativeApp` emülasyonu | ✅ opsiyonel | Accent'ler için inline style ⇒ en yüksek öncelik; ama `body.native-app` layout'u değiştirir |

### 2.2 `initialization_script` — ilk boyamada flash olmaması için

Child webview'i kurarken, sayfa parse edilmeden önce çalışacak script:

```js
(function () {
  var CSS = "/* Rust tarafından inject edilen ilk taslak */";

  // 1) Sitenin boot kodu localStorage'ı okumadan ÖNCE tohumla.
  //    Böylece <style themeStyle> zaten temalı olarak oluşur — flash yok.
  try { localStorage.setItem("theme_content", CSS); } catch (e) {}

  // 2) Canlı güncelleme kancası.
  window.__OA_THEME_APPLY__ = function (css) {
    var el = document.querySelector("style[themeStyle]");
    if (!el) {                                   // site henüz oluşturmadıysa
      el = document.createElement("style");
      el.setAttribute("themeStyle", "true");
      el.type = "text/css";
      (document.head || document.documentElement).appendChild(el);
    }
    el.textContent = css;                        // <- tek satır, anlık
    try { localStorage.setItem("theme_content", css); } catch (e) {}
  };

  // 3) Light/dark modu. Site 100ms'de bir html.classList'i poll ediyor,
  //    dolayısıyla class'ı değiştirmek yeterli.
  window.__OA_THEME_MODE__ = function (mode) {   // "system" | "light" | "dark"
    document.documentElement.className =
      mode === "light" ? "fds-theme-light" : mode === "dark" ? "fds-theme-dark" : "";
    try { localStorage.setItem("theme", mode === "system" ? "0" : mode === "light" ? "1" : "2"); } catch (e) {}
  };

  // 4) (opsiyonel, toggle ile) native app emülasyonu — §0.4 sözleşmesi
  //    window.NativeApp = { fetchInfo: ..., setTheme: ... };
})();
```

`initialization_script` her navigasyonda yeniden çalışır — kullanıcı site içinde
gezindiğinde (`/anime/...`, `/profile` …) tema kaybolmaz. Bu, SPA olmayan tam
sayfa yüklemeleri için de geçerli.

### 2.3 Runtime güncelleme (Rust)

```rust
#[tauri::command]
fn set_token(app: AppHandle, state: State<ThemeState>, path: String, value: JsonValue)
    -> Result<(), String>
{
    let css = { let mut d = state.0.lock().unwrap(); d.apply(&path, value); emit_css(&d) };

    if let Some(pv) = app.get_webview("preview") {
        // serde_json ile kaçış: CSS içindeki ", \, satır sonu, </script> hepsi güvenli
        let payload = serde_json::to_string(&css).map_err(|e| e.to_string())?;
        pv.eval(format!("window.__OA_THEME_APPLY__({payload})"))
          .map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

> **String kaçışı elle yapılmamalı.** `serde_json::to_string` bir Rust `String`'i
> geçerli bir JS string literal'ine çevirir; `format!("...\"{css}\"...")` gibi bir
> yaklaşım tırnak veya ters bölü içeren CSS'te kod enjeksiyonuna dönüşür.

### 2.4 Sayfa → host yönü (yalnızca Faz 3 için gerekli)

`eval` değer döndürmez. "Element seçici / göz damlalığı" özelliği için sayfadan
host'a veri lazım. Tauri 2'de uzak origin'in `invoke` çağırabilmesi için
capability'de `remote.urls` tanımlanmalı:

```json
{
  "identifier": "preview-picker",
  "windows": ["main"],
  "webviews": ["preview"],
  "remote": { "urls": ["https://openani.me/*"] },
  "permissions": ["core:event:allow-emit"]
}
```

⚠️ Bu, **openani.me'ye Tauri IPC erişimi verir**. Bu yüzden:
- yalnızca tek bir dar izin verilir (`core:event:allow-emit`),
- capability yalnızca `preview` webview'ine bağlanır,
- MVP'de **hiç eklenmez** — element seçici Faz 3 özelliğidir.

### 2.5 ⭐ Üretilen CSS'in şekli — specificity çözümü

Bu en ince nokta. Bizim `<style>` etiketimiz DOM'da en sonda ama
`.fds-theme-dark` (specificity `0,1,0`) düz bir `:root`'u (`0,0,1`) yener —
sıra fark etmez. `!important` kullanmadan kazanmanın yolu specificity'yi
eşitlemek yerine **bir tık aşmak**:

```css
/* ── 1. Mod-bağımsız token'lar (accent paleti, radius, font, süre) ── */
:root, .fds-theme-light, .fds-theme-dark {
    --fds-accent-light-3: 191, 98%, 80%;
    --fds-accent-light-2: 199, 99%, 69%;
    --fds-accent-light-1: 205, 100%, 49%;
    --fds-accent-base:    206, 100%, 42%;
    --fds-accent-dark-1:  209, 100%, 36%;
    --fds-accent-dark-2:  215, 100%, 29%;
    --fds-accent-dark-3:  226, 100%, 20%;
    --fds-control-corner-radius: 4px;
    --fds-overlay-corner-radius: 8px;
}

/* ── 2. Light mod ── (0,1,1) > .fds-theme-light (0,1,0) */
@media (prefers-color-scheme: light) {
    :root:not(.fds-theme-dark) { --fds-solid-background-base: hsl(0,0%,100%); /* … */ }
}
:root.fds-theme-light          { --fds-solid-background-base: hsl(0,0%,100%); /* … */ }

/* ── 3. Dark mod ── */
@media (prefers-color-scheme: dark) {
    :root:not(.fds-theme-light) { --fds-solid-background-base: hsl(0,0%,13%);  /* … */ }
}
:root.fds-theme-dark           { --fds-solid-background-base: hsl(0,0%,13%);  /* … */ }
```

Neden bu şekil:
- `:root.fds-theme-dark` = `(0,1,1)` → sitenin `.fds-theme-dark` `(0,1,0)`'ını
  **sıra bağımsız** yener. `!important` gerekmez.
- `:root:not(.fds-theme-dark)` = `(0,1,1)` → sistem modunda (`className === ""`)
  `theme.css`'in media-query `:root` bloğunu yener, ama kullanıcı elle dark'a
  geçtiğinde devreye girmez.
- Light ve dark blokları hep **birlikte** yazılır. Böylece üretilen tek CSS
  dosyası her iki modda da doğru çalışır — kullanıcı export'u alıp normal
  tarayıcısında kullandığında sistem teması değişse bile tema bozulmaz.

Emitter yalnızca **kullanıcının değiştirdiği** token'ları yazar; dokunulmamış
token'lar sitenin varsayılanında kalır. Bu, hem çıktıyı küçük tutar hem de site
gelecekte varsayılanlarını güncellediğinde temanın onunla birlikte evrilmesini
sağlar.

### 2.6 Accent paleti tek bir renkten türetme

`--fds-accent-*` yedilisi `H, S%, L%` triplet'i (`hsl()` sarmalayıcısı **yok** —
site `hsl(var(--fds-accent-dark-1))` şeklinde kullanıyor). Kullanıcı tek bir
accent rengi seçer, emitter Windows'un accent rampasını taklit ederek 7 basamağı
üretir (base'in L değerine göre ±ofsetler, hafif hue kayması). Kullanıcı
isterse "Gelişmiş" bölümünde 7 basamağı tek tek de düzenleyebilir.

Doğrulama: `--fds-accent-default` light modda `hsl(var(--fds-accent-dark-1))`,
dark modda `hsl(var(--fds-accent-light-2))` olarak tanımlı. Yani 7 triplet'i
değiştirmek **tüm siteyi** yeniden renklendirir — buton, focus halkası, link,
seçim rengi dahil. Tek slider, global etki.

---

## 3. Editör Arayüzü — Bileşen Haritası

Kısıt: kendi CSS'imizi yazmıyoruz. Aşağıdaki her şey `fluent-svelte-extra`'nın
export ettiği bileşenlerden kuruluyor (§0.6'da 16'sı mount testinden geçti).

### 3.1 Kabuk (shell)

| Bölge | Bileşen | Not |
|---|---|---|
| Sol gezinme | `NavigationView` | ⚠️ brief'te "deneysel" deniyor — Faz 0'da test edilecek. Çalışmazsa `ListItem` listesi + `Expander` ile aynı iş yapılır |
| Üst menü | `MenuBar` + `MenuBarItem` + `MenuFlyoutItem` + `MenuFlyoutDivider` | Dosya / Düzen / Tema / Yardım |
| Bölüm başlıkları | `TextBlock` (`variant="subtitle"` / `"bodyStrong"`) | |
| Panel/site ayırıcı | *(HTML `<div>` + inline `style`)* | Splitter için fluent karşılığı yok; sadece inline stil, custom CSS sınıfı değil |
| Bildirim | `InfoBar` | "Tema kaydedildi", "Önizleme yeniden yüklendi" |
| İlk kullanım ipucu | `TeachingTip` | "Accent'i değiştir, sağdaki site anında güncellensin" |

### 3.2 Ekran ekran

**A. Renkler / Accent**

| Kontrol | Bileşen |
|---|---|
| Accent seçimi (H / S / L) | 3 × `Slider` + canlı `TextBlock` değeri |
| Hazır palet kartları | `Button` (`variant="hyperlink"`) + inline `style="background:…"` swatch |
| Hex girişi | `TextBox` (+ `TextBoxButton` = "uygula") |
| 7 basamaklı gelişmiş palet | `Expander` içinde 7 × (`NumberBox` ×3) |
| Light / Dark / Sistem | `SegmentedControl` + `SegmentedControlButton` → `__OA_THEME_MODE__` |

> Not: kütüphanede hazır bir **color picker yok**. 2B gradyan yüzeyi custom CSS
> gerektirirdi; bunun yerine HSL slider üçlüsü + swatch kullanıyoruz. Böylece
> "kendi CSS'imizi yazmayacağız" kısıtı bozulmuyor.

**B. Yüzeyler ve Katmanlar** — light/dark için ayrı sekmeler (`SegmentedControl`)

| Kontrol | Bileşen |
|---|---|
| `--fds-solid-background-*` (4 kademe) | `Expander` içinde renk satırları |
| `--fds-card-background-*`, `--fds-layer-*` | aynı |
| `--fds-*-stroke-*` | aynı |
| Token'ı varsayılana döndür | satır sonunda `IconButton` (↺) |
| Token adı ipucu | `Tooltip` — CSS değişkeninin tam adını gösterir |

**C. Tipografi**

| Kontrol | Bileşen |
|---|---|
| Font ailesi (text / small / display / fallback) | `AutoSuggestBox` (sistem fontları) veya `ComboBox` |
| 7 font boyutu | `NumberBox` ×7, `Expander` içinde |
| Önizleme | `TextBlock` — her `variant` bir satır |

**D. Şekil ve Hareket**

| Kontrol | Bileşen |
|---|---|
| `--fds-control-corner-radius` | `Slider` (0–16px) |
| `--fds-overlay-corner-radius` | `Slider` (0–24px) |
| 4 süre token'ı | `Slider` ×4 (0–600ms) |
| Hareketi tamamen kapat | `ToggleSwitch` → hepsini `0ms` |
| Acrylic bulanıklık | `Slider` (`--fds-acrylic-blur-factor`) |
| Acrylic gürültüsü | `ToggleSwitch` |

**E. Ham CSS (kaçış kapağı)**

| Kontrol | Bileşen |
|---|---|
| Serbest CSS ekleme | `TextArea` — emitter çıktısının sonuna eklenir |
| Uyarı | `InfoBar` (`severity="caution"`) — "site güncellemelerinde bozulabilir" |

**F. Dışa / İçe Aktarma**

| Kontrol | Bileşen |
|---|---|
| Aksiyonlar | `SplitButton` (⚠️ deneysel — Faz 0'da test; yedek: `Button` + `MenuFlyout`) |
| ".css olarak kaydet" / "Panoya kopyala" / "theme_content olarak kopyala" | `MenuFlyoutItem` |
| Üzerine yazma onayı | `ContentDialog` |
| Kaydetme ilerlemesi | `ProgressRing` |

**G. Tema Galerisi (Faz 4)**

| Kontrol | Bileşen |
|---|---|
| Kayıtlı temalar ızgarası | `GridViewItem` |
| Tema kartı | `PersonPicture` (yazar avatarı) + `TextBlock` |
| Sil / yeniden adlandır | `ContextMenu` |

### 3.3 Editörün kendi teması siteyle nasıl hizalanır

Editör kabuğu `fluent-svelte-extra`'nın `theme.css` + `switchable.css`'ini aynen
import eder — yani site ile **birebir aynı** token setini kullanır. Üstüne,
editör kendi `<style>`'ına kullanıcının taslak temasını da uygular. Sonuç:
kullanıcı accent'i değiştirdiğinde hem site hem editörün kendi butonları aynı
anda renk değiştirir. "Editör sitenin bir uzantısı gibi hissettirmeli" kısıtı
bu şekilde, ekstra kod olmadan karşılanır.

---

## 4. Adım Adım Uygulama Planı

### Faz 0 — Temel doğrulama (yarım gün) ⚠️ önce bu

Kod yazmadan önce iki bilinmeyeni kapat.

1. **Svelte 4'e geç.** `package.json`: `svelte@^4.2.19`,
   `@sveltejs/vite-plugin-svelte@^3.1.2`, `vite@^5.4`, `svelte-check@^3`.
   `fluent-svelte-extra@2.2.5` ekle. `src/routes/+layout.ts`'deki
   `ssr = false` satırına dokunma (§0.6).
2. `theme.css` + `switchable.css`'i `+layout.svelte`'te import et, bir
   `Button`/`TextBox` bas, `bun run tauri dev` ile aç. Beyaz ekran yoksa geç.
3. **Şüpheli bileşenleri tek tek dene:** `NavigationView`, `SplitButton`,
   `AcrylicSurface`. Çalışmayan için §3'teki yedeğe geç. Sonucu buraya not düş.
4. **`add_child` PoC:** `unstable` feature'ı aç, openani.me'yi child webview
   olarak yükle, `eval` ile `document.title`'ı değiştirebildiğini doğrula.

**Çıkış kriteri:** tek pencerede solda fluent butonu, sağda canlı openani.me.

### Faz 1 — MVP: tek token, uçtan uca (1–2 gün)

Amaç: en dar dikey dilimi çalıştırmak. Sadece **accent rengi**.

1. `ThemeDoc` veri modeli (Rust): `{ accent: [H,S,L], overrides: HashMap<String, String> }`.
2. `emit_css(&ThemeDoc) -> String` — §2.5'teki şekli üretir; §2.6'daki accent
   rampası. Birim testler: 7 triplet doğru mu, specificity blokları doğru mu.
3. `initialization_script` (§2.2) + child webview kurulumu.
4. `set_token` komutu + `eval` köprüsü (§2.3).
5. Editörde 3 `Slider` (H/S/L) + `SegmentedControl` (light/dark/sistem).
6. Pencere resize → `set_preview_bounds` (§1.4).

**Çıkış kriteri:** slider'ı sürüklerken openani.me'nin butonları, linkleri ve
focus halkaları **reload olmadan** anlık renk değiştiriyor.

### Faz 2 — Token kapsamını genişlet (2–3 gün)

1. `--fds-*` token kataloğunu `theme.css` + `switchable.css`'ten **üret**
   (elle yazma — build-time script token adlarını ve varsayılanlarını çıkarsın).
   Böylece kütüphane güncellendiğinde katalog kendiliğinden güncellenir.
2. Kataloğu §3.2'deki gruplara böl (Yüzeyler, Tipografi, Şekil, Hareket).
3. Her token için: değer editörü + `Tooltip`'te tam CSS adı + ↺ sıfırlama.
4. Light/dark ayrı düzenleme + "dark'ı light'tan türet" yardımcısı.
5. `TextArea` ile ham CSS kaçış kapağı.

### Faz 3 — Kalıcılık, export, konfor (2–3 gün)

1. Proje kaydet/yükle: `ThemeDoc` → JSON, `app_data_dir` altında.
2. Export: `.css` dosyası + "panoya `theme_content` olarak kopyala"
   (kullanıcı DevTools'tan `localStorage.theme_content = "..."` yapıştırabilsin).
3. Geri al / yinele (`ThemeDoc` snapshot yığını).
4. Önizleme adres çubuğu: kullanıcı `/anime/...`, `/profile` gibi farklı
   sayfaları gezip temayı orada da görebilsin.
5. "Native app modunu emüle et" toggle'ı (§0.4).
6. *(opsiyonel)* Element seçici — §2.4'teki dar capability ile.

### Faz 4 — Cila

Tema galerisi (`GridViewItem`), hazır tema paketleri, kontrast/erişilebilirlik
uyarıları (`InfoBar` — WCAG kontrast oranı hesabı), `TeachingTip` ile ilk
kullanım turu.

---

## 5. Riskler ve Karşı Önlemler

| Risk | Etki | Önlem |
|---|---|---|
| **Svelte 5 yanlışlıkla geri gelir** (bir `bun update` yeter) | Beyaz ekran, sebebi belirsiz | `package.json`'da `svelte` sürümünü **tam sabitle** (`"4.2.20"`, caret yok). CI'a §0.6'daki import smoke testini ekle |
| Site `theme_content` hook'unu kaldırır | Ürün tamamen kırılır | Emitter zaten saf CSS üretiyor; yedek yol `eval` ile doğrudan `<style>` eklemek — tek satır değişiklik. Uygulama açılışta hook'un varlığını sınayıp `InfoBar` ile uyarsın |
| Site `--fds-*` token adlarını değiştirir | Bazı kontroller etkisiz kalır | Katalog Faz 2'de `theme.css`'ten **üretiliyor**; kütüphaneyi güncellemek yeterli |
| `unstable` feature Tauri'de kırılır | Build patlar | Tauri sürümünü sabitle. Yedek plan: iki ayrı `WebviewWindow` (stabil API, aynı `eval` köprüsü — mimarinin geri kalanı değişmez) |
| `add_child` webview resize'da kayar | Görsel bozukluk | §1.4; `Resized` + splitter sürüklemesinin ikisinden de tetikle |
| openani.me giriş gerektiren sayfaları önizleyemez | Bazı ekranlar test edilemez | Child webview kendi cookie jar'ını tutar; kullanıcı önizleme içinde normal şekilde giriş yapabilir |
| `eval` ile CSS enjeksiyonunda kaçış hatası | Kod enjeksiyonu / bozuk tema | §2.3 — **daima** `serde_json::to_string`, asla elle `format!` |
| `theme_content`'in `localStorage` kotası (~5MB) | Büyük temalar kaydedilmez | Emitter yalnızca değişen token'ları yazar; pratikte birkaç KB |

---

## 6. Özet: neden bu plan işe yarıyor

Üç doğrulanmış gerçek her şeyi basitleştirdi:

1. Site zaten `--fds-*` token sistemini kullanıyor → "resmi giriş noktası"
   sorusunun cevabı hazır geldi.
2. Site `localStorage.theme_content`'i okuyup `<style themeStyle>` olarak
   enjekte ediyor → hem **export formatı** hem **canlı önizleme kanalı** aynı
   şey oldu. WYSIWYG bedava geldi.
3. CSP'de `style-src` yok → enjeksiyonun önünde hiçbir engel yok.

Geriye kalan tek gerçek sürtünme fluent-svelte-extra'nın Svelte 4'e bağlı
olması ki bu da Faz 0'da tek bir `package.json` düzenlemesiyle kapanıyor.
