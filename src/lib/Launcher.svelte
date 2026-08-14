<script lang="ts">
	/**
	 * Ana ekran.
	 *
	 * Uygulama artık doğrudan editöre düşmüyor; önce burası açılıyor.
	 * Düzen openani.me'nin kendi sayfa iskeletini izliyor: geniş bir dış boşluk
	 * (site `2.5rem` kullanıyor), üstte başlık, altında `gap: 2rem` ile ayrılmış
	 * bölümler ve her bölümün kendi içinde `gap: .5rem`.
	 */
	import {
		Button,
		ComboBox,
		ContentDialog,
		IconButton,
		MenuFlyout,
		MenuFlyoutDivider,
		MenuFlyoutItem,
		ProgressRing,
		TextBlock,
		TextBox,
		Tooltip
	} from "fluent-svelte-extra";

	import Icon from "$lib/Icon.svelte";
	import StatusBar from "$lib/StatusBar.svelte";
	import { isEnter, type ForwardedKeyEvent } from "$lib/events";
	import {
		ImportError,
		fetchCss,
		resolveThemeFiles,
		suggestProjectName,
		type GithubFile
	} from "$lib/github";
	import { formatUpdated, type ProjectSummary } from "$lib/projects";

	export let projects: ProjectSummary[] = [];
	export let onOpen: (id: string) => void;
	export let onCreate: () => void;
	export let onOpenFile: () => void;
	export let onImport: (payload: { css: string; name: string; source: string }) => void;
	export let onRename: (id: string, name: string) => void;
	export let onDelete: (id: string) => void;
	export let onPreviewLogin: () => void;
	/** Önizlemede openani.me oturumu açık mı (Rust çerez kavanozundan okuyor). */
	export let loggedIn = false;
	export let onOpenAccount: () => void;

	// --- Kart önizlemesi -----------------------------------------------------
	// Kartta temanın gerçek accent rengini, sitenin gerçek zemin renginin
	// üstünde gösteriyoruz. Değerler uydurma değil: fluent-svelte-extra'nın
	// `--fds-solid-background-base` varsayılanları (bkz. PLAN.md §2.5).
	// "Sistem" modunda uygulamanın kendi zeminini kullanıyoruz ki kart
	// gerçekten sistemin o anki tercihini yansıtsın.
	const SURFACE: Record<string, string> = {
		light: "hsl(0, 0%, 100%)",
		dark: "hsl(0, 0%, 13%)",
		system: "var(--fds-solid-background-base)"
	};

	const accentCss = (accent: [number, number, number]) =>
		`hsl(${accent[0]}, ${accent[1]}%, ${accent[2]}%)`;

	const MODE_LABEL: Record<string, string> = {
		light: "Açık",
		dark: "Koyu",
		system: "Sistem"
	};

	// --- Yeniden adlandırma / silme -----------------------------------------
	let renaming: ProjectSummary | null = null;
	let renameValue = "";

	function startRename(project: ProjectSummary) {
		renaming = project;
		renameValue = project.name;
	}

	function confirmRename() {
		if (!renaming) return;
		const name = renameValue.trim();
		// Boş ada izin vermiyoruz; diyalog açık kalsın ki kullanıcı düzeltsin.
		if (!name) return;
		onRename(renaming.id, name);
		renaming = null;
	}

	let deleting: ProjectSummary | null = null;

	function confirmDelete() {
		if (!deleting) return;
		onDelete(deleting.id);
		deleting = null;
	}

	// --- GitHub'dan içe aktarma ---------------------------------------------
	let importOpen = false;
	let importUrl = "";
	let importName = "";
	let importError = "";
	let importBusy = false;
	/** Depoda birden fazla `.css` bulunduğunda kullanıcıya seçtiriyoruz. */
	let candidates: GithubFile[] = [];
	let chosenPath = "";
	let importSource = "";
	function cleanPathDisplay(p: string): string {
		return p ? (p.split("/").pop() ?? p) : "";
	}

	$: chosen = candidates.find((file) => file.path === chosenPath) ?? candidates[0];
	$: comboItems = candidates.map((file) => ({
		name: cleanPathDisplay(file.path),
		value: file.path
	}));
	$: if (chosen) {
		importName = suggestProjectName(chosen);
	}

	function resetImport() {
		importUrl = "";
		importName = "";
		importError = "";
		importBusy = false;
		candidates = [];
		chosenPath = "";
		importSource = "";
	}

	function openImport() {
		resetImport();
		importOpen = true;
	}

	/** Adım 1: bağlantıyı çözümle, aday `.css` dosyalarını bul. */
	async function resolve() {
		importError = "";
		importBusy = true;
		try {
			const result = await resolveThemeFiles(importUrl);
			candidates = result.files;
			importSource = result.source;
			chosenPath = result.files[0]?.path ?? "";
			if (result.files[0]) importName = suggestProjectName(result.files[0]);
		} catch (e) {
			candidates = [];
			importError = e instanceof ImportError ? e.message : String(e);
		} finally {
			importBusy = false;
		}
	}

	/** Adım 2: seçilen dosyayı indir ve projeye dönüştür. */
	async function runImport() {
		if (!chosen) return;
		importError = "";
		importBusy = true;
		try {
			const css = await fetchCss(chosen);
			onImport({
				css,
				name: importName.trim() || suggestProjectName(chosen),
				source: importSource || importUrl.trim()
			});
			importOpen = false;
			resetImport();
		} catch (e) {
			importError = e instanceof ImportError ? e.message : String(e);
		} finally {
			importBusy = false;
		}
	}

	function onUrlKey(event: ForwardedKeyEvent) {
		if (!isEnter(event)) return;
		// Enter, kullanıcının o an bulunduğu adımı ilerletir: henüz aday yoksa
		// bağlantıyı çözümler, varsa içe aktarır.
		if (candidates.length) runImport();
		else resolve();
	}
</script>

<div class="page">
	<header class="head">
		<TextBlock variant="title">Ana Sayfa</TextBlock>
		<TextBlock variant="body" class="text-secondary">
			Kayıtlı bir temayı açın, sıfırdan yeni bir tane oluşturun ya da GitHub'dan içe aktarın.
		</TextBlock>
	</header>

	<section class="section">
		<TextBlock variant="bodyStrong">Başla</TextBlock>
		<div class="actions">
			<Button variant="accent" on:click={onCreate}>
				<Icon name="add" size={16} /><span class="gap">Yeni tema oluştur</span>
			</Button>
			<Button on:click={openImport}>
				<Icon name="github" size={16} /><span class="gap">GitHub'dan içe aktar</span>
			</Button>
			<Button on:click={onOpenFile}>
				<Icon name="open" size={16} /><span class="gap">CSS dosyası aç…</span>
			</Button>
			{#if loggedIn}
				<Button on:click={onOpenAccount}>
					<Icon name="person" size={16} /><span class="gap">Hesap</span>
				</Button>
			{:else}
				<Button on:click={onPreviewLogin}>
					<Icon name="person" size={16} /><span class="gap">Giriş yap</span>
				</Button>
			{/if}
		</div>
	</section>

	<section class="section">
		<div class="row-between">
			<TextBlock variant="bodyStrong">
				Kayıtlı temalar{projects.length ? ` (${projects.length})` : ""}
			</TextBlock>
		</div>

		{#if projects.length === 0}
			<!--
				Boş durum: kullanıcıyı suçlamayan, tek bir sonraki adım öneren metin.
				Yapı sitenin boş durum bileşenlerinin aynısı (`.notification-empty-wrap`):
				dikey flex, ortalanmış, `gap: .5rem`, 48px'lik üçüncül renkte ikon,
				`type-body-strong` başlık ve `type-caption` + ikincil renkte açıklama.
			-->
			<div class="empty">
				<span class="empty-icon"><Icon name="emptyThemes" size={48} /></span>
				<TextBlock variant="bodyStrong">Henüz kayıtlı tema yok</TextBlock>
				<TextBlock variant="caption" class="text-secondary">
					Bir tema oluşturup editörde "Projeyi kaydet" dediğinizde burada listelenir.
				</TextBlock>
			</div>
		{:else}
			<div class="grid">
				{#each projects as project (project.id)}
					<div class="card">
						<!--
							Kartın gövdesi tek bir düğme: tıklanınca proje kaldığı
							yerden açılıyor. Menü düğmesi kasıtlı olarak bu düğmenin
							DIŞINDA — iç içe düğme geçersiz HTML olurdu ve klavyeyle
							gezinirken menüye hiç ulaşılamazdı.
						-->
						<button class="card-main" on:click={() => onOpen(project.id)}>
							<span
								class="preview"
								style="background: {SURFACE[project.mode] ?? SURFACE.dark}"
							>
								<span class="preview-bar" style="background: {accentCss(project.accent)}"></span>
								<span class="preview-dot" style="background: {accentCss(project.accent)}"></span>
							</span>
							<span class="card-text">
								<!-- Tarih/kaynak bilgisi üçüncül renkte: sitede de mod ve
								     süre bilgisi `.text-tertiary` taşıyor. -->
								<TextBlock variant="bodyStrong">{project.name}</TextBlock>
								<TextBlock variant="caption" class="text-tertiary">
									{MODE_LABEL[project.mode] ?? project.mode} · {formatUpdated(project.updatedAt)}
								</TextBlock>
								{#if project.source}
									<TextBlock variant="caption" class="text-tertiary">
										GitHub'dan içe aktarıldı
									</TextBlock>
								{/if}
							</span>
						</button>

						<div class="card-menu">
							<MenuFlyout placement="bottom" alignment="end">
								<Tooltip text="Proje işlemleri">
									<IconButton aria-label="{project.name} için işlemler">
										<Icon name="more" size={16} />
									</IconButton>
								</Tooltip>
								<svelte:fragment slot="flyout">
									<MenuFlyoutItem on:click={() => onOpen(project.id)}>Aç</MenuFlyoutItem>
									<MenuFlyoutItem on:click={() => startRename(project)}>
										Yeniden adlandır…
									</MenuFlyoutItem>
									<MenuFlyoutDivider />
									<MenuFlyoutItem on:click={() => (deleting = project)}>Sil…</MenuFlyoutItem>
								</svelte:fragment>
							</MenuFlyout>
						</div>
					</div>
				{/each}
			</div>
		{/if}
	</section>
</div>

<!-- --- GitHub içe aktarma diyaloğu ------------------------------------- -->
<ContentDialog bind:open={importOpen} title="GitHub'dan tema içe aktar" size="standard">
	<div class="dialog">
		<TextBlock variant="caption">
			Depo, klasör, dosya ya da gist bağlantısı girebilirsiniz. Örnekler:
		</TextBlock>
		<TextBlock variant="caption">
			<code>https://github.com/sahip/depo</code> ·
			<code>…/blob/main/tema.css</code> ·
			<code>sahip/depo</code>
		</TextBlock>

		<!-- svelte-ignore a11y-label-has-associated-control -->
		<label>
			<TextBlock variant="caption">Bağlantı</TextBlock>
			<TextBox
				bind:value={importUrl}
				placeholder="https://github.com/sahip/depo"
				on:keydown={onUrlKey}
				disabled={importBusy}
			/>
		</label>

		{#if candidates.length > 1}
			<TextBlock variant="caption">
				Depoda {candidates.length} CSS dosyası var — hangisi tema?
			</TextBlock>
			<ComboBox items={comboItems} bind:value={chosenPath} disabled={importBusy} />
		{:else if candidates.length === 1}
			<TextBlock variant="caption">
				<Icon name="file" size={12} /><span class="gap">{cleanPathDisplay(candidates[0].path)}</span>
			</TextBlock>
		{/if}

		{#if candidates.length}
			<!-- svelte-ignore a11y-label-has-associated-control -->
			<label>
				<TextBlock variant="caption">Proje adı</TextBlock>
				<TextBox bind:value={importName} disabled={importBusy} clearButton={false} />
			</label>
		{/if}

		{#if importError}
			<StatusBar severity="critical" title="İçe aktarılamadı" message={importError} closable={false} />
		{/if}

		{#if importBusy}
			<div class="busy">
				<ProgressRing size={20} />
				<TextBlock variant="caption">
					{candidates.length ? "Tema indiriliyor…" : "Depo taranıyor…"}
				</TextBlock>
			</div>
		{/if}

		<StatusBar severity="information" title="" closable={false}>
			<TextBlock variant="caption">
				Çekilen CSS'teki renk, köşe yarıçapı ve yazı tipi değerleri kontrollere otomatik
				eşlenir. Eşlenemeyen kurallar kaybolmaz — "Ham CSS" bölümünde olduğu gibi korunur.
			</TextBlock>
		</StatusBar>
	</div>

	<svelte:fragment slot="footer">
		{#if candidates.length}
			<Button variant="accent" disabled={importBusy} on:click={runImport}>
				<Icon name="download" size={14} /><span class="gap">İçe aktar</span>
			</Button>
		{:else}
			<Button variant="accent" disabled={importBusy || !importUrl.trim()} on:click={resolve}>
				Devam
			</Button>
		{/if}
		<Button on:click={() => (importOpen = false)}>Vazgeç</Button>
	</svelte:fragment>
</ContentDialog>

<!-- --- Yeniden adlandırma ---------------------------------------------- -->
<ContentDialog open={renaming !== null} title="Yeniden adlandır" size="standard">
	<!-- svelte-ignore a11y-label-has-associated-control -->
	<label>
		<TextBlock variant="caption">Proje adı</TextBlock>
		<TextBox
			bind:value={renameValue}
			clearButton={false}
			on:keydown={(e) => isEnter(e) && confirmRename()}
		/>
	</label>

	<svelte:fragment slot="footer">
		<Button variant="accent" disabled={!renameValue.trim()} on:click={confirmRename}>Kaydet</Button>
		<Button on:click={() => (renaming = null)}>Vazgeç</Button>
	</svelte:fragment>
</ContentDialog>

<!-- --- Silme onayı ------------------------------------------------------ -->
<ContentDialog open={deleting !== null} title="Proje silinsin mi?" size="standard">
	<TextBlock>
		<strong>{deleting?.name}</strong> kalıcı olarak silinecek. Bu işlem geri alınamaz.
	</TextBlock>
	<TextBlock variant="caption">
		Daha önce dışa aktardığınız .css dosyaları bundan etkilenmez.
	</TextBlock>

	<svelte:fragment slot="footer">
		<Button variant="accent" on:click={confirmDelete}>
			<Icon name="remove" size={14} /><span class="gap">Evet, sil</span>
		</Button>
		<Button on:click={() => (deleting = null)}>Vazgeç</Button>
	</svelte:fragment>
</ContentDialog>

<style>
	/* Görsel karar yok: tüm renk / kenarlık / yarıçap / gölge değerleri
	   --fds-* token'larından. Buradaki her şey yerleşim ve sitenin kendi
	   boşluk ölçekleri (dış boşluk 2.5rem, bölümler arası 2rem, bölüm içi
	   .5rem — openani.me'nin Ayarlar sayfasından ölçüldü). */
	.page {
		box-sizing: border-box;
		height: 100%;
		overflow-y: auto;
		padding: 2.5rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
		background-color: var(--fds-solid-background-base);
	}

	.head {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.row-between {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	/* Kart ızgarası: sabit sütun sayısı yok; pencere daraldıkça kartlar
	   kendiliğinden alt satıra iniyor. */
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 12px;
	}

	/* Sitenin `.card-wrapper` kartı: `overlay-corner-radius`, hover'da
	   `translateY(-2px)` + `--fds-flyout-shadow`. Süre kart hover'ı için
	   250ms (`--fds-control-normal-duration`), easing `cubic-bezier(0,0,0,1)`.
	   Sitenin kendi kuralı `transition: transform .2s` diyor; aradaki 50ms'yi
	   şartnamedeki ölçeğe yuvarlıyoruz ki uygulamada tek bir süre tablosu
	   olsun. */
	.card {
		position: relative;
		border-radius: var(--fds-overlay-corner-radius);
		background-color: var(--fds-card-background-default);
		border: 1px solid var(--fds-card-stroke-default);
		box-shadow: var(--fds-card-shadow);
		overflow: hidden;
		transition: transform var(--fds-control-normal-duration)
				var(--fds-control-fast-out-slow-in-easing),
			box-shadow var(--fds-control-normal-duration) var(--fds-control-fast-out-slow-in-easing);
	}

	.card:hover {
		transform: translateY(-2px);
		box-shadow: var(--fds-flyout-shadow);
	}

	/* Kartın tıklanabilir gövdesi. Zemini şeffaf: kartın kendi yüzeyi görünsün. */
	.card-main {
		appearance: none;
		width: 100%;
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px;
		border: none;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.card-main:hover {
		background-color: var(--fds-subtle-fill-secondary);
	}

	.card-main:active {
		background-color: var(--fds-subtle-fill-tertiary);
	}

	.card-main:focus-visible {
		outline: none;
		box-shadow: var(--fds-focus-stroke);
	}

	/* Mini önizleme: temanın zemini + accent rengi. */
	.preview {
		position: relative;
		display: block;
		height: 76px;
		border-radius: var(--fds-control-corner-radius);
		border: 1px solid var(--fds-control-stroke-default);
		overflow: hidden;
	}

	.preview-bar {
		position: absolute;
		left: 10px;
		top: 12px;
		width: 3px;
		height: 16px;
		border-radius: 3px;
	}

	.preview-dot {
		position: absolute;
		left: 24px;
		top: 14px;
		width: 44px;
		height: 12px;
		border-radius: var(--fds-control-corner-radius);
	}

	.card-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		/* Uzun proje adı kartı genişletmesin. */
		min-width: 0;
	}

	/* Menü düğmesi kartın sağ üstünde; gövde düğmesinin dışında. */
	.card-menu {
		position: absolute;
		top: 6px;
		right: 6px;
	}

	/* Sitenin `.notification-empty-wrap` / `.mobile-notification-empty`
	   kuralları: dikey flex, ortalanmış, `gap: .5rem`, üstte `2rem` boşluk.
	   Çerçeve YOK — sitenin boş durumları çizgisiz. */
	.empty {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 0.5rem;
		margin-top: 2rem;
		text-align: center;
	}

	/* Boş durum ikonu üçüncül renkte; metin renginden bağımsız olması şart,
	   yoksa başlıkla aynı beyazlıkta olup dikkati ondan çalıyor. */
	.empty-icon {
		display: inline-flex;
		color: var(--fds-text-tertiary);
	}

	.dialog {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	/* ContentDialog içinde ComboBox dropdown'u düzgün görünsün diye
	   z-index ve overflow düzeltmeleri. */
	.dialog :global(.combo-box-dropdown) {
		z-index: 200;
	}

	.busy {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	label {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.gap {
		margin-left: 6px;
	}
</style>
