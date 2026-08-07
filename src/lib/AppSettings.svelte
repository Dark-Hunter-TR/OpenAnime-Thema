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
	import {
		Button,
		ComboBox,
		Expander,
		InfoBar,
		SegmentedControlButton,
		TextBlock,
		ToggleSwitch,
		Tooltip
	} from "fluent-svelte-extra";

	import Icon from "$lib/Icon.svelte";
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
	export let onOpenAccount: () => void;

	const routeItems = ROUTES.map((route) => ({
		name: route.auth ? `${route.name} (giriş gerekir)` : route.name,
		value: route.path
	}));

	const viewportItems = VIEWPORTS.map((vp) => ({ name: vp.name, value: vp.id }));
</script>

<div class="page">
	<header class="head">
		<TextBlock variant="title">Ayarlar</TextBlock>
		<TextBlock variant="body">Uygulamanın kendi tercihleri. Düzenlediğiniz temayı etkilemez.</TextBlock>
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

			<Expander expandable={false}>
				<Icon slot="icon" name="typography" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">Dil</TextBlock>
						<TextBlock variant="caption">
							Uygulama şu an yalnızca Türkçe. Başka bir dil seçeneği eklenmediği için bu
							kontrol devre dışı — çalışmayan bir seçenek sunmak yerine olduğu gibi
							gösteriyoruz.
						</TextBlock>
					</span>
					<span class="item-action">
						<Tooltip text="Şimdilik tek dil mevcut">
							<ComboBox items={[{ name: "Türkçe", value: "tr" }]} value="tr" disabled />
						</Tooltip>
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

			<Expander expandable={false}>
				<Icon slot="icon" name="avatar" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">OpenAnime hesabı</TextBlock>
						<TextBlock variant="caption">
							{loggedIn
								? "Önizlemede oturum açık. Giriş gerektiren sayfalar önizlenebilir."
								: "Önizlemede oturum açık değil."}
						</TextBlock>
						<TextBlock variant="caption">
							Uygulama sizden OpenAnime parolası istemez ve hesabınıza erişmez.
						</TextBlock>
					</span>
					<span class="item-action">
						{#if loggedIn}
							<Button on:click={onOpenAccount}>
								<Icon name="person" size={16} /><span class="gap">Hesabı önizlemede aç</span>
							</Button>
						{:else}
							<Button on:click={onPreviewLogin}>
								<Icon name="person" size={16} /><span class="gap">Önizlemede giriş yap</span>
							</Button>
						{/if}
					</span>
				</div>
			</Expander>

			<!--
				Bu kutu bir özür değil, bir kayıt: özelliğin neden yapılmadığını
				kullanıcıya açıkça söylüyoruz ki "eksik kalmış" sanılmasın.
			-->
			<InfoBar severity="information" title="Neden uygulama içi giriş yok?" closable={false}>
				<TextBlock variant="caption">
					openani.me üçüncü taraf uygulamalar için herkese açık bir giriş (OAuth) mekanizması
					sunmuyor: sitenin giriş uç noktası e-posta ve parolayı doğrudan istiyor, uygulama
					kaydı ya da yetkilendirme ekranı yok. Bu yüzden buraya parola alan bir form
					koymuyoruz. Bunun yerine önizleme penceresi kendi oturumunu tutuyor ve siteye
					tarayıcıdaki gibi giriş yapabiliyorsunuz — kimlik bilgileriniz uygulamaya hiç
					uğramıyor. Tema düzenleme ve önizleme zaten girişten bağımsız çalışır.
				</TextBlock>
			</InfoBar>
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

			<Expander>
				<Icon slot="icon" name="navAbout" size={20} />
				<div class="item">
					<span class="item-header">
						<TextBlock variant="body">OpenAnime Tema Editörü{appVersion ? ` ${appVersion}` : ""}</TextBlock>
						<TextBlock variant="caption">Temaların siteye nasıl uygulandığı hakkında.</TextBlock>
					</span>
				</div>

				<svelte:fragment slot="content">
					<div class="about">
						<TextBlock variant="caption">
							Editör, sitenin kendi resmî tema giriş noktasını kullanır: openani.me açılışta
							<code>localStorage.theme_content</code> içindeki CSS'i okuyup sayfaya
							<code>&lt;style themeStyle&gt;</code> olarak ekler. Ürettiğimiz dosya tam olarak
							budur — yani önizlemede gördüğünüz ile dışa aktardığınız aynı metindir.
						</TextBlock>
						<TextBlock variant="caption">
							Hazır temanızı sitede kullanmak için: openani.me → Ayarlar → Görünüm → Özel Tema →
							Yükle ile dışa aktardığınız <code>.css</code> dosyasını seçin.
						</TextBlock>
						<TextBlock variant="caption">
							Renkler <code>--fds-*</code> token'ları üzerinden değiştirilir; bunlar sitenin
							kendi tasarım sistemidir, dolayısıyla tema site güncellendikçe birlikte evrilir.
						</TextBlock>
					</div>
				</svelte:fragment>
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

	.item-action {
		flex: 0 0 auto;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.about {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
	}

	.gap {
		margin-left: 6px;
	}
</style>
