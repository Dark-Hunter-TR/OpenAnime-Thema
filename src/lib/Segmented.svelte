<script lang="ts">
	import { SegmentedControl } from "fluent-svelte-extra";

	export let value: any = undefined;

	// fluent-svelte-extra bu ikisini varsayılansız export ettiği için
	// (`export let containerElement: HTMLDivElement;`) svelte2tsx onları zorunlu
	// prop sanıyor — oysa yalnızca dışa bind edilen DOM referanslarıdır. Burada
	// içeride ZORUNLU (undefined olmayan) bir kopyayla bağlanıp dışarıya
	// OPSİYONEL bir prop olarak senkronluyoruz; böylece bu sarmalayıcıyı
	// containerElement'i hiç kullanmayan mevcut çağrı yerleri de kırılmadan
	// çalışmaya devam ediyor, isteyen (ör. bir sekme şeridini elle kaydırmak
	// için) de `bind:containerElement` yapabiliyor.
	let innerContainer: HTMLDivElement;
	let innerHighlight: HTMLDivElement;

	export let containerElement: HTMLDivElement | undefined = undefined;
	export let highlightElement: HTMLDivElement | undefined = undefined;

	$: containerElement = innerContainer;
	$: highlightElement = innerHighlight;
</script>

<SegmentedControl bind:value bind:containerElement={innerContainer} bind:highlightElement={innerHighlight}>
	<slot />
</SegmentedControl>
