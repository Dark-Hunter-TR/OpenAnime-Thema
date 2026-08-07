# Gelişmiş Stil ve Özelleştirme Bulguları

> Sitenin canlı CSS bundle'ları analiz edilmiş ve her selector openani.me üzerinde doğrulanmıştır. Belge, uygulamada sunulan gelişmiş özelleştirme seçeneklerinin teknik gerekçelerini açıklar.

## 0. Doğrulanan ve Düzeltilen CSS Kuralları

| Konu | Analiz Sonucu | Uygulama Kararı |
|---|---|---|
| `.calendar-card` | Sitenin canlı CSS'inde bulunmadı (0 eşleşme). | `CARD_SELECTOR` grubundan çıkarıldı, yerine sitede aktif olan `.grid-view-item` eklendi. |
| Maskot boyutları | `--s-width` ve `--s-height` değişkenleri sitenin CSS'inde yer almaz. | Gerçek boyutlar doğrudan hedef seçicilere uygulandı: `#notification-setsuki`/`#download-setsuki` 170px, `#mobile-notification-setsuki` 150px. |
| Kaydırma çubuğu | Site OverlayScrollbars kullanır ve `::-webkit-scrollbar` stil tanımlarını kapatır (`display:none !important`). | Kaydırma çubuğu kontrolü sitenin aktif `--os-*` API'sine yönlendirildi (`.os-scrollbar { --os-size / --os-handle-bg / --os-track-bg-hover }`). |

### Kart Varsayılan Ölçüleri

`.anime-card` bileşeninin sitedeki temel CSS yapısı:

```css
.anime-card {
  border-radius: var(--fds-overlay-corner-radius); /* = 8px */
  box-shadow: var(--fds-card-shadow);
}
.anime-card.hoverable:hover {
  box-shadow: var(--fds-flyout-shadow);
  transform: translateY(-2px);
}
```

---

## 1. Etkilenen Resmi `--fds-*` Token'ları

Sitenin CSS yapısında yaygın kullanılan resmi Fluent UI token'ları:

| Token | Açıklama | Panel Durumu |
|---|---|---|
| `--fds-accent-default` | Vurgu rengi | Vurgu bölümünde mevcut |
| `--fds-card-background-default` | Kart arkaplanı | Kartlar bölümüne eklendi |
| `--fds-control-corner-radius` | Kontrol yuvarlama yarıçapı | Yuvarlama bölümünde mevcut |
| `--fds-overlay-corner-radius` | Katman yuvarlama yarıçapı | Yuvarlama bölümünde mevcut |
| `--fds-text-primary` | Ana metin rengi | Metin bölümüne eklendi |
| `--fds-text-secondary` | İkincil metin rengi | Metin bölümüne eklendi |
| `--fds-focus-stroke-inner` / `-outer` | Odak halkası renkleri | Odak bölümüne eklendi |
| `--fds-card-stroke-default` | Kart kenarlık rengi | Kartlar bölümüne eklendi |
| `--fds-solid-background-base` / `-secondary` | Sayfa ve zemin renkleri | Yüzeyler bölümüne eklendi |
| `--fds-layer-background-default` | Katman arkaplanı | Yüzeyler bölümüne eklendi |
| `--fds-subtle-fill-secondary` / `-tertiary` | Hover/active dolgu renkleri | Buton bölümünde mevcut |
| `--fds-person-picture-size` | Üst çubuk avatar boyutu | Avatar bölümüne eklendi |
| `--fds-system-attention` | Uyarı ve bildirim rengi | Durum renkleri bölümüne eklendi |

---

## 2. Doğrulanan Seçiciler (Selector Analizi)

### Sitede Aktif Olanlar (Uygulamaya dahil edildi)

| Seçici | Fonksiyonu |
|---|---|
| `.bottom-controls` | Oynatıcı alt kontrol çubuğu |
| `.slider.orientation-horizontal` | Oynatıcı ilerleme ve ses çubuğu |
| `.slider-rail` / `.slider-track` / `.slider-thumb` | İlerleme çubuğu bileşenleri |
| `.openanime-scene-controller` | Sahne içi kontrol butonları |
| `.player-episode-list-item` | Bölüm listesi elemanları |
| `.currentEpisode` | Aktif izlenen bölüm |
| `.setsuki` + `#image` | Maskot görseli |
| `#lottie-player` | Animasyonlu maskot |
| `.anime-card`, `.slider-card`, `.grid-view-item` | İçerik kartları |
| `.topbar`, `.logo`, `.logo-button` | Üst gezinti çubuğu ve logo |
| `video::cue` | Altyazı metinleri (native HTML5 video track) |
| `.scene-inner-content`, `.gradient-scene` | Sayfa katmanları |

### Sitede Karşılığı Olmayanlar (Dışarıda bırakıldı)

`.player-sidebar`, `.video-sidebar`, `.player-comments`, `.comments-panel`, `.explore-card`, `.trending-card` gibi seçiciler sitenin mevcut CSS yapısında yer almadığı için kontrol paneline eklenmemiştir.

---

## 3. Özelleştirme Yöntemleri ve Teknik Kısıtlar

### Logo ve Site Adı Değişimi

Orijinal logo ve metin BAĞIMSIZ olarak gizlenip `::before` ve `::after` sözde elemanları üzerinden özelleştirilir (bkz. `src/lib/advanced.ts`, `src/lib/advancedBuild.ts`):

```css
.topbar a.logo img, .topbar a.logo svg,
.topbar a.logo-button img, .topbar a.logo-button svg { display: none !important; } /* yalnızca görsel değişince */
.topbar a.logo > .text-block, .topbar a.logo-button > .text-block { display: none !important; } /* yalnızca ad değişince */

.topbar a.logo::before, .topbar a.logo-button::before {
  content: ''; order: 0; width: var(--logo-size); height: var(--logo-size);
  background: var(--url-logo) no-repeat center / contain;
}
.topbar a.logo::after, .topbar a.logo-button::after {
  content: 'Özel Başlık'; order: 1; font-family: var(--font-logo);
}
.topbar a.logo #badge, .topbar a.logo-button #badge { order: 2; flex: 0 0 auto !important; }
```

İki ince nokta:

- `::after` CSS gereği HER ZAMAN bir elemanın son kutusudur — gerçek çocuklardan (`#badge` rozeti dâhil) sonra basılır. `order` verilmezse özel ad, DOM'da nerede olursa olsun rozetin ARKASINA düşer. Sıra `order: 0/1/2` ile açıkça ikon → ad → rozet olarak sabitlenir.
- Yalnızca ad değişip görsel değişmediğinde orijinal ikon gizlenmez, ama satır flex olduğu ve rozet sabit genişlik aldığı için ikonun `flex-shrink`'i de kapatılmalıdır (`flex: 0 0 auto`) — aksi hâlde satırdaki tek esnek öğe odur ve yer daralınca 0 genişliğe küçülüp kaybolur.

### Maskot Değişimi

```css
.setsuki .image-wrapper img { opacity: 0 !important; }
.setsuki .image-wrapper::after {
  content: ''; position: absolute; inset: 0;
  background-image: var(--url-maskot);
  background-size: contain; background-repeat: no-repeat; background-position: center;
}
```

### Altyazı Özelleştirmesi Sınırlaması

Sitede altyazılar iki ayrı mekanizmayla çizilir:
1. Native HTML5 `<track>` kullanıldığında `video::cue` ile CSS kuralları uygulanabilir.
2. ASS/SSA formatındaki gelişmiş altyazılar `<canvas>` üzerine çizildiği için CSS ile doğrudan özelleştirilemez.

---

## 4. Eklenen Gelişmiş Özelleştirme Bölümleri

| Bölüm | Amaç |
|---|---|
| **Kenar çubuğu ve menü** | Sol navigasyon çubuğu genişliği, seçili öğe göstergesi ve etiket boyutları. |
| **Yüzeyler ve katmanlar** | Opak zemin, ikincil zemin ve katman yüzeylerinin renk tonları. |
| **Bağlantılar** | Hiperlink metinlerinin vurgu renginden bağımsız boyanması. |
| **Kaydırma çubuğu** | OverlayScrollbars `--os-*` değişkenleri üzerinden boyut ve tutamak renkleri. |
| **Rozetler** | Çeşitli rozet ve vurgu şeritlerinin gradyan renkleri. |
| **Profil fotoğrafı** | Üst çubuktaki avatar görselinin piksel boyutu (kartlardaki avatarları etkilemez). |
| **Banner / kayan kartlar** | Kayan banner kartlarının çerçevesi ve ilerleme çubuğu stilleri. |
| **Durum renkleri** | Sistem bilgi, başarı, uyarı ve hata bildirim renkleri. |
