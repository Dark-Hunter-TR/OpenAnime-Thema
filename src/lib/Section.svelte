<script lang="ts">
	import { Expander, IconButton, TextBlock } from "fluent-svelte-extra";
	import Tooltip from "$lib/Tooltip.svelte";
	import Icon from "$lib/Icon.svelte";
	import type { IconName } from "$lib/icons";

	export let icon: IconName;
	export let title: string;
	export let expanded = false;
	/**
	 * Verilirse başlıkta o bölüme özel bir "Sıfırla" düğmesi çıkar.
	 * Verilmezse düğme hiç render edilmez — her bölüm sıfırlanabilir olmak
	 * zorunda değil (ör. yalnız gezinme amaçlı olanlar).
	 */
	export let onReset: (() => void) | null = null;

	/**
	 * Sıfırlama düğmesi Expander'ın başlık düğmesinin İÇİNDE duruyor; tıklama
	 * yukarı kabarırsa bölüm bir de açılıp kapanır. Olayı burada durduruyoruz.
	 * `keydown` de gerekli: Expander başlıkta Enter/Space'i yakalayıp
	 * genişletmeyi çeviriyor.
	 */
	function handleReset(event: Event) {
		event.stopPropagation();
		onReset?.();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === "Enter" || event.key === " ") event.stopPropagation();
	}
</script>

<!--
	Panelin her bölümü aynı kalıptan geçer: Expander + ikon + başlık + (varsa)
	sıfırlama. Tek yerde tanımlı olduğu için boşluklandırma ve hizalama
	bölümler arasında tutarlı kalıyor; her bölümde ayrı ayrı elle ayarlanmıyor.
-->
<Expander bind:expanded>
	<Icon slot="icon" name={icon} />

	<div class="title-row">
		<TextBlock variant="bodyStrong">{title}</TextBlock>
		{#if onReset}
			<!-- svelte-ignore a11y-no-static-element-interactions -->
			<span class="reset" on:click={handleReset} on:keydown={handleKeydown}>
				<Tooltip text="{title} bölümünü varsayılana döndür">
					<IconButton aria-label="{title} bölümünü sıfırla">
						<Icon name="reset" size={14} />
					</IconButton>
				</Tooltip>
			</span>
		{/if}
	</div>

	<div slot="content" class="content">
		<slot />
	</div>
</Expander>

<style>
	.title-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		/* Başlık metni uzarsa sıfırlama düğmesi ezilmesin. */
		min-width: 0;
	}

	.reset {
		flex: 0 0 auto;
		display: inline-flex;
	}

	.content {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	/* `overflow: hidden` taşıyan çocuklar (fluent'in .segmented-control'ü gibi)
	   dikey flex içinde sıfır yüksekliğe ezilebiliyor — min-height'ları `auto`
	   yerine 0 hesaplanıyor. Hiçbiri küçülmesin. */
	.content > :global(*) {
		flex: 0 0 auto;
	}
</style>
