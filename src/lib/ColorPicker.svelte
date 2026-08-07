<script lang="ts">
	/**
	 * Tam renk paleti: doygunluk/parlaklık alanı + ton kaydırıcısı +
	 * hex ve RGB girişleri.
	 *
	 * fluent-svelte-extra'da hazır bir ColorPicker yok, bu yüzden bileşen
	 * kütüphanenin kendi parçalarından kuruldu: kabuk `Flyout`, sayısal
	 * girişler `NumberBox`, hex girişi `TextBox`, ton kaydırıcısı `Slider`.
	 * Yalnızca SV alanı ham bir div — Fluent'te karşılığı olmayan tek parça;
	 * o da rengini yine `--fds-*` token'larından alıyor.
	 *
	 * Tek state kuralı: dışarıya SADECE `hex` bağlanır. HSV yalnızca bu
	 * bileşenin içinde, kullanıcı sürüklerken tutulur — böylece HSL
	 * kaydırıcıları, hex kutusu ve palet aynı değeri gösterir, biri
	 * diğerini ezmez.
	 */
	import { createEventDispatcher } from "svelte";
	import { NumberBox, Slider, TextBlock, TextBox } from "fluent-svelte-extra";
	import { hexToRgb } from "$lib/customization";

	export let hex = "#ffffff";
	export let disabled = false;

	/**
	 * `change` olayı, `bind:hex` KULLANAMAYAN çağıranlar için.
	 *
	 * Vurgu rengi bölümünde asıl kaynak `doc.accent` (HSL). Orada iki yönlü
	 * bağlama, Svelte'in "cyclical dependency" hatasına yol açıyor: bir reaktif
	 * ifade `doc`'tan `hex`e, bir başkası `hex`ten `doc`'a yazardı. Olayla
	 * yazınca yön bir olay işleyicisine taşınıyor ve döngü kalmıyor.
	 * (hex -> HSL -> hex turu 8-bit renklerin TAMAMI için birebir aynı sonucu
	 * veriyor, dolayısıyla değer zıplaması olmuyor.)
	 */
	const dispatch = createEventDispatcher<{ change: string }>();
	let lastSent = hex;
	$: if (hex !== lastSent) {
		lastSent = hex;
		dispatch("change", hex);
	}

	// --- Dönüşümler ---------------------------------------------------------

	function rgbToHsv(r: number, g: number, b: number): [number, number, number] {
		const rn = r / 255;
		const gn = g / 255;
		const bn = b / 255;
		const max = Math.max(rn, gn, bn);
		const min = Math.min(rn, gn, bn);
		const d = max - min;
		let h = 0;
		if (d !== 0) {
			if (max === rn) h = ((gn - bn) / d) % 6;
			else if (max === gn) h = (bn - rn) / d + 2;
			else h = (rn - gn) / d + 4;
			h *= 60;
			if (h < 0) h += 360;
		}
		return [h, max === 0 ? 0 : d / max, max];
	}

	function hsvToRgb(h: number, s: number, v: number): [number, number, number] {
		const c = v * s;
		const hp = (((h % 360) + 360) % 360) / 60;
		const x = c * (1 - Math.abs((hp % 2) - 1));
		const [r1, g1, b1] =
			hp < 1
				? [c, x, 0]
				: hp < 2
					? [x, c, 0]
					: hp < 3
						? [0, c, x]
						: hp < 4
							? [0, x, c]
							: hp < 5
								? [x, 0, c]
								: [c, 0, x];
		const m = v - c;
		return [
			Math.round((r1 + m) * 255),
			Math.round((g1 + m) * 255),
			Math.round((b1 + m) * 255)
		];
	}

	const toHex = (rgb: [number, number, number]) =>
		"#" + rgb.map((n) => Math.max(0, Math.min(255, Math.round(n))).toString(16).padStart(2, "0")).join("");

	// --- İç durum -----------------------------------------------------------
	// Ton, doygunluk 0 ya da parlaklık 0 olduğunda hex'ten geri okunamıyor
	// (siyah ve beyazın tonu yoktur). Kullanıcı kaydırıcıyı oraya götürüp geri
	// getirdiğinde ton sıfırlanmasın diye tonu ayrıca saklıyoruz.
	let hue = 0;
	let sat = 0;
	let val = 1;
	/** Değişikliğin kaynağı biz miyiz? Öyleyse hex'ten geri okuma yapma. */
	let selfEdit = false;

	/**
	 * Ton kaydırıcısı `bind:value` ile doğrudan `hue`'yu yazıyor. Slider'ın
	 * kendi `change` olayı her değer değişiminde (bizim programatik
	 * güncellememizde de) tetiklendiği için ona güvenemiyoruz; bunun yerine
	 * "uygulanmış ton"u ayrı tutup yalnızca gerçek fark olduğunda emit
	 * ediyoruz. Böylece hex -> ton -> hex döngüsü kapanmıyor.
	 */
	let appliedHue = 0;

	function syncFromHex(value: string) {
		if (selfEdit) {
			selfEdit = false;
			return;
		}
		const rgb = hexToRgb(value);
		if (!rgb) return;
		const [h, s, v] = rgbToHsv(...rgb);
		// s veya v sıfırsa ton tanımsızdır (siyah/beyazın tonu yoktur);
		// kullanıcı kaydırıcıyı oraya götürüp geri getirince ton sıfırlanmasın.
		if (s > 0 && v > 0) hue = h;
		sat = s;
		val = v;
		appliedHue = hue;
	}

	$: syncFromHex(hex);

	$: if (hue !== appliedHue) {
		appliedHue = hue;
		emit();
	}

	function emit() {
		selfEdit = true;
		hex = toHex(hsvToRgb(hue, sat, val));
	}

	// --- SV alanı -----------------------------------------------------------

	let area: HTMLDivElement;

	function pick(event: PointerEvent) {
		if (disabled || !area) return;
		const rect = area.getBoundingClientRect();
		sat = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width));
		val = 1 - Math.min(1, Math.max(0, (event.clientY - rect.top) / rect.height));
		emit();
	}

	function onPointerDown(event: PointerEvent) {
		if (disabled) return;
		// Pointer capture olmadan imleç alandan çıkınca sürükleme kopuyor.
		area.setPointerCapture(event.pointerId);
		pick(event);
	}

	function onPointerMove(event: PointerEvent) {
		if (disabled || !area?.hasPointerCapture(event.pointerId)) return;
		pick(event);
	}

	function onPointerUp(event: PointerEvent) {
		if (area?.hasPointerCapture(event.pointerId)) area.releasePointerCapture(event.pointerId);
	}

	/** Klavye erişilebilirliği — SV alanı ok tuşlarıyla da gezilebilmeli. */
	function onKeydown(event: KeyboardEvent) {
		if (disabled) return;
		const step = event.shiftKey ? 0.1 : 0.02;
		if (event.key === "ArrowLeft") sat = Math.max(0, sat - step);
		else if (event.key === "ArrowRight") sat = Math.min(1, sat + step);
		else if (event.key === "ArrowUp") val = Math.min(1, val + step);
		else if (event.key === "ArrowDown") val = Math.max(0, val - step);
		else return;
		event.preventDefault();
		emit();
	}

	// --- RGB girişleri ------------------------------------------------------
	// NumberBox string döndürüyor; sayıya çevirip hex'e yazıyoruz.
	$: rgb = hexToRgb(hex) ?? [255, 255, 255];

	/**
	 * NumberBox `change`/`input` olaylarını kendi `<input>`'undan iletiyor,
	 * yani elimizdeki gerçek bir DOM olayı. `currentTarget` iletim sırasında
	 * güvenilir değil; `target` her zaman o input.
	 */
	function onChannel(index: number, event: Event) {
		const input = event.target as HTMLInputElement | null;
		if (!input) return;
		const n = Number(input.value);
		if (!Number.isFinite(n)) return;
		const next: [number, number, number] = [rgb[0], rgb[1], rgb[2]];
		next[index] = Math.max(0, Math.min(255, Math.round(n)));
		selfEdit = false; // hex'ten HSV'yi yeniden okusun
		hex = toHex(next);
	}

	$: hueColor = toHex(hsvToRgb(hue, 1, 1));
</script>

<div class="picker" class:disabled>
	<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
	<div
		class="area"
		bind:this={area}
		role="slider"
		tabindex={disabled ? -1 : 0}
		aria-label="Doygunluk ve parlaklık"
		aria-valuetext="Doygunluk %{Math.round(sat * 100)}, parlaklık %{Math.round(val * 100)}"
		aria-valuenow={Math.round(sat * 100)}
		aria-valuemin={0}
		aria-valuemax={100}
		style="background-color: {hueColor}"
		on:pointerdown={onPointerDown}
		on:pointermove={onPointerMove}
		on:pointerup={onPointerUp}
		on:pointercancel={onPointerUp}
		on:keydown={onKeydown}
	>
		<div class="area-white"></div>
		<div class="area-black"></div>
		<div
			class="thumb"
			style="left: {sat * 100}%; top: {(1 - val) * 100}%; background: {hex}"
		></div>
	</div>

	<div class="hue">
		<Slider bind:value={hue} min={0} max={360} step={1} {disabled} />
	</div>

	<div class="inputs">
		<!-- svelte-ignore a11y-label-has-associated-control -->
		<!-- Etiket, fluent bileşeninin ürettiği input'u sarıyor; `for` verecek
		     kararlı bir id dışarıdan erişilebilir değil. -->
		<label class="hex-field">
			<TextBlock variant="caption">Hex</TextBlock>
			<TextBox bind:value={hex} {disabled} clearButton={false} placeholder="#ffffff" />
		</label>
	</div>

	<div class="rgb">
		{#each ["R", "G", "B"] as channel, i}
			<!-- svelte-ignore a11y-label-has-associated-control -->
			<label>
				<TextBlock variant="caption">{channel}</TextBlock>
				<NumberBox
					value={String(rgb[i])}
					min={0}
					max={255}
					step={1}
					{disabled}
					on:change={(e) => onChannel(i, e)}
					on:input={(e) => onChannel(i, e)}
				/>
			</label>
		{/each}
	</div>
</div>

<style>
	/* Yalnızca yerleşim ve renk uzayı gradyanları; kenarlık/yarıçap
	   değerleri --fds-* token'larından geliyor. */
	.picker {
		display: flex;
		flex-direction: column;
		gap: 8px;
		width: 232px;
	}

	.picker.disabled {
		opacity: 0.5;
		pointer-events: none;
	}

	.area {
		position: relative;
		height: 130px;
		border-radius: var(--fds-control-corner-radius);
		border: 1px solid var(--fds-control-stroke-default);
		overflow: hidden;
		cursor: crosshair;
		touch-action: none;
		outline: none;
	}

	.area:focus-visible {
		box-shadow: var(--fds-focus-stroke);
	}

	/* Soldan sağa doygunluk, yukarıdan aşağı parlaklık — HSV alanının
	   standart iki katmanlı kurulumu. */
	.area-white,
	.area-black {
		position: absolute;
		inset: 0;
	}

	.area-white {
		background: linear-gradient(to right, #fff, rgba(255, 255, 255, 0));
	}

	.area-black {
		background: linear-gradient(to top, #000, rgba(0, 0, 0, 0));
	}

	.thumb {
		position: absolute;
		width: 12px;
		height: 12px;
		border-radius: 50%;
		border: 2px solid #fff;
		box-shadow: var(--fds-flyout-shadow);
		transform: translate(-50%, -50%);
		pointer-events: none;
	}

	/* Ton kaydırıcısının rayını gökkuşağı yapıyoruz — Slider'ın kendi
	   yapısını değiştirmeden, yalnızca zeminini boyayarak. */
	.hue :global(.slider-rail) {
		background: linear-gradient(
			to right,
			#f00 0%,
			#ff0 17%,
			#0f0 33%,
			#0ff 50%,
			#00f 67%,
			#f0f 83%,
			#f00 100%
		) !important;
	}

	/* Ton seçilirken dolgu şeridi anlamsız; rayın rengi zaten bilgiyi taşıyor. */
	.hue :global(.slider-track) {
		background: transparent !important;
	}

	.inputs,
	.rgb {
		display: flex;
		gap: 6px;
	}

	.rgb label {
		flex: 1 1 0;
		min-width: 0;
	}

	.hex-field {
		flex: 1 1 auto;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
</style>
