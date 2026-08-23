<script lang="ts">
	/**
	 * İpucu balonu.
	 *
	 * fluent-svelte-extra'nın `Tooltip`'i yerine neden kendi bileşenimiz var:
	 * o bileşen balonu tetikleyicinin YANINDA, `position: absolute` bir kardeş
	 * olarak açıyor. İki sonucu var ve ikisi de uygulamada görünür hâle geldi.
	 *
	 *   1. Kırpılma. Balon, `overflow` taşıyan herhangi bir üst kutunun içinde
	 *      kalıyor. Editör paneli (`.panel`) dikey kaydırma için
	 *      `overflow-y: auto` taşıyor; CSS'te bir eksen `visible` dışındaysa
	 *      diğeri de `auto`ya düştüğü için panel YATAYDA da kırpıyor. 320px'e
	 *      kadar genişleyebilen balon, dar bir panelde iki yanından birden
	 *      kesiliyordu — metnin başı ve sonu görünmüyordu.
	 *
	 *   2. Ekran dışına taşma. Kütüphanede çarpışma denetimi yok: balon
	 *      tetikleyiciye göre ortalanıyor ve pencere kenarına yakın bir
	 *      tetikleyicide pencerenin dışına çıkıyor.
	 *
	 * Buradaki bileşen balonu `document.body`ye taşıyıp `position: fixed` ile
	 * konumlandırıyor. `fixed` kutu görünüm alanına göre konumlandığı için
	 * hiçbir üst kutunun `overflow`u onu kırpamıyor; konum da her açılışta
	 * ölçülüp görünür alana sıkıştırılıyor.
	 */
	import { onDestroy } from "svelte";

	/** Balonun metni. Boşsa balon hiç açılmaz. */
	export let text = "";
	/** Tercih edilen yön. Yer yoksa karşı tarafa çevriliyor. */
	export let placement: "top" | "bottom" | "left" | "right" = "bottom";
	/** Tetikleyici ile balon arasındaki boşluk (px). */
	export let offset = 8;
	/** Açılma gecikmesi (ms). */
	export let delay = 400;

	/** Görünür alanın kenarına bırakılan asgari pay. */
	const MARGIN = 8;

	let trigger: HTMLElement;
	let visible = false;
	let timer: ReturnType<typeof setTimeout> | undefined;

	function show() {
		if (!text) return;
		clearTimeout(timer);
		timer = setTimeout(() => (visible = true), delay);
	}

	function hide() {
		clearTimeout(timer);
		visible = false;
	}

	onDestroy(() => clearTimeout(timer));

	/**
	 * Balonu gövdeye taşır, konumlar ve konumu güncel tutar.
	 *
	 * Ölçüm DOM'a girdikten sonra yapılmak zorunda: balonun genişliği
	 * içeriğine bağlı (`width: max-content`) ve hangi yöne çevrileceği o
	 * genişliğe bakılarak belirleniyor.
	 */
	function floating(node: HTMLElement) {
		document.body.appendChild(node);
		const update = () => position(node);
		update();

		// `capture: true` şart: balonu kırpan kutu pencerenin kendisi değil,
		// içerideki kaydırılabilir panel. Kabarcık aşamasında dinlenseydi o
		// panelin kaydırması hiç duyulmaz ve balon havada asılı kalırdı.
		window.addEventListener("scroll", update, true);
		window.addEventListener("resize", update);

		return {
			destroy() {
				window.removeEventListener("scroll", update, true);
				window.removeEventListener("resize", update);
				// Düğüm artık bileşenin kendi ağacında değil; Svelte'in blok
				// temizliği ona ulaşamayabileceği için elle kaldırılıyor.
				node.remove();
			}
		};
	}

	function position(node: HTMLElement) {
		if (!trigger) return;

		const a = trigger.getBoundingClientRect();
		const t = node.getBoundingClientRect();
		const vw = document.documentElement.clientWidth;
		const vh = document.documentElement.clientHeight;

		// Yer yoksa karşı tarafa çevir. Yalnızca bir kez çevriliyor: iki taraf
		// da darsa aşağıdaki sıkıştırma zaten balonu görünür alanda tutuyor.
		let side = placement;
		if (side === "bottom" && a.bottom + offset + t.height > vh - MARGIN) side = "top";
		else if (side === "top" && a.top - offset - t.height < MARGIN) side = "bottom";
		else if (side === "right" && a.right + offset + t.width > vw - MARGIN) side = "left";
		else if (side === "left" && a.left - offset - t.width < MARGIN) side = "right";

		let top: number;
		let left: number;
		if (side === "top") {
			top = a.top - offset - t.height;
			left = a.left + a.width / 2 - t.width / 2;
		} else if (side === "bottom") {
			top = a.bottom + offset;
			left = a.left + a.width / 2 - t.width / 2;
		} else if (side === "left") {
			left = a.left - offset - t.width;
			top = a.top + a.height / 2 - t.height / 2;
		} else {
			left = a.right + offset;
			top = a.top + a.height / 2 - t.height / 2;
		}

		// Görünür alana sıkıştır. `Math.max(MARGIN, …)` dıştaki sınır için:
		// balon pencereden genişse sol kenara yaslanıp sağdan taşması,
		// ortalanıp iki yandan birden kesilmesine yeğdir.
		left = Math.min(Math.max(MARGIN, left), Math.max(MARGIN, vw - MARGIN - t.width));
		top = Math.min(Math.max(MARGIN, top), Math.max(MARGIN, vh - MARGIN - t.height));

		node.style.left = `${Math.round(left)}px`;
		node.style.top = `${Math.round(top)}px`;
	}
</script>

<svelte:window on:keydown={(e) => e.key === "Escape" && hide()} />

<!--
	`on:click` ile de kapanıyor: bir düğmeye basıldığında balonun basılan şeyin
	üstünde asılı kalması, tıklamanın işe yarayıp yaramadığını gizliyor.
-->
<div
	class="tooltip-trigger"
	bind:this={trigger}
	on:mouseenter={show}
	on:mouseleave={hide}
	on:focusin={show}
	on:focusout={hide}
	on:click={hide}
	role="presentation"
>
	<slot />
</div>

{#if visible && text}
	<div class="oa-tooltip" role="tooltip" use:floating>{text}</div>
{/if}

<style>
	/*
	   Kutu modeli bilerek kütüphanenin `.tooltip-wrapper`'ı ile aynı
	   (`display: block`): bileşen yerine geçtiği için yerleşimi değiştirmemeli.
	   `position: relative` gerekmiyor — balon artık gövdede.
	*/
	.tooltip-trigger {
		display: block;
	}

	/*
	   `:global` şart: düğüm `document.body`ye taşındığı için Svelte'in
	   kapsamlı sınıfı ona uygulanmıyor.

	   Görünüm, kütüphanenin `TooltipSurface`'ı ile aynı token'lardan kuruluyor;
	   böylece bileşen değişse de balon uygulamanın geri kalanıyla aynı görünür.
	*/
	:global(.oa-tooltip) {
		position: fixed;
		z-index: 10001;
		pointer-events: none;
		box-sizing: border-box;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: max-content;
		/* Dar pencerede 320px bile taşabilir; görünür alan üst sınır. */
		max-width: min(320px, calc(100vw - 2 * 8px));
		padding-inline: 8px;
		padding-block: 5px 7px;
		font-family: var(--fds-font-family-text);
		font-size: var(--fds-body-font-size);
		font-weight: 400;
		line-height: 20px;
		color: var(--fds-text-primary);
		background-color: var(--fds-solid-background-quarternary);
		background-clip: padding-box;
		border: 1px solid var(--fds-surface-stroke-flyout);
		border-radius: var(--fds-control-corner-radius);
		box-shadow: var(--fds-tooltip-shadow);
		user-select: none;
		/* Uzun metin sığmıyorsa kesilmek yerine sarsın. */
		overflow-wrap: anywhere;
	}
</style>
