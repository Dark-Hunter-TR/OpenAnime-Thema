<script lang="ts">
	/**
	 * OpenAnime Theme Editor Sol Kenar Çubuğu (Sidebar / NavRail).
	 *
	 * openani.me sitesinin canlı sidebar yapısı (`.sidebar.svelte-xoay80`)
	 * ile %100 piksel düzeyinde görsel ve animasyon uyumu:
	 *
	 *   - Genişlik: Sabit 72px (4.5rem).
	 *   - Yerleşim: Flexbox space-between, #top en üstte, #bottom en altta (margin-bottom: 0.75rem / 12px).
	 *   - Buton Kutusu: 64px (4rem) x 59.2px (3.7rem), border-radius: 4px.
	 *   - Aktif Gösterge (Indicator): Sol kenarda 4px x 24px kapsül çubuk (border-radius: 999px, #61caff).
	 *   - İkon Boyutu: 26px optik genişlik (openani.me Lottie ikon alanı dolgunluğunda).
	 *   - İkon & Etiket Geçişi: Pasifken ikon yukarı kayar (margin-bottom: 12px) ve 10px etiket altta görünür.
	 *     Seçildiğinde ikon butonun tam ortasına inmektedir (margin-bottom: 0px) ve altındaki etiket gizlenir.
	 */
	import Icon from "$lib/Icon.svelte";
	import type { IconName } from "$lib/icons";
	import type { NavId } from "$lib/nav";

	interface NavEntry {
		id: NavId;
		label: string;
		icon: IconName;
		iconOn: IconName;
		hint: string;
	}

	export let current: NavId = "home";
	export let aboutOpen = false;
	export let onNavigate: (id: NavId) => void;
	export let onOpenAbout: () => void;

	const TOP: NavEntry[] = [
		{
			id: "home",
			label: "Ana Sayfa",
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

	const BOTTOM: NavEntry[] = [
		{
			id: "about",
			label: "Hakkında",
			icon: "navAbout",
			iconOn: "navAboutOn",
			hint: "OpenAnime hakkında"
		},
		{
			id: "settings",
			label: "Ayarlar",
			icon: "navSettings",
			iconOn: "navSettingsOn",
			hint: "Uygulama ayarları"
		}
	];

	/**
	 * Hiçbir giriş devre dışı değil — Editör dâhil. Açık tema yoksa
	 * `+page.svelte` "ne yapmak istersiniz?" seçicisini açıyor (yeni tema /
	 * GitHub'dan içe aktar / CSS dosyası aç); düğmeyi kapalı göstermek
	 * kullanıcıya nasıl açılacağını göstermezdi.
	 */
	function go(entry: NavEntry) {
		if (entry.id === "about") {
			onOpenAbout();
			return;
		}
		onNavigate(entry.id);
	}
</script>

<nav class="sidebar no-select">
	<div id="top" class="group" role="group">
		{#each TOP as entry (entry.id)}
			{@const selected = aboutOpen ? false : current === entry.id}
			<button
				type="button"
				class="list-item no-select"
				class:selected
				aria-label={entry.label}
				on:click={() => go(entry)}
			>
				<span class="text-block type-body">
					<Icon name={selected ? entry.iconOn : entry.icon} size={26} />
					<div id="label">
						<span class="text-block type-caption text-tertiary" style="font-size: 10px;">{entry.label}</span>
					</div>
				</span>
			</button>
		{/each}
	</div>

	<div id="bottom" class="group" role="group">
		{#each BOTTOM as entry (entry.id)}
			{@const selected = aboutOpen ? entry.id === "about" : current === entry.id}
			<button
				type="button"
				class="list-item no-select setting-btn"
				class:selected
				aria-label={entry.label}
				on:click={() => go(entry)}
			>
				<span class="text-block type-body">
					<Icon name={selected ? entry.iconOn : entry.icon} size={26} />
					<div id="label">
						<span class="text-block type-caption text-tertiary" style="font-size: 10px;">{entry.label}</span>
					</div>
				</span>
			</button>
		{/each}
	</div>
</nav>

<style>
	.sidebar {
		box-sizing: border-box;
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: space-between;
		min-width: 4.5rem;
		max-width: 4.5rem;
		height: 100%;
		flex: 0 0 auto;
		z-index: 10;
		transition: all var(--fds-control-fast-duration);
		background-color: var(--oa-acrylic-fill);
		background-image: var(--oa-acrylic-tint);
		backdrop-filter: blur(20px);
		-webkit-backdrop-filter: blur(20px);
		border-right: 1px solid var(--fds-card-stroke-default);
		user-select: none;
		-webkit-user-select: none;
	}

	.group {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		margin: 0;
		padding: 0;
		width: 100%;
	}

	/* openani.me canlı CSS kuralı: #bottom { margin-bottom: .75rem } */
	#bottom {
		margin-bottom: 0.75rem;
	}

	.sidebar button.list-item {
		position: relative;
		width: 4rem;
		height: 3.7rem;
		padding-inline: 0;
		display: flex;
		justify-content: center;
		align-items: center;
		margin: 0;
		overflow: hidden;
		border-radius: var(--fds-control-corner-radius);
		border: none;
		background: transparent;
		outline: none;
		cursor: pointer;
		transition: background-color var(--fds-control-fast-duration) ease,
			transform var(--fds-control-faster-duration) var(--fds-control-fast-out-slow-in-easing);
	}

	.sidebar button.list-item:hover {
		background-color: var(--fds-subtle-fill-secondary);
	}

	.sidebar button.list-item:active {
		background-color: var(--fds-subtle-fill-tertiary);
		transform: scale(0.97);
	}

	/* Sol kenar aktif çubuğu (24px x 4px kapsül) */
	.sidebar button.list-item::before {
		content: "";
		position: absolute;
		inset-inline-start: 0;
		inline-size: 0.25rem;
		block-size: 1.5rem;
		border-radius: 999px;
		background-color: var(--fds-accent-default);
		opacity: 0;
		transform: scaleY(0);
		transition: transform var(--fds-control-fast-duration) var(--fds-control-fast-out-slow-in-easing),
			opacity var(--fds-control-fast-duration) ease;
	}

	.sidebar button.list-item.selected::before {
		opacity: 1;
		transform: scaleY(1);
	}

	/* Selected zemin rengi (#454545 / var(--fds-control-solid-fill-default)) */
	.sidebar button.list-item.selected {
		background-color: var(--fds-control-solid-fill-default);
		animation: selected-in var(--fds-control-fast-duration);
	}

	@keyframes selected-in {
		0% {
			background-color: var(--fds-control-strong-fill-disabled);
		}
		to {
			background-color: var(--fds-control-solid-fill-default);
		}
	}

	.sidebar button.list-item > span {
		display: flex !important;
		justify-content: center;
		align-items: center;
		flex-direction: column;
		min-width: 24px;
		min-height: 24px;
		width: 100%;
		height: 100%;
		color: var(--fds-text-tertiary) !important;
		transition: color var(--fds-control-faster-duration) ease;
	}

	/* Seçili olduğunda ikon ve span rengi accent rengine döner */
	.sidebar button.list-item.selected > span,
	.sidebar button.list-item.selected :global(svg) {
		color: var(--fds-accent-text-primary, var(--fds-accent-default)) !important;
		fill: currentColor !important;
	}

	.sidebar button.list-item :global(.icon) {
		margin-bottom: 0.75rem;
		transition: margin-bottom var(--fds-control-fast-duration) var(--fds-control-fast-out-slow-in-easing),
			transform var(--fds-control-fast-duration) var(--fds-control-fast-out-slow-in-easing);
	}

	/* Seçildiğinde ikon merkeze oturur (margin-bottom: 0) */
	.sidebar button.list-item.selected :global(.icon) {
		margin-bottom: 0;
	}

	.sidebar button.list-item:hover :global(.icon) {
		transform: scale(1.08);
	}

	.sidebar button #label {
		position: absolute;
		bottom: 0.25rem;
		pointer-events: none;
		text-align: center;
		white-space: nowrap;
		opacity: 1;
		visibility: visible;
		transition: opacity var(--fds-control-fast-duration) ease,
			visibility var(--fds-control-fast-duration) ease;
	}

	/* Seçili durumda alt metin etiketi gizlenir */
	.sidebar button.list-item.selected #label {
		opacity: 0;
		visibility: hidden;
	}

	.sidebar button #label span {
		font-size: 10px !important;
		color: var(--fds-text-tertiary) !important;
		line-height: 12px;
		letter-spacing: 0.1px;
	}
</style>
