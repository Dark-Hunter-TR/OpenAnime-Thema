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
	import { fly } from "svelte/transition";
	import { cubicOut } from "svelte/easing";
	import { IconButton, TextBlock } from "fluent-svelte-extra";
	import Tooltip from "$lib/Tooltip.svelte";
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

	/**
	 * "Geri" oku. `null` ise hiç yok; bir fonksiyon verilirse logonun SOLUNDA
	 * belirir.
	 *
	 * Sitenin kendi üst çubuğu bunu BİREBİR böyle yapıyor (bundle'dan
	 * doğrulandı, `openanime-bzhM1JJJ.js`): ana sayfadayken logo tek başına
	 * bir bağlantı (`homepage-nav`), herhangi bir alt sayfadaysa
	 * (`route.id != "/"`) `fluent:arrow-left-24-regular` oku logonun önüne
	 * `fly({x:-10, duration:300, easing: <cubic-out>})` ile kayarak giriyor.
	 *
	 * Derlenmiş koddaki iz (`xn(e,Tn,{...},!0)` girişte, aynı örnek üstünde
	 * `.run(1)`/`.run(0)` ile yön değiştirilerek çıkışta) tek bir çift yönlü
	 * `transition:` yönergesinin imzası — Svelte'in ayrı `in:`/`out:`
	 * çiftinde ürettiği `create_in_transition`/`create_out_transition`
	 * değil. Aşağıda da bilerek `transition:fly` (tek yönerge) kullanılıyor;
	 * ayrı `in:fly`+`out:fly` görünüşte aynı dursa da yarıda kesilen bir
	 * geçişi TERSİNE çevirmek yerine sıfırdan başlatır.
	 *
	 * Bizde "alt sayfa" karşılığı Editör — Ana Sayfa/Ayarlar/Hakkında'dan
	 * girildiği için oradan geri dönmenin bir yolu olmalı.
	 */
	export let onBack: (() => void) | null = null;

	const appWindow = getCurrentWindow();

	/**
	 * macOS'ta pencere `decorations: true` + `titleBarStyle: "Overlay"` ile
	 * açılıyor (bkz. `tauri.macos.conf.json`), yani sistemin kendi trafik ışıkları
	 * duruyor. Kendi küçült/büyüt/kapat düğmelerimizi orada göstermek onların
	 * ikizini çizmek olurdu; bunun yerine düğmeleri gizleyip sola trafik
	 * ışıklarının kaplayacağı kadar dolgu bırakıyoruz.
	 *
	 * Tespit `navigator` üzerinden: `@tauri-apps/plugin-os` yalnızca bu tek
	 * bilgi için bir eklenti daha eklemeyi gerektirirdi ve WKWebView'in
	 * userAgent'ı bu konuda güvenilir.
	 */
	const isMac = typeof navigator !== "undefined" && /Mac/i.test(navigator.userAgent);

	/** Büyütme düğmesinin ikonu pencerenin gerçek durumunu göstermeli. */
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
<div class="titlebar" class:mac={isMac} data-tauri-drag-region>
	<!--
		Üç bölge, sitenin üst barındaki gibi:
		  sol   — logo + ad + rozet
		  orta  — bağlam alanı (sitede arama kutusu)
		  sağ   — ikon grubu (bizde pencere düğmeleri)
	-->
	<div class="brand" data-tauri-drag-region>
		{#if onBack}
			<div class="back-button" transition:fly={{ x: -10, duration: 300, easing: cubicOut }}>
				<Tooltip text="Ana sayfaya dön">
					<IconButton on:click={onBack} aria-label="Ana sayfaya dön">
						<Icon name="back" size={16} />
					</IconButton>
				</Tooltip>
			</div>
		{/if}
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

	{#if !isMac}
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
	{/if}
</div>

<style>
	/* Renk ve tipografi kararları --fds-* token'larından; buradaki tek sabit
	   değer kapatma düğmesinin kırmızısı (aşağıda gerekçesi var).

	   Yerleşim sitenin canlı `.topbar` kuralının birebir aynısı
	   (`openanime-CtLsnAnr.css`, `svelte-1jpbqve` kapsamı):

	     .topbar        { padding: .6rem; position: relative; width: 100%;
	                      z-index: 100; display: flex; align-items: center;
	                      justify-content: space-between;
	                      transition: all var(--fds-control-fast-duration) }
	     .topbar .logo  { display: flex; align-items: center; justify-content: center }
	     .topbar .logo img { margin-left: .75rem; margin-right: 1rem;
	                         width: 1.1rem; height: 1.1rem }
	     #search        { position: absolute; left: 50%; transform: translate(-50%);
	                      width: 40%; z-index: 1 }
	     .header-right  { display: flex; gap: .5rem; align-items: center }

	   Sitede üst barın kendi zemini ve alt kenarlığı YOK; altındaki gövde
	   rengini gösteriyor. Sabit yükseklik de yok: yükseklik .6rem dolgu +
	   en uzun içerikten (32px'lik hesap avatarı / bizde pencere düğmeleri)
	   çıkıyor. İkisini de aynen alıyoruz. */
	.titlebar {
		flex: 0 0 auto;
		position: relative;
		width: 100%;
		z-index: 100;
		display: flex;
		align-items: center;
		justify-content: space-between;
		transition: all var(--fds-control-fast-duration);
		/* Sağ dolgu 4px: pencere düğmeleri Windows konvansiyonu gereği
		   köşeye yakın durmalı, .6rem onları içeri kaçırırdı. */
		padding: 0.6rem 4px 0.6rem 0.6rem;
		box-sizing: border-box;
		/* Başlık çubuğu bir sürükleme yüzeyi; metin seçimi imleci burada
		   sürüklemeyi bozuyor. */
		user-select: none;
		-webkit-user-select: none;
	}

	/* macOS: sistemin trafik ışıkları başlık çubuğunun üstüne biniyor
	   (`titleBarStyle: "Overlay"`). 78px onların kapladığı şeritten biraz
	   geniş — logo tam bitişiğine değil, rahat bir boşluktan sonra başlıyor.
	   Sağdaki 4px'lik dar dolgu da kalkıyor: orada artık pencere düğmesi yok,
	   Windows konvansiyonunu macOS'a taşımanın anlamı olmaz. */
	.titlebar.mac {
		padding-left: 78px;
		padding-right: 0.6rem;
	}

	.titlebar.mac .app-icon {
		margin-left: 0;
	}

	/* Sitenin `.topbar .logo` kuralı. */
	.brand {
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 0;
		transition: all var(--fds-control-fast-duration);
	}

	/* Geri oku: logonun soluna, aralarında IconButton'ın kendi dolgusu yeterli
	   boşluğu bıraktığı için ek bir margin gerekmiyor — sitede de böyle. */
	.back-button {
		display: flex;
		align-items: center;
		flex: none;
	}

	/* Sitenin `.topbar .logo img` kuralı: 1.1rem kare, solda .75rem,
	   sağda 1rem boşluk. */
	.app-icon {
		width: 1.1rem;
		height: 1.1rem;
		margin-left: 0.75rem;
		margin-right: 1rem;
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

	/* Orta bölge: sitede arama kutusunun durduğu yer, onun geometrisiyle.
	   Akıştan çıkarılmış olması şart — `space-between` ile hizalansaydı sol
	   bölge uzadıkça (uzun proje adı) kayardı; sitede de bu yüzden mutlak. */
	.context {
		position: absolute;
		left: 50%;
		transform: translate(-50%);
		width: 40%;
		z-index: 1;
		transition: all ease-out 0.2s;
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 0;
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
