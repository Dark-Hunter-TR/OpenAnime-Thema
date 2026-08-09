<script>
	// Sitenin kullandığı tema token setinin BİREBİR aynısı.
	// openani.me de bu iki dosyayı yüklüyor (bkz. PLAN.md §0.1), dolayısıyla
	// editörün kendi görsel dili site ile otomatik olarak hizalı kalıyor.
	import "fluent-svelte-extra/theme.css";
	import "fluent-svelte-extra/switchable.css";
</script>

<slot />

<style>
	/* Kendi CSS'imizi yazmıyoruz (PLAN.md kısıt 1). Buradaki istisnalar:
	   (a) uygulama kabuğunun pencereyi kaplaması için gereken sıfırlama,
	   (b) sitenin `--fds-*` setinde KARŞILIĞI OLMAYAN durum kutusu renkleri,
	   (c) fluent bileşenlerinin sitedeki görünüşe getirilmesi.
	   Hiçbirinde uydurma değer yok; hepsi openani.me'nin canlı CSS'inden.

	   ## `--fds-*` token'ları neden burada YENİDEN tanımlanmıyor

	   Yukarıdaki iki import, sitenin yüklediği token dosyalarının aynısı ve
	   tasarım sisteminin tamamını (metin/zemin/kenarlık/accent/durum/süre/
	   yarıçap/tipografi) koyu ve açık varyantlarıyla birlikte zaten getiriyor.
	   Değerleri site ile karşılaştırdık: birebir aynı.

	   Bunları `:root` altında yeniden tanımlamak İKİ ŞEYİ BOZARDI:

	   1. Tema. theme.css renkleri `.fds-theme-light` / `.fds-theme-dark`
	      sınıflarında tanımlıyor. `:root` (0-1-0) ile aynı özgüllükte ama
	      bu dosya SONRA yüklendiği için `:root` kazanır — tek renk seti her
	      iki temaya da yapışır ve açık tema çalışmaz.
	   2. Accent rampası. theme.css `--fds-accent-light-2`yi ÇIPLAK üçlü
	      (`199, 99%, 69%`) olarak tutuyor, çünkü türevleri
	      `hsla(var(--fds-accent-light-2), 90%)` biçiminde üretiliyor.
	      Değeri `hsl(199, 99%, 69%)` yapmak bunu `hsla(hsl(...), 90%)`e
	      çevirir — geçersiz renk, yani buton hover/active ve rozet gradyanı
	      sessizce ölür.

	   Bu yüzden bu dosya token setini yeniden kurmuyor, yalnızca sette
	   KARŞILIĞI OLMAYANI ekliyor: aşağıdaki durum kutusu renkleri ve kenar
	   çubuğunun acrylic yüzeyi. */

	/* Tema değişimi süresince (bir kare) bütün geçişleri kapatır.
	   Gerekçesi `settings.ts` -> `applyAppTheme` içinde: fluent bileşenleri
	   rengi yumuşatarak değiştirirken satır içi stille boyanan renk alanları
	   anında değişiyor ve arada tutarsız kareler oluşuyordu. */
	:global(html.oa-theme-switching *),
	:global(html.oa-theme-switching *::before),
	:global(html.oa-theme-switching *::after) {
		transition: none !important;
		animation-duration: 0s !important;
		animation-delay: 0s !important;
	}

	:global(html),
	:global(body) {
		margin: 0;
		height: 100%;
		overflow: hidden;
		background-color: var(--fds-solid-background-base);
		color: var(--fds-text-primary);
		font-family: var(--fds-font-family-text);
		font-size: var(--fds-body-font-size);
	}

	/* ==================================================================
	   Durum kutusu renkleri
	   ==================================================================
	   Bunlar `--fds-*` setinde YOK. openani.me durum bildirimlerini Sonner ile
	   çiziyor ve `data-rich-colors` modunda her önem derecesi için ayrı bir
	   zemin/kenarlık/metin üçlüsü tanımlıyor. Aşağıdaki on iki değerin tamamı
	   sitenin canlı `openanime-CtLsnAnr.css` dosyasındaki `--success-*`,
	   `--info-*`, `--warning-*`, `--error-*` değişkenlerinden birebir alındı.

	   Neden fluent'in kendi `--fds-system-background-*` değerlerini
	   kullanmıyoruz: onlar tek bir donuk zemin veriyor (başarı için
	   `hsl(67,39%,17%)` — zeytin yeşili) ve metni `--fds-text-primary` beyaz
	   bırakıyor. Sitede aynı kutu koyu yeşil zemin + parlak yeşil metin;
	   uygulamadaki en görünür uyumsuzluk buydu.

	   Blok sırası theme.css'in kendi kalıbını izliyor: önce
	   `prefers-color-scheme` (sınıf yokken, yani "Sistem" modunda geçerli),
	   sonra `.fds-theme-*` sınıfları (elle seçilen tema kazansın diye).
	   İkisi de aynı özgüllükte olduğu için sıra belirleyici.

	   ## `--oa-acrylic-*` neden ayrı iki değişken

	   theme.css'in `--fds-acrylic-background-base`ı tek bir dizede hem katman
	   görselini hem zemin rengini taşıyor. Koyu temada bu geçerli bir
	   `background` kısayolu, ama AÇIK temadaki değeri `transparent,
	   rgba(243,243,243,90%)` — renk yalnızca son katmanda olabildiği için
	   geçersiz. `var()` bir kısayolda çözülemezse kısayolun tamamı `unset`
	   oluyor (altındaki `background-color` bildirimi de dahil), yani şerit
	   açık temada tümüyle saydam kalıyordu. Ölçtük: arkasındaki renk aynen
	   görünüyordu.

	   Bu yüzden değeri iki longhand'e ayırıyoruz — `background-color` ve
	   `background-image`. Değerler yine token'ın kendi içeriği, uydurma yok. */

	@media (prefers-color-scheme: light) {
		:global(:root) {
			--success-bg: hsl(143, 85%, 96%);
			--success-border: hsl(145, 92%, 91%);
			--success-text: hsl(140, 100%, 27%);

			--info-bg: hsl(208, 100%, 97%);
			--info-border: hsl(221, 91%, 91%);
			--info-text: hsl(210, 92%, 45%);

			--warning-bg: hsl(49, 100%, 97%);
			--warning-border: hsl(49, 91%, 91%);
			--warning-text: hsl(31, 92%, 45%);

			--error-bg: hsl(359, 100%, 97%);
			--error-border: hsl(359, 100%, 94%);
			--error-text: hsl(360, 100%, 45%);

			--oa-acrylic-fill: rgba(243, 243, 243, 90%);
			--oa-acrylic-tint: none;
		}
	}

	@media (prefers-color-scheme: dark) {
		:global(:root) {
			--success-bg: hsl(150, 100%, 6%);
			--success-border: hsl(147, 100%, 12%);
			--success-text: hsl(150, 86%, 65%);

			--info-bg: hsl(215, 100%, 6%);
			--info-border: hsl(223, 100%, 12%);
			--info-text: hsl(216, 87%, 65%);

			--warning-bg: hsl(64, 100%, 6%);
			--warning-border: hsl(60, 100%, 12%);
			--warning-text: hsl(46, 87%, 65%);

			--error-bg: hsl(358, 76%, 10%);
			--error-border: hsl(357, 89%, 16%);
			--error-text: hsl(358, 100%, 81%);

			--oa-acrylic-fill: rgba(32, 32, 32, 96%);
			--oa-acrylic-tint: linear-gradient(0deg, rgba(32, 32, 32, 50%), rgba(32, 32, 32, 50%));
		}
	}

	:global(.fds-theme-light) {
		--success-bg: hsl(143, 85%, 96%);
		--success-border: hsl(145, 92%, 91%);
		--success-text: hsl(140, 100%, 27%);

		--info-bg: hsl(208, 100%, 97%);
		--info-border: hsl(221, 91%, 91%);
		--info-text: hsl(210, 92%, 45%);

		--warning-bg: hsl(49, 100%, 97%);
		--warning-border: hsl(49, 91%, 91%);
		--warning-text: hsl(31, 92%, 45%);

		--error-bg: hsl(359, 100%, 97%);
		--error-border: hsl(359, 100%, 94%);
		--error-text: hsl(360, 100%, 45%);

		--oa-acrylic-fill: rgba(243, 243, 243, 90%);
		--oa-acrylic-tint: none;
	}

	:global(.fds-theme-dark) {
		--success-bg: hsl(150, 100%, 6%);
		--success-border: hsl(147, 100%, 12%);
		--success-text: hsl(150, 86%, 65%);

		--info-bg: hsl(215, 100%, 6%);
		--info-border: hsl(223, 100%, 12%);
		--info-text: hsl(216, 87%, 65%);

		--warning-bg: hsl(64, 100%, 6%);
		--warning-border: hsl(60, 100%, 12%);
		--warning-text: hsl(46, 87%, 65%);

		--error-bg: hsl(358, 76%, 10%);
		--error-border: hsl(357, 89%, 16%);
		--error-text: hsl(358, 100%, 81%);

		--oa-acrylic-fill: rgba(32, 32, 32, 96%);
		--oa-acrylic-tint: linear-gradient(0deg, rgba(32, 32, 32, 50%), rgba(32, 32, 32, 50%));
	}

	/* ==================================================================
	   Durum kutusu yerleşimi ve renklendirmesi (`InfoBar`)
	   ==================================================================
	   Yapı: sol ikon (20px) + orta metin bloğu (başlık + açıklama) + sağ
	   aksiyon. Ölçüler: `gap: .75rem`, `padding: 12px 16px`, `radius: 6px`,
	   `1px` durum kenarlığı.

	   fluent'in kendi kutusu 48px'lik sabit yükseklik, 15px sol dolgu ve
	   ikonu tepeye yaslayan `margin-block-start: 16px` kullanıyor; bunları
	   sıfırlamak zorundayız, yoksa dolgu iki kez uygulanıyor.

	   ## Neden `!important`

	   fluent-svelte-extra bileşenlerini ÇİFT kapsam sınıfıyla derliyor:
	   `.info-bar.svelte-fp4fp6.svelte-fp4fp6` (özgüllük 0-3-0), önem derecesi
	   kuralları ise 0-4-0. Uygulama düzeyindeki global bir yaprak stil bunu
	   özgüllükle geçemez — `.info-bar` yalnızca 0-1-0. Seçiciyi dört-beş kez
	   tekrarlayarak yapay özgüllük üretmek okunmaz bir CSS bırakırdı.

	   openani.me da fluent bileşenlerini tam olarak böyle eziyor (kenar
	   çubuğundaki `color: var(--fds-text-tertiary)!important` ve
	   `.setting-btn { position: absolute!important }` aynı sorunun aynı
	   çözümü). Yani bu, kod tabanına yabancı bir kaçamak değil, sitenin de
	   kullandığı yol. */

	:global(.info-bar) {
		gap: 0.75rem !important;
		padding: 12px 16px !important;
		padding-inline-start: 16px !important;
		min-block-size: 0 !important;
		border-radius: 6px !important;
		align-items: center !important;
	}

	/* İkon artık kutunun ortasında ve metinle aynı renkte. */
	:global(.info-bar-icon) {
		margin-block-start: 0 !important;
		align-self: center !important;
		color: inherit !important;
	}

	/* fluent metin bloğuna dört bir yandan pay veriyor; dolgu artık kutuda. */
	:global(.info-bar-content),
	:global(.info-bar-content.message-wrapped),
	:global(.info-bar-content.action-wrapped) {
		margin: 0 !important;
		gap: 0.75rem !important;
		align-items: center !important;
	}

	:global(.info-bar-content.action-wrapped .info-bar-action) {
		padding-block-start: 0 !important;
	}

	:global(.info-bar-content.message-wrapped .info-bar-action) {
		margin-inline-end: 0 !important;
	}

	/* Başlık `type-body-strong` (14/20/600), açıklama çağrı yerlerinde
	   `type-caption` (12/16/400). İkisi de kutunun durum rengini miras alır. */
	:global(.info-bar h5),
	:global(.info-bar p) {
		color: inherit !important;
		margin-inline-end: 0 !important;
	}

	/* `margin-inline-start: auto` aksiyonu satırın sağ ucuna itiyor. Metin
	   uzun olduğunda `.info-bar-content` sarıyor ve aksiyon tek başına ikinci
	   satıra düşüyor; bu olmadan orada sola yapışık, düzeni bozan bir düğme
	   kalıyordu. Sarmadığı durumda etkisi yok — `p` zaten `flex: 1 1 auto`. */
	:global(.info-bar-action) {
		align-self: center !important;
		margin-inline-start: auto !important;
		margin-inline-end: 0 !important;
	}

	/* `StatusBar` aksiyon slot'unu koşulsuz iletiyor (gerekçesi orada);
	   aksiyon verilmediğinde sarmalayıcı boş kalır ve gizlenmeli. */
	:global(.info-bar-action:empty) {
		display: none !important;
	}

	/* Kapatma düğmesi 38px'lik kutusuyla 12px'lik dolgunun içinde satırı
	   şişiriyordu; standart kontrol ölçüsüne (32px) çekildi. */
	:global(.info-bar-close-button) {
		align-self: center !important;
		margin: 0 !important;
		inline-size: 32px !important;
		block-size: 32px !important;
		color: inherit !important;
	}

	:global(.info-bar.severity-success) {
		background-color: var(--success-bg) !important;
		border-color: var(--success-border) !important;
		color: var(--success-text) !important;
	}

	/* `information` ve `attention` aynı "bilgi" ailesinden; fluent ikisini ayrı
	   önem derecesi sayıyor, sitede ikisi de mavi. */
	:global(.info-bar.severity-information),
	:global(.info-bar.severity-attention) {
		background-color: var(--info-bg) !important;
		border-color: var(--info-border) !important;
		color: var(--info-text) !important;
	}

	:global(.info-bar.severity-caution) {
		background-color: var(--warning-bg) !important;
		border-color: var(--warning-border) !important;
		color: var(--warning-text) !important;
	}

	:global(.info-bar.severity-critical) {
		background-color: var(--error-bg) !important;
		border-color: var(--error-border) !important;
		color: var(--error-text) !important;
	}

	/* ==================================================================
	   Buton basılma geri bildirimi
	   ==================================================================
	   Renk / hover / disabled kuralları fluent'te zaten sitedeki değerlerle
	   birebir aynı (üç varyantın tamamı). Eksik olan tek şey basılma anındaki
	   `scale(.97)` küçülmesiydi. Süre 83ms (`--fds-control-faster-duration`).

	   fluent devre dışı durumu `:disabled` ile değil `.disabled` sınıfıyla
	   işaretliyor — seçici ona göre.

	   `transition` üzerindeki `!important` yukarıdakiyle aynı gerekçeden:
	   fluent onu `.button.svelte-x` (0-2-0) ile yazıyor. `transform` ve
	   `min-block-size` ise fluent'te hiç tanımlı değil, dolayısıyla onlarda
	   ezmeye gerek yok. */
	:global(.button),
	:global(.icon-button) {
		transition: background var(--fds-control-faster-duration) ease,
			border-color var(--fds-control-faster-duration) ease,
			transform var(--fds-control-faster-duration)
				var(--fds-control-fast-out-slow-in-easing) !important;
	}

	:global(.button) {
		min-block-size: 32px;
	}

	:global(.button:not(.disabled):active),
	:global(.icon-button:not(.disabled):not(:disabled):active) {
		transform: scale(0.97);
	}

	/* ==================================================================
	   Metin rengi yardımcıları
	   ==================================================================
	   Sitenin kendi yardımcı sınıflarının aynısı (`.text-primary`,
	   `.text-secondary`, `.text-tertiary`). Açıklama metinleri, tarih/ipucu
	   gibi ikincil bilgiler bunlarla işaretleniyor; renk seçimi böylece
	   çağrı yerinde görünür oluyor, gizli bir CSS kuralında değil.
	   `!important` sitedeki tanımın parçası: `TextBlock` rengi `currentColor`
	   ile miras aldığı için, kapsayıcı bir renk verdiğinde onu ezmeleri gerek. */
	:global(.text-primary) {
		color: var(--fds-text-primary) !important;
	}

	:global(.text-secondary) {
		color: var(--fds-text-secondary) !important;
	}

	:global(.text-tertiary) {
		color: var(--fds-text-tertiary) !important;
	}
</style>
