<script lang="ts">
	import { ICONS, type IconName } from "$lib/icons";

	export let name: IconName;
	/** Kenar uzunluğu (px). Fluent ikonları 24'lük ızgarada çizilmiş. */
	export let size = 16;
</script>

<span class="icon" style="width:{size}px;height:{size}px">
	{@html ICONS[name]}
</span>

<style>
	/* Yalnızca ölçek; renk currentColor üzerinden --fds-* metin renginden gelir. */
	.icon {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex: none;
	}

	.icon :global(svg) {
		width: 100%;
		height: 100%;
		fill: currentColor;
	}
</style>
