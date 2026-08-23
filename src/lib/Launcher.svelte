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
		ContentDialog,
		IconButton,
		MenuFlyout,
		MenuFlyoutDivider,
		MenuFlyoutItem,
		TextBlock,
		TextBox
	} from "fluent-svelte-extra";
	import Tooltip from "$lib/Tooltip.svelte";
	import { unclip } from "$lib/unclip";

	import GithubImportDialog from "$lib/GithubImportDialog.svelte";
	import Icon from "$lib/Icon.svelte";
	import { isEnter } from "$lib/events";
	import { formatUpdated, type ProjectSummary } from "$lib/projects";

	export let projects: ProjectSummary[] = [];
	export let onOpen: (id: string) => void;
	export let onCreate: () => void;
	export let onOpenFile: () => void;
	export let onImport: (payload: { css: string; name: string; source: string }) => void;
	export let onRename: (id: string, name: string) => void;
	export let onDelete: (id: string) => void;
	export let onSetCover: (id: string) => void;
	export let onRemoveCover: (id: string) => void;
	export let onExportCss: (id: string) => void;
	export let onLogin: () => void;
	/** openani.me oturumu açık mı (Rust, önizleme webview'inin çerez
	 * kavanozundan okuyor — kullanıcı için görünmez bir ayrıntı). */
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
	// Diyaloğun kendisi `GithubImportDialog.svelte`'de: Editör'e açık proje
	// olmadan girildiğinde `+page.svelte` de aynı diyaloğu açıyor.
	let importOpen = false;
</script>

<div class="page" use:unclip>
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
			<Button on:click={() => (importOpen = true)}>
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
				<Button on:click={onLogin}>
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
								{#if project.coverImage}
									<img class="preview-cover" src={project.coverImage} alt="" />
								{:else}
									<span class="preview-bar" style="background: {accentCss(project.accent)}"></span>
									<span class="preview-dot" style="background: {accentCss(project.accent)}"></span>
								{/if}
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
									<MenuFlyoutItem on:click={() => onExportCss(project.id)}>
										CSS olarak kaydet…
									</MenuFlyoutItem>
									<MenuFlyoutItem on:click={() => onSetCover(project.id)}>
										{project.coverImage ? "Kapak görselini değiştir…" : "Kapak görseli ekle…"}
									</MenuFlyoutItem>
									{#if project.coverImage}
										<MenuFlyoutItem on:click={() => onRemoveCover(project.id)}>
											Kapak görselini kaldır
										</MenuFlyoutItem>
									{/if}
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
<GithubImportDialog bind:open={importOpen} {onImport} />

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
		/* `overflow: hidden` BİLEREK burada değil: `.card-menu`'nün açılır menüsü
		   de bu kartın bir çocuğu ve menü kartın altına taşacak kadar uzunsa
		   (CSS/kapak görseli eylemleri eklendikten sonra artık öyle) burada
		   kesilip görünmez olurdu. Köşe yuvarlaklığının klibi yerine, ona
		   gerçekten ihtiyacı olan tek şeye — `.card-main`'in hover dolgusuna —
		   taşındı. */
		transition: transform var(--fds-control-normal-duration)
				var(--fds-control-fast-out-slow-in-easing),
			box-shadow var(--fds-control-normal-duration) var(--fds-control-fast-out-slow-in-easing);
	}

	.card:hover {
		transform: translateY(-2px);
		box-shadow: var(--fds-flyout-shadow);
	}

	/* Kartın tıklanabilir gövdesi. Zemini şeffaf: kartın kendi yüzeyi görünsün.
	   Köşe yuvarlaklığı ve klip burada — `.card` yerine — çünkü kesilmesi
	   gereken tek şey bu (hover dolgusu, kapak görseli); menü kartın DIŞINDA. */
	.card-main {
		appearance: none;
		width: 100%;
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px;
		border: none;
		border-radius: var(--fds-overlay-corner-radius);
		overflow: hidden;
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

	.preview-cover {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
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

	label {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.gap {
		margin-left: 6px;
	}
</style>
