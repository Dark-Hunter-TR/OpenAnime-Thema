<script lang="ts">
	/**
	 * Açılır liste — kütüphanenin `ComboBox`'ı yerine.
	 *
	 * ## Neden kendi bileşenimiz
	 *
	 * `ComboBox`'ın listesi bu panelde tetikleyicisinden kopup pencerenin sol
	 * üst köşesine, tam genişlikte açılıyordu. Kütüphanenin kaynağı doğru
	 * (`<ul>`, `.combo-box` div'inin içinde; div `position: relative`, liste
	 * `position: absolute`) ve uygulamada `.combo-box`a dokunan bir kural yok;
	 * sebep kaynaktan bulunamadı.
	 *
	 * Yerel `<select>` konumlandırmayı çözüyordu ama görünümü çözmüyor:
	 * Windows'ta açılır listeyi işletim sistemi çiziyor ve `option` renkleri
	 * yok sayılıyor — koyu arayüzün ortasında beyaz bir liste kalıyordu.
	 *
	 * Buradaki çözüm uygulamanın KENDİ kanıtlanmış kalıbı (`Tooltip.svelte`):
	 * liste `document.body`ye taşınıp `position: fixed` ile tetikleyicinin
	 * dikdörtgenine göre konumlanıyor. Böylece hem konum doğru (hiçbir
	 * ata kutunun kırpması ya da konumlandırma bağlamı işe karışmıyor) hem de
	 * görünüm tamamen bizim elimizde.
	 */
	import { createEventDispatcher, onDestroy } from "svelte";

	export let items: { name: string; value: number }[] = [];
	export let value: number = 0;
	export let disabled = false;
	/** Ekran okuyucu etiketi. */
	export let label = "";

	const dispatch = createEventDispatcher<{ change: number }>();

	/** Görünür alanın kenarına bırakılan asgari pay. */
	const MARGIN = 8;
	/** Tetikleyici ile liste arasındaki boşluk. */
	const GAP = 4;

	let trigger: HTMLButtonElement;
	let open = false;
	/** Klavyeyle gezinirken vurgulanan satır. */
	let active = 0;

	$: selected = items.find((item) => item.value === value);

	function toggle() {
		if (disabled) return;
		open = !open;
		if (open) active = Math.max(0, items.findIndex((item) => item.value === value));
	}

	function choose(next: number) {
		value = next;
		open = false;
		dispatch("change", next);
		trigger?.focus();
	}

	function onKeydown(event: KeyboardEvent) {
		if (disabled) return;

		if (event.key === "Escape") {
			if (!open) return;
			open = false;
			event.preventDefault();
			return;
		}
		if (!open) {
			if (event.key === "Enter" || event.key === " " || event.key === "ArrowDown") {
				toggle();
				event.preventDefault();
			}
			return;
		}
		if (event.key === "ArrowDown") {
			active = Math.min(items.length - 1, active + 1);
			event.preventDefault();
		} else if (event.key === "ArrowUp") {
			active = Math.max(0, active - 1);
			event.preventDefault();
		} else if (event.key === "Enter" || event.key === " ") {
			if (items[active]) choose(items[active].value);
			event.preventDefault();
		}
	}

	/** Dışarı tıklamada kapat. */
	function onPointerDown(event: PointerEvent) {
		if (!open) return;
		const target = event.target as Node | null;
		if (target && (trigger?.contains(target) || listNode?.contains(target))) return;
		open = false;
	}

	let listNode: HTMLElement | null = null;

	onDestroy(() => {
		listNode = null;
	});

	/**
	 * Listeyi gövdeye taşır, konumlar ve konumu güncel tutar.
	 *
	 * `capture: true` şart: listeyi kırpan ve kaydıran kutu pencerenin kendisi
	 * değil, içerideki editör paneli. Kabarcık aşamasında dinlenseydi o panelin
	 * kaydırması duyulmaz ve liste havada asılı kalırdı — `Tooltip.svelte`
	 * aynı sebeple aynı şeyi yapıyor.
	 */
	function floating(node: HTMLElement) {
		listNode = node;
		document.body.appendChild(node);

		const update = () => position(node);
		update();

		window.addEventListener("scroll", update, true);
		window.addEventListener("resize", update);

		return {
			destroy() {
				window.removeEventListener("scroll", update, true);
				window.removeEventListener("resize", update);
				// Düğüm artık bileşenin kendi ağacında değil; Svelte'in blok
				// temizliği ona ulaşamayabileceği için elle kaldırılıyor.
				node.remove();
				listNode = null;
			}
		};
	}

	function position(node: HTMLElement) {
		if (!trigger) return;

		const a = trigger.getBoundingClientRect();
		const vh = document.documentElement.clientHeight;

		// Genişlik her zaman tetikleyiciyle aynı: açılır liste onun devamı.
		node.style.width = `${a.width}px`;
		node.style.left = `${a.left}px`;

		// Aşağıda yer yoksa yukarı çevir. Ölçüm DOM'a girdikten sonra yapılıyor
		// çünkü yükseklik öğe sayısına bağlı.
		const height = node.getBoundingClientRect().height;
		const below = a.bottom + GAP;
		node.style.top =
			below + height > vh - MARGIN && a.top - GAP - height > MARGIN
				? `${a.top - GAP - height}px`
				: `${below}px`;
	}
</script>

<svelte:window on:pointerdown={onPointerDown} />

<button
	type="button"
	class="select-trigger"
	class:open
	bind:this={trigger}
	{disabled}
	aria-label={label || undefined}
	aria-haspopup="listbox"
	aria-expanded={open}
	on:click={toggle}
	on:keydown={onKeydown}
>
	<span class="select-label">{selected?.name ?? ""}</span>
	<svg class="select-chevron" viewBox="0 0 48 48" aria-hidden="true">
		<path
			fill="currentColor"
			d="M8.4 16.1a1.5 1.5 0 0 1 2.1 0L24 29.6l13.5-13.5a1.5 1.5 0 1 1 2.1 2.1L25.1 32.7a1.5 1.5 0 0 1-2.2 0L8.4 18.2a1.5 1.5 0 0 1 0-2.1Z"
		/>
	</svg>
</button>

{#if open}
	<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
	<ul class="select-list" role="listbox" use:floating>
		{#each items as item, i}
			<!--
				Klavye satırlarda DEĞİL tetikleyicide yönetiliyor: ARIA listbox
				deseninde odak düğmede kalır, yön tuşları `aria-activedescendant`
				ile vurguyu gezdirir. Satırlara ayrı bir tuş dinleyicisi koymak
				onları odak sırasına sokar ve deseni bozar.
			-->
			<!-- svelte-ignore a11y-click-events-have-key-events -->
			<li
				role="option"
				aria-selected={item.value === value}
				class:active={i === active}
				class:selected={item.value === value}
				on:click={() => choose(item.value)}
				on:pointerenter={() => (active = i)}
			>
				{item.name}
			</li>
		{/each}
	</ul>
{/if}

<style>
	/* Kapalı hâl kütüphanenin kutularıyla aynı token'lardan kuruluyor. */
	.select-trigger {
		display: flex;
		align-items: center;
		gap: 8px;
		inline-size: 100%;
		box-sizing: border-box;
		padding: 5px 11px 7px;
		font-family: var(--fds-font-family-text);
		font-size: var(--fds-body-font-size);
		line-height: 20px;
		text-align: start;
		color: var(--fds-text-primary);
		background-color: var(--fds-control-fill-default);
		border: 1px solid var(--fds-control-stroke-default);
		border-bottom-color: var(--fds-control-strong-stroke-default);
		border-radius: var(--fds-control-corner-radius);
		cursor: pointer;
		transition: background-color var(--fds-control-faster-duration) ease;
	}

	.select-trigger:hover:not(:disabled) {
		background-color: var(--fds-control-fill-secondary);
	}

	.select-trigger:active:not(:disabled),
	.select-trigger.open {
		background-color: var(--fds-control-fill-tertiary);
	}

	.select-trigger:focus-visible {
		outline: 2px solid var(--fds-accent-default);
		outline-offset: -2px;
	}

	.select-trigger:disabled {
		color: var(--fds-text-disabled);
		background-color: var(--fds-control-fill-disabled);
		cursor: default;
	}

	.select-label {
		flex: 1 1 auto;
		min-inline-size: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.select-chevron {
		flex: 0 0 auto;
		inline-size: 12px;
		block-size: 12px;
		color: var(--fds-text-secondary);
	}

	/*
	   `:global` şart: düğüm `document.body`ye taşındığı için Svelte'in
	   kapsamlı sınıfı ona uygulanmıyor (bkz. `Tooltip.svelte`'deki aynı not).
	*/
	:global(.select-list) {
		position: fixed;
		z-index: 10000;
		box-sizing: border-box;
		margin: 0;
		padding: 2px;
		list-style: none;
		max-block-size: 40vh;
		overflow-y: auto;
		background-color: var(--fds-solid-background-quarternary);
		background-clip: padding-box;
		border: 1px solid var(--fds-surface-stroke-flyout);
		border-radius: var(--fds-overlay-corner-radius);
		box-shadow: var(--fds-flyout-shadow);
	}

	:global(.select-list li) {
		position: relative;
		padding: 7px 11px 8px;
		font-family: var(--fds-font-family-text);
		font-size: var(--fds-body-font-size);
		line-height: 20px;
		color: var(--fds-text-primary);
		border-radius: var(--fds-control-corner-radius);
		cursor: pointer;
	}

	:global(.select-list li.active) {
		background-color: var(--fds-subtle-fill-secondary);
	}

	/* Seçili satırın soluna vurgu şeridi — kütüphanenin liste kalıbı. */
	:global(.select-list li.selected::before) {
		content: "";
		position: absolute;
		inset-block: 25%;
		inset-inline-start: 0;
		inline-size: 3px;
		border-radius: 3px;
		background-color: var(--fds-accent-default);
	}
</style>
