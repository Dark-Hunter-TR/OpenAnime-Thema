<script lang="ts">
	/**
	 * Uygulamanın kendi ayarlar sayfası.
	 *
	 * Düzeni openani.me'nin `/settings` sayfasından ölçüldü (canlı CSS'ten):
	 *
	 *   .settings   { margin: 2.5rem }
	 *   .expanders  { display: flex; flex-direction: column; gap: 2rem }
	 *   .expand-section { display: flex; flex-direction: column; gap: .5rem }
	 *   .item-header    { display: flex; flex-direction: column }   ← başlık + açıklama
	 *
	 * Yani: bölüm başlığı, altında `Expander` satırları, her satırda solda
	 * başlık + açıklama, sağda kontrol. Sitenin i18n anahtarları da bu
	 * bölümlemeyi doğruluyor (`settings.appearance.*`, `settings.performance.*`).
	 * Kopya değil, aynı tasarım dilinin devamı: bizim bölümlerimiz uygulamanın
	 * kendi ayarları.
	 */
	import { createEventDispatcher } from "svelte";
	import {
		Button,
		ComboBox,
		Expander,
		SegmentedControlButton,
		TextBlock,
		ToggleSwitch,
		Tooltip
	} from "fluent-svelte-extra";

	import AccountCard from "$lib/AccountCard.svelte";
	import Icon from "$lib/Icon.svelte";
	import StatusBar from "$lib/StatusBar.svelte";
	import SegmentedControl from "$lib/Segmented.svelte";
	import { ROUTES, VIEWPORTS } from "$lib/routes";
	import type { AppSettings } from "$lib/settings";

	export let settings: AppSettings;
	export let projectCount = 0;
	export let projectsPath = "";
	export let appVersion = "";
	export let onOpenProjectsFolder: () => void;
	export let onPreviewLogin: () => void;
	/** Önizlemede openani.me oturumu açık mı (Rust çerez kavanozundan okuyor). */
	export let loggedIn = false;

	const dispatch = createEventDispatcher<{ change: AppSettings }>();

	function notifyChange() {
		dispatch("change", settings);
	}

	$: if (settings) {
		notifyChange();
	}

	const routeItems = ROUTES.map((route) => ({
		name: route.auth ? `${route.name} (giriş gerekir)` : route.name,
		value: route.path
	}));

	const viewportItems = VIEWPORTS.map((vp) => ({ name: vp.name, value: vp.id }));
</script>

<div class="page">
	<header class="head">
		<TextBlock variant="title">Ayarlar</TextBlock>
		<TextBlock variant="body" class="text-secondary">
			Uygulamanın kendi tercihleri. Düzenlediğiniz temayı etkilemez.
		</TextBlock>
	</header>

	<div class="sections">
		<!-- --- Görünüm ------------------------------------------------------ -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong">Görünüm</TextBlock>

			<Expander expandable={false}>
				<Icon slot="icon" name="appearance" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">Uygulama teması</TextBlock>
						<TextBlock variant="caption">
							Editör arayüzünün rengi. Önizlemedeki sitenin teması bundan bağımsızdır —
							onu "Görünüm" bölümünden ayarlarsınız.
						</TextBlock>
					</span>
					<span class="item-action">
						<SegmentedControl bind:value={settings.appTheme}>
							<SegmentedControlButton
								value="system"
								on:click={() => (settings.appTheme = "system")}
							>
								Sistem
							</SegmentedControlButton>
							<SegmentedControlButton
								value="light"
								on:click={() => (settings.appTheme = "light")}
							>
								Açık
							</SegmentedControlButton>
							<SegmentedControlButton value="dark" on:click={() => (settings.appTheme = "dark")}>
								Koyu
							</SegmentedControlButton>
						</SegmentedControl>
					</span>
				</div>
			</Expander>
		</section>

		<!-- --- Tema editörü ------------------------------------------------- -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong">Tema editörü</TextBlock>

			<Expander expandable={false}>
				<Icon slot="icon" name="code" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">Açılış düzenleme modu</TextBlock>
						<TextBlock variant="caption">Bir proje açtığınızda hangi sekmeyle başlanacağı.</TextBlock>
					</span>
					<span class="item-action">
						<SegmentedControl bind:value={settings.defaultEditMode}>
							<SegmentedControlButton
								value="visual"
								on:click={() => (settings.defaultEditMode = "visual")}
							>
								Görsel
							</SegmentedControlButton>
							<SegmentedControlButton
								value="code"
								on:click={() => (settings.defaultEditMode = "code")}
							>
								Kod
							</SegmentedControlButton>
						</SegmentedControl>
					</span>
				</div>
			</Expander>

			<Expander expandable={false}>
				<Icon slot="icon" name="viewport" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">Varsayılan önizleme genişliği</TextBlock>
						<TextBlock variant="caption">
							Önizleme webview'i gerçekten daraltılır, yani sitenin kendi medya sorguları
							tetiklenir.
						</TextBlock>
					</span>
					<span class="item-action">
						<ComboBox items={viewportItems} bind:value={settings.defaultViewport} />
					</span>
				</div>
			</Expander>

			<Expander expandable={false}>
				<Icon slot="icon" name="page" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">Varsayılan önizleme sayfası</TextBlock>
						<TextBlock variant="caption">Editör açıldığında önizlemenin gideceği sayfa.</TextBlock>
					</span>
					<span class="item-action">
						<ComboBox items={routeItems} bind:value={settings.defaultPreviewPath} />
					</span>
				</div>
			</Expander>

			<Expander expandable={false}>
				<Icon slot="icon" name="save" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">Ana ekrana dönerken otomatik kaydet</TextBlock>
						<TextBlock variant="caption">
							Kapalıyken, kaydedilmemiş değişikliklerle ayrılmak üzereyken onay istenir.
						</TextBlock>
					</span>
					<span class="item-action">
						<ToggleSwitch bind:checked={settings.autoSaveOnLeave} />
					</span>
				</div>
			</Expander>
		</section>

		<!-- --- Hesap -------------------------------------------------------- -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong">Hesap</TextBlock>

			{#if loggedIn}
				<AccountCard {onPreviewLogin} />
			{:else}
				<Expander expandable={false}>
					<Icon slot="icon" name="avatar" size={20} />
					<div class="item">
						<span class="item-header">
							<TextBlock variant="body">OpenAnime hesabı</TextBlock>
							<TextBlock variant="caption">Önizlemede oturum açık değil.</TextBlock>
						</span>
						<span class="item-action">
							<Button variant="accent" on:click={onPreviewLogin}>
								<Icon name="person" size={16} /><span class="gap">Önizlemede giriş yap</span>
							</Button>
						</span>
					</div>
				</Expander>
			{/if}
		</section>

		<!-- --- Depolama ----------------------------------------------------- -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong">Depolama</TextBlock>

			<Expander expandable={false}>
				<Icon slot="icon" name="open" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">Proje klasörü</TextBlock>
						<TextBlock variant="caption">
							{projectCount} kayıtlı proje. Her proje tek bir JSON dosyası; klasörü
							kopyalayarak yedekleyebilirsiniz.
						</TextBlock>
						{#if projectsPath}
							<TextBlock variant="caption"><code>{projectsPath}</code></TextBlock>
						{/if}
					</span>
					<span class="item-action">
						<Button on:click={onOpenProjectsFolder} disabled={!projectsPath}>Klasörü aç</Button>
					</span>
				</div>
			</Expander>
		</section>

		<!-- --- Hakkında ----------------------------------------------------- -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong">Hakkında</TextBlock>

			<Expander expandable={false}>
				<Icon slot="icon" name="navAbout" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">OpenAnime Tema Editörü</TextBlock>
						<TextBlock variant="caption">
							Sürüm {appVersion || "v0.1.0"}
						</TextBlock>
					</span>
				</div>
			</Expander>
		</section>
	</div>
</div>

<style>
	/* Görsel karar yok; ölçüler openani.me'nin Ayarlar sayfasından alındı.
	   Renk ve yüzey değerlerinin tamamı --fds-* token'larından geliyor. */
	.page {
		box-sizing: border-box;
		height: 100%;
		overflow-y: auto;
		padding: 2.5rem;
		background-color: var(--fds-solid-background-base);
	}

	.head {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	/* Sitenin `.expanders` kuralı: bölümler arası 2rem. */
	.sections {
		margin-top: 1rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	/* Sitenin `.expand-section` kuralı: bölüm içi .5rem. */
	.expand-section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	/* Satır: solda başlık+açıklama, sağda kontrol. */
	.item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
		flex-wrap: wrap;
	}

	/* Sitenin `.item-header` kuralı. */
	.item-header {
		display: flex;
		flex-direction: column;
		padding-top: 0.35rem;
		padding-bottom: 0.35rem;
		/* Açıklama uzun; kontrolü ezmesin ama satırı da taşırmasın. */
		flex: 1 1 320px;
		min-width: 0;
	}

	/* Her satırda üstte `type-body` başlık, altında `type-caption` açıklama var.
	   Açıklama ikincil renkte olmalı (rapordaki hiyerarşi: "açıklama metinleri
	   → --fds-text-secondary"). Tek tek sınıf yazmak yerine kural burada:
	   desen her satırda tekrar ediyor ve bir satır eklendiğinde unutulmasın. */
	.item-header :global(.text-block.type-caption) {
		color: var(--fds-text-secondary);
	}

	.item-action {
		flex: 0 0 auto;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.gap {
		margin-left: 6px;
	}

	:global(.combo-box-dropdown) {
		scrollbar-width: none !important;
		-ms-overflow-style: none !important;
	}

	:global(.combo-box-dropdown::-webkit-scrollbar) {
		display: none !important;
		width: 0 !important;
		height: 0 !important;
		background: transparent !important;
	}
</style>
