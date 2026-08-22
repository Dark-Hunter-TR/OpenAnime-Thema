<script lang="ts">
	/**
	 * Uygulamanın kendi ayarlar sayfası.
	 *
	 * Düzen ve DOM yapısı openani.me'nin `/settings` sayfasından BİREBİR alındı.
	 * Sayfanın gerçek iskeleti (canlı bundle'dan, `svelte-ndcra2` hash'i):
	 *
	 *   .settings
	 *     ├ TextBlock variant="title"          ("Ayarlar" — alt başlık yok)
	 *     ├ <hr class="horizontal" style="margin: 1rem 0px;">
	 *     └ .expanders
	 *         └ .expand-section                 (bölüm başına bir tane)
	 *             ├ TextBlock class="title" variant="bodyStrong"
	 *             └ .expander …
	 *
	 * Sitede İKİ tür satır var ve ayrımı kaçırmamak önemli:
	 *
	 *   1) Katlanmayan satır — `expandable={false}` + `class="space-between"`
	 *      .expander-header
	 *        ├ .expander-icon                   (20x20 — fluent'in 16px'i EZİLİYOR)
	 *        └ .expander-header-title           (display:flex; space-between)
	 *            ├ .item-header                 (başlık + açıklama, dikey)
	 *            └ .expander-control            (kontrol; gap .5rem)
	 *
	 *   2) Katlanan satır — hiçbir prop verilmiyor (varsayılan `expandable`)
	 *      ve `space-between` yok: başlıkta kontrol durmuyor, kontroller
	 *      açılan gövdeye iniyor.
	 *      .expander-content                    (fluent'in padding'i 0'a çekilmiş)
	 *        └ .item                            (padding: .5rem 3rem; border-top)
	 *
	 * Sitede (1) tekil ayarlar için, (2) ise birbirine bağlı ayarları tek
	 * satırda toplamak için kullanılıyor — "Tema" (mod seçimi + özel tema
	 * yükleme) ve "Kişiselleştirilmiş öneriler" (üç alt anahtar) böyle.
	 * Buradaki düzen de aynı mantığı izliyor: "Önizleme" ve "Güncellemeler"
	 * katlanan satır, geri kalanı tekil.
	 *
	 * Bölüm sırası da siteden: orada "Hesap" en üstteki bölüm
	 * (Hesap → Cihazlar → Profil → Performans → Görünüm → Oynatıcı → Gelişmiş).
	 *
	 * `space-between` fluent'in bir prop'u değil: site onu `class` ile
	 * geçiriyor ve kuralını kendi yazıyor (bkz. aşağıdaki style bloğu).
	 */
	import { createEventDispatcher } from "svelte";
	import { Button, ComboBox, Expander, TextBlock, ToggleSwitch } from "fluent-svelte-extra";

	import AccountCard from "$lib/AccountCard.svelte";
	import Icon from "$lib/Icon.svelte";
	import { ROUTES, VIEWPORTS } from "$lib/routes";
	import type { AppSettings } from "$lib/settings";

	export let settings: AppSettings;
	export let projectCount = 0;
	export let projectsPath = "";
	export let appVersion = "";
	export let onOpenProjectsFolder: () => void;
	export let onLogin: () => void;
	/** Çıkış tamamlandığında oturum durumunu tazelemek için. */
	export let onLoggedOut: () => void;
	/** openani.me oturumu açık mı (Rust, önizleme webview'inin çerez
	 * kavanozundan okuyor — kullanıcı için görünmez bir ayrıntı). */
	export let loggedIn = false;
	/**
	 * Önizlemenin Sistem/Açık/Koyu modu — editördeki floating seçiciyle AYNI
	 * değeri gösterir, aynı `setMode`'u çağırır (ikinci bir uygulama yolu yok)
	 * ve açık proje olup olmadığından bağımsız olarak her zaman değiştirilebilir;
	 * bu, uygulamanın kendi arayüz temasından (`settings.appTheme`) ayrı bir
	 * ayar.
	 */
	export let themeMode: "system" | "light" | "dark" = "dark";
	export let onThemeModeChange: (mode: "system" | "light" | "dark") => void;
	export let onCheckForUpdates: () => void;
	/** Kontrol mantığı `+page.svelte`'de yaşıyor (Ayarlar'daki düğme ile
	 * açılıştaki otomatik kontrol AYNI kodu paylaşıyor) — burası yalnızca
	 * sonucu gösteriyor. */
	export let updateCheckStatus:
		| "idle"
		| "checking"
		| "up-to-date"
		| "channel-empty"
		| "error" = "idle";
	export let updateCheckError = "";
	/** Son kontrolün kanal adı; "bu kanalda sürüm yok" metni için. */
	export let updateChannelLabel = "";
	/** Kanal değiştiğinde: atlanan sürümü sıfırlayıp yeniden kontrol eder. */
	export let onChannelChange: () => void;

	const dispatch = createEventDispatcher<{ change: AppSettings }>();

	function notifyChange() {
		dispatch("change", settings);
	}

	$: if (settings) {
		notifyChange();
	}

	/**
	 * Anahtarların yanındaki "Etkin / Devre Dışı" etiketi — sitede her
	 * `ToggleSwitch`'in SOLUNDA duruyor (`.expander-control` içinde,
	 * `type-body` metin olarak). Sitenin `general.enabled` / `general.disabled`
	 * çevirileri de tam olarak bu iki kelime.
	 */
	const onOff = (value: boolean) => (value ? "Etkin" : "Devre Dışı");

	const routeItems = ROUTES.map((route) => ({
		name: route.auth ? `${route.name} (giriş gerekir)` : route.name,
		value: route.path
	}));

	const viewportItems = VIEWPORTS.map((vp) => ({ name: vp.name, value: vp.id }));

	const editorStartupActionItems = [
		{ name: "Her zaman sor", value: "ask" },
		{ name: "Yeni tema oluştur", value: "new" },
		{ name: "GitHub'dan içe aktar", value: "github" },
		{ name: "CSS dosyası aç", value: "file" }
	];

	const themeModeItems = [
		{ name: "Sistem", value: "system" },
		{ name: "Açık", value: "light" },
		{ name: "Koyu", value: "dark" }
	];

	const editModeItems = [
		{ name: "Görsel", value: "visual" },
		{ name: "Kod", value: "code" }
	];

	const updateChannelItems = [
		{ name: "Stable", value: "stable" },
		{ name: "Beta", value: "beta" },
		{ name: "Alpha", value: "alpha" }
	];

	const discordScopeItems = [
		{ name: "Her zaman", value: "always" },
		{ name: "Sadece düzenlerken", value: "editor" }
	];

	/**
	 * ComboBox'ın `select` olayı sadece gerçek bir kullanıcı seçiminde değil,
	 * bileşen İLK KURULDUĞUNDA da bir kere kendiliğinden ateşleniyor
	 * (fluent-svelte-extra: `value` değişimini izleyen reaktif blok, önceki
	 * değeri `Symbol("init")` ile başlatıyor — ilk karşılaştırma her zaman
	 * "değişti" sayılıyor). `onChannelChange` bir AĞ isteği tetiklediği için
	 * (`runUpdateCheck`) bu ilk ateşlemeyi yutmazsak Ayarlar sayfası her
	 * açıldığında sessizce bir güncelleme kontrolü daha yapardı.
	 */
	let channelSelectPrimed = false;
	function handleChannelSelect(event: CustomEvent<{ value: AppSettings["updateChannel"] }>) {
		settings.updateChannel = event.detail.value;
		if (channelSelectPrimed) onChannelChange();
		channelSelectPrimed = true;
	}

	/**
	 * Katlanan satırların başlığında seçili değerleri özetliyoruz.
	 *
	 * Katlanan satırın başlığında kontrol duramıyor (sitenin kuralı, bkz.
	 * yukarıdaki not) — o yüzden satır kapalıyken ne seçili olduğu hiç
	 * görünmezdi. Özet, açmadan da okunabilsin diye açıklama satırında duruyor.
	 */
	$: viewportSummary = VIEWPORTS.find((vp) => vp.id === settings.defaultViewport)?.name ?? "";
	$: routeSummary =
		ROUTES.find((route) => route.path === settings.defaultPreviewPath)?.name ??
		settings.defaultPreviewPath;
	$: channelName =
		settings.updateChannel === "beta"
			? "Beta"
			: settings.updateChannel === "alpha"
				? "Alpha"
				: "Stable";
	$: discordSummary = !settings.discordRpc
		? "Kapalı"
		: settings.discordRpcScope === "editor"
			? "Yalnızca tema düzenlerken görünüyorsun"
			: "Discord'da görünüyorsun";
	$: editorStartupSummary = !settings.editorQuickStart
		? "Kapalı — her girişte sorulur"
		: settings.editorStartupAction === "new"
			? "Doğrudan yeni tema oluşturur"
			: settings.editorStartupAction === "github"
				? "Doğrudan GitHub'dan içe aktarır"
				: settings.editorStartupAction === "file"
					? "Doğrudan CSS dosyası açar"
					: "Her girişte sorulur";
</script>

<div class="page">
	<!-- Sitede başlık tek satır: `TextBlock variant="title"` + hemen altında
	     `<hr class="horizontal" style="margin: 1rem 0px;">`. Alt başlık yok. -->
	<TextBlock variant="title">Ayarlar</TextBlock>
	<hr class="horizontal margin" />

	<div class="sections">
		<!-- --- Hesap -------------------------------------------------------- -->
		<!-- Sitede olduğu gibi EN ÜSTTEKİ bölüm. Oturum açıkken profil kartı
		     (`AccountCard`) olduğu gibi duruyor; kapalıyken sitenin giriş
		     satırının aynısı (`fluent:person-20-regular` ikonu + hyperlink). -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong" class="title">Hesap</TextBlock>

			{#if loggedIn}
				<AccountCard {onLogin} {onLoggedOut} />
			{:else}
				<Expander class="space-between" expandable={false}>
					<Icon slot="icon" name="person" size={20} />
					<div class="item-header">
						<TextBlock variant="body">OpenAnime hesabı</TextBlock>
						<TextBlock variant="caption" class="text-secondary">
							Oturum açık değil.
						</TextBlock>
					</div>
					<div class="expander-control">
						<Button variant="hyperlink" on:click={onLogin}>Giriş yap</Button>
					</div>
				</Expander>
			{/if}
		</section>

		<!-- --- Discord ---------------------------------------------------------- -->
		<!-- Katlanan satır: ana anahtar ve onu özelleştiren iki ayar tek başlık
		     altında (sitenin "Kişiselleştirilmiş öneriler" kalıbı). Hesap'ın
		     hemen altında, çünkü ikisi de dış servislerle ilgili; depolama ve
		     güncelleme gibi bakım ayarları aşağıda kalmalı. -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong" class="title">Discord</TextBlock>

			<Expander>
				<Icon slot="icon" name="discord" size={20} />
				<div class="item-header">
					<TextBlock variant="body">Rich Presence</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						{discordSummary}
					</TextBlock>
				</div>

				<svelte:fragment slot="content">
					<div class="items">
						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Ne yaptığımı Discord'da göster</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Hangi bölümde olduğun Discord profilinde görünür. Discord
									kapalıysa hiçbir şey olmaz.
								</TextBlock>
							</div>
							<div class="expander-control">
								<TextBlock variant="body">{onOff(settings.discordRpc)}</TextBlock>
								<ToggleSwitch bind:checked={settings.discordRpc} />
							</div>
						</div>

						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Ne zaman görünsün</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									"Sadece düzenlerken" seçilirse ana ekranda, ayarlarda ve
									hakkında ekranında aktivite tamamen kaldırılır.
								</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox
									items={discordScopeItems}
									bind:value={settings.discordRpcScope}
									disabled={!settings.discordRpc}
								/>
							</div>
						</div>

						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Tema adını göster</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Kapalıyken düzenlediğin temanın adı yerine yalnızca "Tema
									düzenliyor" yazar.
								</TextBlock>
							</div>
							<div class="expander-control">
								<TextBlock variant="body">{onOff(settings.discordRpcThemeName)}</TextBlock>
								<ToggleSwitch
									bind:checked={settings.discordRpcThemeName}
									disabled={!settings.discordRpc}
								/>
							</div>
						</div>
					</div>
				</svelte:fragment>
			</Expander>
		</section>

		<!-- --- Görünüm ------------------------------------------------------ -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong" class="title">Görünüm Ayarları</TextBlock>

			<!-- İkon siteninkiyle aynı: `/settings`'teki "Tema" satırı
			     `fluent:color-20-regular` kullanıyor (bizde `accent`). Katlanan
			     yapı da sitenin kendi "Tema" satırıyla aynı — tek fark sitedeki
			     "Özel Tema" satırının burada karşılığı yok, uygulamanın kendi
			     teması özelleştirilebilir değil. -->
			<Expander>
				<Icon slot="icon" name="accent" size={20} />
				<div class="item-header">
					<TextBlock variant="body">Tema</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						Uygulamanın kendi görünümü ve önizlediğiniz sitenin modu — ikisi
						birbirinden bağımsız, ikisi de aşağıdan ayarlanır.
					</TextBlock>
				</div>

				<svelte:fragment slot="content">
					<div class="items">
						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Uygulamanın modu</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Editör arayüzünün rengi.
								</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox items={themeModeItems} bind:value={settings.appTheme} />
							</div>
						</div>

						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Önizlemenin modu</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Editördeki floating seçiciyle aynı kontrol — uygulamanın modundan
									bağımsız, açık proje olmasa da değiştirilebilir.
								</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox
									items={themeModeItems}
									value={themeMode}
									on:select={(e) => onThemeModeChange(e.detail.value)}
								/>
							</div>
						</div>
					</div>
				</svelte:fragment>
			</Expander>
		</section>

		<!-- --- Tema editörü ------------------------------------------------- -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong" class="title">Tema editörü</TextBlock>

			<!-- Discord Rich Presence bölümüyle aynı kalıp: üst satır ana anahtar
			     (açık/kapalı), alt satır yalnızca açıkken anlamlı olan seçim. -->
			<Expander>
				<Icon slot="icon" name="navEditor" size={20} />
				<div class="item-header">
					<TextBlock variant="body">Editör'e hızlı başlama</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						{editorStartupSummary}
					</TextBlock>
				</div>

				<svelte:fragment slot="content">
					<div class="items">
						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Sol menüdeki Editör düğmesine tıklayınca</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Kapalıyken (varsayılan) her tıklamada "Ne yapmak istersiniz?" seçicisi
									çıkar. Açıkken seçici atlanır, aşağıda seçtiğiniz eylem doğrudan çalışır.
								</TextBlock>
							</div>
							<div class="expander-control">
								<TextBlock variant="body">{onOff(settings.editorQuickStart)}</TextBlock>
								<ToggleSwitch bind:checked={settings.editorQuickStart} />
							</div>
						</div>

						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Doğrudan çalışacak eylem</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									"Her zaman sor" seçiliyse üstteki anahtar açık olsa bile seçici yine
									çıkar — kapatmadan seçiciyi geri getirmenin yolu bu.
								</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox
									items={editorStartupActionItems}
									bind:value={settings.editorStartupAction}
									disabled={!settings.editorQuickStart}
								/>
							</div>
						</div>
					</div>
				</svelte:fragment>
			</Expander>

			<Expander>
				<Icon slot="icon" name="code" size={20} />
				<div class="item-header">
					<TextBlock variant="body">Açılış düzenleme modu</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						Bir proje açtığınızda hangi sekmeyle başlanacağı.
					</TextBlock>
				</div>

				<svelte:fragment slot="content">
					<div class="items">
						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Sekme</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox items={editModeItems} bind:value={settings.defaultEditMode} />
							</div>
						</div>
					</div>
				</svelte:fragment>
			</Expander>

			<!-- Katlanan satır: önizlemenin iki açılış ayarı tek başlık altında.
			     Sitenin "Tema" satırıyla aynı kalıp — `expandable` varsayılan,
			     `space-between` yok, kontroller `.item` satırlarında. -->
			<Expander>
				<Icon slot="icon" name="viewport" size={20} />
				<div class="item-header">
					<TextBlock variant="body">Önizleme açılışı</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						{viewportSummary} • {routeSummary}
					</TextBlock>
				</div>

				<svelte:fragment slot="content">
					<div class="items">
						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Genişlik</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Önizleme webview'i gerçekten daraltılır, yani sitenin kendi medya sorguları
									tetiklenir.
								</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox items={viewportItems} bind:value={settings.defaultViewport} />
							</div>
						</div>

						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Açılış sayfası</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Editör açıldığında önizlemenin gideceği adres.
								</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox items={routeItems} bind:value={settings.defaultPreviewPath} />
							</div>
						</div>
					</div>
				</svelte:fragment>
			</Expander>

			<Expander class="space-between" expandable={false}>
				<Icon slot="icon" name="save" size={20} />
				<div class="item-header">
					<TextBlock variant="body">Ana ekrana dönerken otomatik kaydet</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						Kapalıyken, kaydedilmemiş değişikliklerle ayrılmak üzereyken onay istenir.
					</TextBlock>
				</div>
				<div class="expander-control">
					<TextBlock variant="body">{onOff(settings.autoSaveOnLeave)}</TextBlock>
					<ToggleSwitch bind:checked={settings.autoSaveOnLeave} />
				</div>
			</Expander>
		</section>

		<!-- --- Depolama ----------------------------------------------------- -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong" class="title">Depolama</TextBlock>

			<Expander class="space-between" expandable={false}>
				<Icon slot="icon" name="open" size={20} />
				<div class="item-header">
					<TextBlock variant="body">Proje klasörü</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						{projectCount} kayıtlı proje. Her proje tek bir JSON dosyası; klasörü kopyalayarak
						yedekleyebilirsiniz.
					</TextBlock>
					{#if projectsPath}
						<TextBlock variant="caption" class="text-secondary">
							<code>{projectsPath}</code>
						</TextBlock>
					{/if}
				</div>
				<div class="expander-control">
					<Button variant="hyperlink" on:click={onOpenProjectsFolder} disabled={!projectsPath}>
						Klasörü aç
					</Button>
				</div>
			</Expander>
		</section>

		<!-- --- Güncellemeler -------------------------------------------------- -->
		<!-- Tek katlanan satır: başlıkta sürüm/durum, gövdede hem otomatik
		     kontrol anahtarı hem elle kontrol düğmesi. Sitedeki
		     "Kişiselleştirilmiş öneriler" satırının kalıbı. -->
		<section class="expand-section">
			<TextBlock variant="bodyStrong" class="title">Güncellemeler</TextBlock>

			<Expander>
				<Icon slot="icon" name="update" size={20} />
				<div class="item-header">
					<TextBlock variant="body">OpenAnime Tema Editörü</TextBlock>
					<TextBlock variant="caption" class="text-secondary">
						{#if updateCheckStatus === "checking"}
							Kontrol ediliyor…
						{:else if updateCheckStatus === "up-to-date"}
							{appVersion || "Sürüm bilinmiyor"} — {channelName} kanalında güncel
						{:else if updateCheckStatus === "channel-empty"}
							{updateChannelLabel || channelName} kanalında yayınlanmış sürüm yok
						{:else if updateCheckStatus === "error"}
							Kontrol edilemedi: {updateCheckError}
						{:else}
							{appVersion || "Sürüm bilinmiyor"} · {channelName} kanalı
						{/if}
					</TextBlock>
				</div>

				<svelte:fragment slot="content">
					<div class="items">
						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Açılışta otomatik kontrol et</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Yeni bir sürüm varsa açılıştan birkaç saniye sonra sessizce sorulur.
								</TextBlock>
							</div>
							<div class="expander-control">
								<TextBlock variant="body">{onOff(settings.updateAutoCheck)}</TextBlock>
								<ToggleSwitch bind:checked={settings.updateAutoCheck} />
							</div>
						</div>

						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Yayın kanalı</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Stable kanalda yalnızca tamamlanmış sürümler görünür; alpha ve
									beta sürümleri buraya hiç düşmez. Ön-sürüm kanalları erken
									özellikler içerir ama kararsız olabilir.
								</TextBlock>
							</div>
							<div class="expander-control">
								<ComboBox
									items={updateChannelItems}
									value={settings.updateChannel}
									on:select={handleChannelSelect}
								/>
							</div>
						</div>

						<div class="item">
							<div class="item-text">
								<TextBlock variant="body">Elle kontrol</TextBlock>
								<TextBlock variant="caption" class="text-secondary">
									Beklemeden şimdi bakar; güncelleme varsa aynı diyalog açılır.
								</TextBlock>
							</div>
							<div class="expander-control">
								<Button
									variant="hyperlink"
									on:click={onCheckForUpdates}
									disabled={updateCheckStatus === "checking"}
								>
									Şimdi kontrol et
								</Button>
							</div>
						</div>
					</div>
				</svelte:fragment>
			</Expander>
		</section>

	</div>
</div>

<style>
	/* Görsel karar yok; her ölçü openani.me'nin canlı CSS'inden birebir.
	   Renk ve yüzeyler --fds-* token'larından geliyor. */

	/* .settings { margin: 2.5rem; padding-bottom: 2.5rem } */
	.page {
		box-sizing: border-box;
		height: 100%;
		overflow-y: auto;
		padding: 2.5rem;
		background-color: var(--fds-solid-background-base);
	}

	/* Sitenin kendi `hr` sınıfı — fluent temasında yok, site ekliyor:
	   hr.horizontal { border-block-start: 1px solid var(--fds-divider-stroke-default) } */
	hr.horizontal {
		border: none;
		border-block-start: 1px solid var(--fds-divider-stroke-default);
		width: 100%;
	}

	hr.horizontal.margin {
		margin-block: 1rem 0;
	}

	/* .expanders { margin-top: 1rem; display:flex; flex-direction:column; gap:2rem } */
	.sections {
		margin-top: 1rem;
		display: flex;
		flex-direction: column;
		gap: 2rem;
	}

	/* .expand-section { display:flex; flex-direction:column; gap:.5rem } */
	.expand-section {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	/* ------------------------------------------------------------------
	   Sitenin fluent Expander'a KENDİ eklediği katman.
	   ------------------------------------------------------------------
	   Bu kuralların canlı CSS'te svelte hash'i YOK ya da sitenin kendi
	   sayfa hash'ini (`svelte-ndcra2` = /settings) taşıyor — yani fluent'in
	   varsayılanı değil, site ekliyor.

	   `.page` ile sınırlı: editör panelindeki Expander'lar
	   (`AdvancedSections.svelte`) bu ölçülere göre tasarlanmadı. */

	/* .expander-header { min-height: 50px } */
	.page :global(.expander-header) {
		min-height: 50px;
	}

	/* .expander-icon { block-size: 20px !important; inline-size: 20px !important }
	   Fluent'in varsayılanı 16x16; ikonlarımız 20px çizildiği için o kutuya
	   sıkışıyor ve sitedekinden küçük duruyordu. */
	.page :global(.expander-icon) {
		block-size: 20px;
		inline-size: 20px;
	}

	/* .settings .space-between .expander-header-title {
	       display:flex; justify-content:space-between;
	       margin-right:.7rem; flex-wrap:wrap }
	   NOT: sitenin DİĞER sayfalarında bu kuralda `gap:1rem; align-items:center`
	   de var — ama /settings sürümünde YOK. Ayarlar sürümünü alıyoruz. */
	.page :global(.space-between .expander-header-title) {
		display: flex;
		justify-content: space-between;
		margin-right: 0.7rem;
		flex-wrap: wrap;
	}

	/* .item-header { display:flex; flex-direction:column;
	                  padding-top:.35rem; padding-bottom:.35rem } */
	.item-header {
		display: flex;
		flex-direction: column;
		padding-top: 0.35rem;
		padding-bottom: 0.35rem;
	}

	/* .expander-control { display:flex; align-items:center;
	                       gap:.5rem; min-width:fit-content } */
	.expander-control {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		min-width: fit-content;
	}

	/* --- Katlanan satırların gövdesi -------------------------------------- */
	/*
	   .settings .expander .expander-content { padding: 0 }
	   Fluent gövdeye 16px padding veriyor; site onu sıfırlayıp boşluğu
	   `.item`lere bırakıyor — böylece satır ayıraçları kenardan kenara uzuyor.
	*/
	.page :global(.expander .expander-content) {
		padding: 0;
	}

	/* Sitenin gövde içindeki KENDİ sarmalayıcısı
	   (`.expander-content { display:flex; flex-direction:column }`).
	   Burada `items` adıyla duruyor ki fluent'in aynı adlı kutusuyla
	   karışmasın. */
	.items {
		display: flex;
		flex-direction: column;
	}

	/* .item { display:flex; justify-content:space-between; align-items:center;
	           width:100%; padding:.5rem 3rem; gap:5px }
	   3rem'lik sol boşluk tesadüf değil: başlıktaki 16px padding + 20px ikon +
	   16px ikon boşluğu ile aynı hizaya düşüyor, alt satırlar başlığın metin
	   sütununun altında başlıyor.

	   Sitenin ikinci kuralı da buraya katlandı:
	   .item:nth-child(n) { border-top: 1px solid var(--fds-card-stroke-default);
	                        background-clip: padding-box }
	   `nth-child(n)` HEPSİNİ seçiyor — yani ilk satır da üstten çizgi alıyor;
	   fluent gövdenin üst kenarlığını kaldırdığı için (`border-block-start:none`)
	   başlıkla gövdeyi ayıran çizgi işte bu. */
	.item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		box-sizing: border-box;
		padding: 0.5rem 3rem;
		gap: 5px;
		border-top: 1px solid var(--fds-card-stroke-default);
		background-clip: padding-box;
	}

	.item-text {
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	/* .settings.mobile .item { padding: .5rem 1rem !important }
	   Sitede dar ekranda girinti kayboluyor; pencere daraltıldığında bizde de
	   kaybolsun. */
	@media (max-width: 640px) {
		.item {
			padding: 0.5rem 1rem;
		}
	}

	/* Sitenin combo-box açılır listesinde kaydırma çubuğu görünmüyor. */
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
