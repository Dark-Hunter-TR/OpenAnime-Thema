<script>
	// Sitenin kullandığı tema token setinin BİREBİR aynısı.
	// openani.me de bu iki dosyayı yüklüyor (bkz. PLAN.md §0.1), dolayısıyla
	// editörün kendi görsel dili site ile otomatik olarak hizalı kalıyor.
	import "fluent-svelte-extra/theme.css";
	import "fluent-svelte-extra/switchable.css";
</script>

<slot />

<style>
	/* Kendi CSS'imizi yazmıyoruz (PLAN.md kısıt 1). Buradaki tek istisna,
	   uygulama kabuğunun pencereyi tam kaplaması için gereken sıfırlama —
	   hiçbir görsel karar içermez, tüm renk/şekil değerleri --fds-* token'larından gelir. */
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
</style>
