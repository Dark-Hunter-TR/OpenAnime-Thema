<script lang="ts">
	import { Flyout, IconButton, Slider, TextBlock, TextBox } from "fluent-svelte-extra";
	import Tooltip from "$lib/Tooltip.svelte";
	import ColorPicker from "$lib/ColorPicker.svelte";
	import type { ColorTokenSpec } from "$lib/customization";
	import { toCssColor } from "$lib/customization";

	export let spec: ColorTokenSpec;
	export let hex = "#ffffff";
	export let alpha: number = spec.defaultAlpha;
	export let disabled = false;

	// Değeri üst bileşen `toCssColor` ile kendisi hesaplıyor; buradaki hesap
	// yalnızca önizleme çipi için. Böylece tek yönlü veri akışı korunuyor.
	$: css = toCssColor(hex, alpha);
	$: valid = css !== null;

	// Palet ve hex kutusu AYNI `hex` değişkenine bağlı — ikisi tek state.
	let pickerOpen = false;
</script>

<div class="field">
	<div class="head">
		<Tooltip text={spec.token}>
			<TextBlock variant="caption">{spec.label}</TextBlock>
		</Tooltip>

		<!-- Renk çipi aynı zamanda paleti açan düğme. -->
		<Flyout bind:open={pickerOpen} placement="bottom" alignment="end">
			<Tooltip text="Renk paletini aç">
				<IconButton {disabled} aria-label="{spec.label} için renk paleti">
					<span class="chip" style={valid ? `background: ${css}` : ""}></span>
				</IconButton>
			</Tooltip>
			<svelte:fragment slot="flyout">
				<ColorPicker bind:hex {disabled} />
			</svelte:fragment>
		</Flyout>
	</div>

	<TextBox bind:value={hex} {disabled} placeholder="#ffffff" clearButton={false} />

	{#if spec.alpha}
		<div class="alpha">
			<TextBlock variant="caption">Saydamlık — %{alpha}</TextBlock>
			<Slider bind:value={alpha} min={0} max={100} step={1} {disabled} suffix="%" />
		</div>
	{/if}

	<TextBlock variant="caption">{spec.hint}</TextBlock>
</div>

<style>
	/* Yalnızca yerleşim; renkler --fds-* token'larından. */
	.field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}

	.chip {
		width: 20px;
		height: 20px;
		display: block;
		border-radius: var(--fds-control-corner-radius);
		border: 1px solid var(--fds-control-stroke-default);
		background-color: var(--fds-control-alt-fill-tertiary);
	}

	.alpha {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
</style>
