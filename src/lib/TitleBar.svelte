<script lang="ts">
	// Özel başlık çubuğu. Pencere `decorations: false` ile açıldığı için
	// (tauri.conf.json) sürükleme, küçültme, büyütme ve kapatma işlerini
	// buradan yürütüyoruz.
	//
	// Sürükleme native tarafta: `data-tauri-drag-region` taşıyan bir elemana
	// basıldığında Tauri `start_dragging`'i kendisi çağırıyor. Öznitelik
	// yalnızca olayın HEDEFİNDE aranıyor, bu yüzden düğmeler kendiliğinden
	// hariç kalıyor — onlara ayrıca bir şey eklemek gerekmiyor.

	import { onDestroy, onMount } from "svelte";
	import { IconButton, TextBlock, Tooltip } from "fluent-svelte-extra";
	import { getCurrentWindow } from "@tauri-apps/api/window";

	import Icon from "$lib/Icon.svelte";

	/**
	 * Ortadaki bağlam metni — sitenin üst barında arama kutusunun durduğu yer.
	 *
	 * Uygulamada arama yok; oraya bir arama kutusu koymak işlevi olmayan bir
	 * kontrol göstermek olurdu. Sitenin düzenindeki o üçüncü bölge, o an
	 * bulunulan görünümün adıyla dolduruluyor.
	 */
	export let title = "";

	const appWindow = getCurrentWindow();

	/** Büyütme düğmesinin ikonu pencerenin GERÇEK durumunu göstermeli. */
	let maximized = false;

	let unlisten: (() => void) | null = null;

	async function sync() {
		maximized = await appWindow.isMaximized();
	}

	onMount(async () => {
		await sync();
		// Kullanıcı pencereyi kenardan çift tıklayarak ya da Win+Yukarı ile de
		// büyütebiliyor; durumu düğmemizden değil, pencerenin kendisinden
		// öğreniyoruz ki ikon her yolda doğru kalsın.
		unlisten = await appWindow.onResized(sync);
	});

	onDestroy(() => unlisten?.());
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="titlebar" data-tauri-drag-region>
	<!--
		Üç bölge, sitenin üst barındaki gibi:
		  sol   — logo + ad + rozet
		  orta  — bağlam alanı (sitede arama kutusu)
		  sağ   — ikon grubu (bizde pencere düğmeleri)
	-->
	<div class="brand" data-tauri-drag-region>
		<img class="app-icon" src="/app-icon.png" alt="" draggable="false" data-tauri-drag-region />
		<TextBlock variant="caption">OpenAnime</TextBlock>
		<!-- Sitedeki "Next-Gen" rozetinin karşılığı; stili birebir onunki. -->
		<span class="badge"><TextBlock variant="caption">Tema Editörü</TextBlock></span>
	</div>

	<div class="context" data-tauri-drag-region>
		{#if title}
			<TextBlock variant="caption">{title}</TextBlock>
		{/if}
	</div>

	<div class="controls">
		<Tooltip text="Simge durumuna küçült">
			<IconButton on:click={() => appWindow.minimize()} aria-label="Simge durumuna küçült">
				<Icon name="minimize" size={16} />
			</IconButton>
		</Tooltip>

		<Tooltip text={maximized ? "Geri yükle" : "Ekranı kapla"}>
			<IconButton
				on:click={async () => {
					await appWindow.toggleMaximize();
					await sync();
				}}
				aria-label={maximized ? "Geri yükle" : "Ekranı kapla"}
			>
				<Icon name={maximized ? "restore" : "maximize"} size={16} />
			</IconButton>
		</Tooltip>

		<Tooltip text="Kapat">
			<IconButton class="close" on:click={() => appWindow.close()} aria-label="Kapat">
				<Icon name="close" size={16} />
			</IconButton>
		</Tooltip>
	</div>
</div>

<style>
	/* Renk ve tipografi kararları --fds-* token'larından; buradaki tek sabit
	   değer kapatma düğmesinin kırmızısı (aşağıda gerekçesi var). */
	/* Izgara, çünkü orta bölge yan bölgelerin genişliğinden BAĞIMSIZ olarak
	   gerçekten ortalanmalı; `space-between` ile ortadaki metin sol taraf
	   uzadıkça kayardı. Sitenin üst barı da aynı üç bölgeli düzende. */
	.titlebar {
		flex: 0 0 auto;
		height: 40px;
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		padding: 0 4px 0 12px;
		box-sizing: border-box;
		gap: 8px;
		background-color: var(--fds-solid-background-tertiary);
		border-bottom: 1px solid var(--fds-divider-stroke-default);
		/* Başlık çubuğu bir sürükleme yüzeyi; metin seçimi imleci burada
		   sürüklemeyi bozuyor. */
		user-select: none;
		-webkit-user-select: none;
	}

	.brand {
		display: flex;
		align-items: center;
		min-width: 0;
		height: 100%;
	}

	/* Sitenin `.topbar .logo img` ölçüsü 1.1rem; sağındaki boşluk da onun. */
	.app-icon {
		width: 1.1rem;
		height: 1.1rem;
		margin-right: 0.5rem;
		flex: none;
	}

	/* Sitenin `#badge` kuralının birebir aynısı. */
	.badge {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0.05rem 0.25rem;
		margin-left: 0.5rem;
		border-radius: var(--fds-control-corner-radius);
		background: linear-gradient(
			135deg,
			hsl(var(--fds-accent-light-1)) 0%,
			var(--fds-accent-default) 100%
		);
		color: var(--fds-text-on-accent-primary);
		flex: none;
	}

	.badge :global(.text-block) {
		text-transform: uppercase;
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.5px;
	}

	/* Orta bölge: sitede arama kutusunun durduğu yer. */
	.context {
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 0;
		height: 100%;
		color: var(--fds-text-secondary);
		overflow: hidden;
		white-space: nowrap;
	}

	/* Sağdaki ikon grubu. Sitenin `.header-right` boşluğu .5rem, ama bunlar
	   pencere düğmeleri: Windows başlık çubuğu konvansiyonu gereği bitişik
	   duruyorlar, aksi hâlde OS kromu gibi görünmezlerdi. */
	.controls {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 2px;
		flex: none;
	}

	/* Kapatma düğmesi: Windows/Fluent konvansiyonu gereği hover'da kırmızı.
	   Bu bilinçli olarak --fds-accent-* DIŞINDA tutuldu — kullanıcının seçtiği
	   vurgu rengi ne olursa olsun "kapat" aynı sistem rengiyle uyarılmalı.
	   Değerler Windows 11'in başlık çubuğu kırmızısı.
	   `:global` şart: sınıf IconButton'ın kök elemanına gidiyor.

	   Seçici zinciri (.titlebar .controls) bilerek uzun: IconButton'ın kendi
	   kuralı `.icon-button:not(.animated):not(:disabled):not(.disabled):hover`
	   ile geliyor ve tek sınıfla yazınca özgüllük eşitleniyor, sıralama da
	   kütüphaneden yana çıkıyordu (hover gri kalıyordu). */
	.titlebar .controls :global(.icon-button.close:hover) {
		background-color: #c42b1c;
		color: #ffffff;
	}

	.titlebar .controls :global(.icon-button.close:active) {
		background-color: #b22a1d;
		color: #ffffff;
	}
</style>
