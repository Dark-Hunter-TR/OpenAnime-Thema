<script lang="ts">
	/**
	 * Launcher'ın sol kenar çubuğu.
	 *
	 * İskelet openani.me'nin `sidebar` DOM'uyla aynı prensipler üzerine kurulu
	 * (kopya değil, aynı yapı):
	 *
	 *   sidebar
	 *   ├── #top      birincil gezinme öğeleri
	 *   └── #bottom   ikincil öğeler
	 *
	 * Her öğe bir `list-item`; içinde üstte ikon, altında 10px'lik `.label`.
	 * Aktif öğe `selected` sınıfını alır ve İKON RENGİ vurgu rengine döner;
	 * pasif öğeler `--fds-text-tertiary` gri tonunda kalır. Etiket, sitede
	 * olduğu gibi seçili olsun olmasın hep üçüncül renkte.
	 *
	 * Sitenin canlı CSS'inden alınan ölçüler:
	 *   .sidebar          { min/max-width: 4.5rem; justify-content: space-between }
	 *   .sidebar > div    { flex-direction: column; align-items: center; gap: .25rem }
	 *   .sidebar #bottom  { margin-bottom: .75rem }
	 *   .sidebar a        { width: 4rem; height: 3.7rem; padding-inline: 0;
	 *                       margin: 0; overflow: hidden }
	 *   .sidebar a::before{ inline-size: .25rem; block-size: 1.5rem }
	 *   .sidebar a.selected { background: var(--fds-control-solid-fill-default) }
	 *   .sidebar a > span { flex-direction: column; color: var(--fds-text-tertiary);
	 *                       transition: color var(--fds-control-faster-duration) ease }
	 *   .sidebar a #label { position: absolute; bottom: .25rem }
	 *   .sidebar a .iconify { margin-bottom: .75rem }
	 *
	 * İki bilinçli uyarlama:
	 *
	 * 1. Sitede Anasayfa/Kütüphane/Ayarlar ikonları Lottie animasyonu; bizde
	 *    statik Fluent System Icons. Boyut (24px), kalınlık (`regular`/`filled`)
	 *    ve renk davranışı aynı bırakıldı.
	 * 2. Etiket sarmalayıcısı sitede `id="label"`; aynı öğeden birden fazla
	 *    olduğu için burada `class="label"`. Aynı yapı, geçerli HTML.
	 *
	 * `::before` göstergesini elle çizmiyoruz — `ListItem` fluent-svelte-extra'nın
	 * bileşeni ve o çubuk onun içinde zaten var. Sitenin kuralları da aynı
	 * bileşenin `.list-item` sınıfını hedefliyor.
	 */
	import { ListItem, TextBlock, Tooltip } from "fluent-svelte-extra";

	import Icon from "$lib/Icon.svelte";
	import type { IconName } from "$lib/icons";
	import type { NavId } from "$lib/nav";

	interface NavEntry {
		id: NavId;
		label: string;
		icon: IconName;
		/** Seçiliyken kullanılan `filled` varyant. */
		iconOn: IconName;
		hint: string;
	}

	export let current: NavId = "home";
	/** Editör yalnızca bir proje açıkken anlamlı — kapalıyken sönük görünür. */
	export let editorEnabled = false;
	export let onNavigate: (id: NavId) => void;

	/** `#top` — birincil gezinme. */
	const TOP: NavEntry[] = [
		{
			id: "home",
			label: "Ana ekran",
			icon: "navHome",
			iconOn: "navHomeOn",
			hint: "Kayıtlı temalar ve yeni tema oluşturma"
		},
		{
			id: "editor",
			label: "Editör",
			icon: "navEditor",
			iconOn: "navEditorOn",
			hint: "Açık temayı düzenle"
		}
	];

	/** `#bottom` — ikincil öğeler. Sitede de Ayarlar bu grupta. */
	const BOTTOM: NavEntry[] = [
		{
			id: "settings",
			label: "Ayarlar",
			icon: "navSettings",
			iconOn: "navSettingsOn",
			hint: "Uygulama ayarları ve hakkında"
		}
	];

	const isDisabled = (entry: NavEntry) => entry.id === "editor" && !editorEnabled;

	function go(entry: NavEntry) {
		if (isDisabled(entry)) return;
		onNavigate(entry.id);
	}
</script>

<nav class="sidebar">
	<!--
		Gruplar `<ul>` DEĞİL: `Tooltip` slot'unu bir `<div>` içine sarıyor,
		dolayısıyla `<ul>` kullansaydık araya geçersiz bir katman girerdi
		(`ul > div > li`). Öğeler `role="button"` taşıdığı için liste
		semantiğine ihtiyaç yok; gruplama `role="group"` ile veriliyor.
	-->
	{#each [{ id: "top", items: TOP }, { id: "bottom", items: BOTTOM }] as group (group.id)}
		<div id={group.id} class="group" role="group">
			{#each group.items as entry (entry.id)}
				{@const disabled = isDisabled(entry)}
				{@const selected = current === entry.id}
				<Tooltip
					text={disabled ? "Önce bir tema açın ya da oluşturun" : entry.hint}
					placement="right"
				>
					<ListItem
						{selected}
						{disabled}
						role="button"
						aria-current={selected ? "page" : undefined}
						aria-label={entry.label}
						on:click={() => go(entry)}
					>
						<!--
							Seçili / sönük durumu CSS'te `.list-item.selected .entry`
							diye inemiyoruz: Svelte `:global(...)`'a bir seçici
							dizisinin ORTASINDA izin vermiyor. Durumu doğrudan
							elemana yazmak hem bu kısıtı çözüyor hem de stilin
							nereden geldiğini okunur bırakıyor.
						-->
						<span class="entry" class:selected class:disabled>
							<Icon name={selected ? entry.iconOn : entry.icon} size={24} />
							<span class="label">
								<TextBlock variant="caption">{entry.label}</TextBlock>
							</span>
						</span>
					</ListItem>
				</Tooltip>
			{/each}
		</div>
	{/each}
</nav>

<style>
	/* Görsel karar yok: renk, gösterge çubuğu, hover ve odak halkası
	   `ListItem`'dan geliyor. Buradaki ölçülerin tamamı openani.me'nin
	   `.sidebar` kurallarından birebir alındı (bkz. yukarıdaki not). */
	.sidebar {
		box-sizing: border-box;
		position: relative;
		min-width: 4.5rem;
		max-width: 4.5rem;
		flex: 0 0 auto;
		display: flex;
		flex-direction: column;
		align-items: center;
		/* İki blok: #top yukarı, #bottom aşağı yaslanır. */
		justify-content: space-between;
		transition: all var(--fds-control-fast-duration);
		background-color: var(--fds-solid-background-base);
		border-right: 1px solid var(--fds-divider-stroke-default);
		/* Sitede şeridin ve her öğenin taşıdığı `no-select` sınıfının karşılığı. */
		user-select: none;
		-webkit-user-select: none;
	}

	.group {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		margin: 0;
		padding: 0.5rem 0 0;
	}

	/* Sitenin `#bottom` kuralı. */
	#bottom {
		padding-top: 0;
		margin-bottom: 0.75rem;
	}

	/* --- ListItem'ı şerit ölçüsüne getir ---------------------------------
	   `:global` şart: sınıf `ListItem`'ın kök elemanına gidiyor. Değiştirilen
	   şeylerin hepsi YERLEŞİM; renk ve durum stilleri bileşenden geliyor. */
	.sidebar :global(.list-item) {
		position: relative;
		inline-size: 4rem;
		block-size: 3.7rem;
		padding-inline: 0;
		margin: 0;
		display: flex;
		justify-content: center;
		align-items: center;
		/* Etiket ve ikon kutunun dışına taşmasın. */
		overflow: hidden;
	}

	/* Aktif göstergenin ölçüsü: sitede 3px/16px değil, .25rem/1.5rem. */
	.sidebar :global(.list-item::before) {
		inline-size: 0.25rem;
		block-size: 1.5rem;
	}

	/* Seçili öğenin zemini — sitenin kendi değeri. */
	.sidebar :global(.list-item.selected) {
		background-color: var(--fds-control-solid-fill-default);
	}

	/* `ListItem` varsayılan slot'u bir `TextBlock` içine sarıyor; payı
	   sıfırlanmazsa içerik şeritte ortalanmıyor. */
	.sidebar :global(.list-item > .text-block) {
		margin: 0;
		display: flex;
		justify-content: center;
		align-items: center;
	}

	/* --- Öğe içi: ikon üstte, etiket altta ------------------------------- */
	.entry {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		min-width: 24px;
		min-height: 24px;
		color: var(--fds-text-tertiary);
		/* Aktif durum geçişi yumuşak olsun — sitenin kendi kuralı. */
		transition: color var(--fds-control-faster-duration) ease;
	}

	/* Etiket her öğede duruyor, dolayısıyla ikon sabit bir alt pay bırakıp
	   ona yer açıyor (sitenin `.iconify { margin-bottom: .75rem }` kuralı). */
	.entry :global(.icon) {
		margin-bottom: 0.75rem;
	}

	/* Aktif öğe: yalnızca İKON rengi vurgu rengine döner. */
	.entry.selected {
		color: var(--fds-accent-default);
	}

	.entry.disabled {
		color: var(--fds-text-disabled);
	}

	/* Etiket: sitedeki `#label` sarmalayıcısının karşılığı — akıştan çıkarılmış,
	   dibe .25rem uzaklıkta. Akıştan çıkması, ikonun dikey hizasını etiketin
	   uzunluğundan bağımsız tutuyor. */
	.label {
		position: absolute;
		bottom: 0.25rem;
		pointer-events: none;
		max-width: 100%;
		overflow: hidden;
	}

	/* Sitedeki etiket `type-caption` + `text-tertiary` + satır içi 10px.
	   Rengi ikondan BAĞIMSIZ: öğe seçiliyken ikon accent'e döner, etiket
	   üçüncül metin renginde kalır — sitede de böyle. */
	.label :global(.text-block) {
		display: block;
		font-size: 10px;
		line-height: 1;
		color: var(--fds-text-tertiary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
