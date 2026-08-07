<script lang="ts">
	import { SegmentedControl } from "fluent-svelte-extra";

	export let value: any = undefined;

	// fluent-svelte-extra bu ikisini varsayılansız export ettiği için
	// (`export let containerElement: HTMLDivElement;`) svelte2tsx onları zorunlu
	// prop sanıyor — oysa yalnızca dışa bind edilen DOM referansları.
	// Burada bir kez bağlayıp her kullanım yerinde tekrarlamaktan kurtuluyoruz.
	let containerElement: HTMLDivElement;
	let highlightElement: HTMLDivElement;
</script>

<SegmentedControl bind:value bind:containerElement bind:highlightElement>
	<slot />
</SegmentedControl>
