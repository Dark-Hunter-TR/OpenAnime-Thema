<script lang="ts">
	/**
	 * Ayarlar sayfasındaki "Hesap" bölümü.
	 *
	 * Düzen, openani.me'nin KENDİ profil sayfasını taklit ediyor: banner
	 * zemin, üstüne binen büyük avatar (durum emojisi ve varsa avatar
	 * dekorasyonuyla), kullanıcı adı + rozetler, `about_me` ve tıklanabilir
	 * "N Takipçi • M Takip Edilen" satırı.
	 *
	 * Rozet koşulları, varlık adları ve takipçi uç noktaları sitenin herkese
	 * açık istemci paketinden birebir çıkarıldı — her biri aşağıda kaynağıyla
	 * not edildi.
	 */
	import { onDestroy, onMount } from "svelte";
	import { openUrl } from "@tauri-apps/plugin-opener";
	import { getCurrentWindow } from "@tauri-apps/api/window";
	import {
		Button,
		ContentDialog,
		IconButton,
		PersonPicture,
		ProgressRing,
		TextBlock
	} from "fluent-svelte-extra";
	import Tooltip from "$lib/Tooltip.svelte";

	import Icon from "$lib/Icon.svelte";
	import StatusBar from "$lib/StatusBar.svelte";
	import {
		accountLogout,
		fetchAccountFollows,
		fetchAccountInfo,
		type AccountInfo,
		type FollowUser
	} from "$lib/theme";

	/** Uygulama içi giriş diyaloğunu açar (`+page.svelte` -> `openLoginDialog`). */
	export let onLogin: () => void;
	/** Çıkış tamamlandığında çağrılır — çağıran taraf oturum durumunu
	 * tazeleyip kartı giriş ekranına döndürüyor. */
	export let onLoggedOut: () => void;

	let account: AccountInfo | null = null;
	let loading = false;
	let error = "";

	async function load() {
		loading = true;
		error = "";
		try {
			account = await fetchAccountInfo();
		} catch (e) {
			error = String(e);
			account = null;
		} finally {
			loading = false;
		}
	}

	onMount(load);

	/**
	 * Uygulama uzun süre arka planda kaldıktan sonra öne gelince hesap
	 * bilgisini SESSİZCE yeniden dener.
	 *
	 * Neden gerekli: Chromium'un arka plan zamanlayıcı kısıtlaması, önizleme
	 * sayfasının kendi geçit yenileme döngülerini (35sn yeniden imzalama,
	 * 5dk'lık yeni oturum) arka plandayken geciktiriyor/durduruyor; öne
	 * geldiğimizde `/user` isteği bayat bir oturumla 401/400 dönebiliyor
	 * (bkz. `lib.rs` -> `bridge_get`, `"reloading"` aşaması — köprü kendi
	 * kendini bir sayfa yenilemesiyle toparlıyor, bu birkaç saniye sürüyor).
	 * Kullanıcının "Tekrar dene"ye elle basmasını beklemek yerine, pencere
	 * odağı geri geldiğinde HÂLÂ hata gösteriyorsak otomatik olarak bir kez
	 * daha deniyoruz. Zaten başarılıysa dokunmuyoruz — her odak değişiminde
	 * gereksiz bir istek atmayalım.
	 */
	let unlistenFocus: (() => void) | null = null;
	/**
	 * Bileşen hâlâ ekranda mı?
	 *
	 * `onFocusChanged` bir `Promise` döndürüyor ve kaydı ancak çözüldüğünde
	 * elimize veriyor. Kullanıcı o çözülmeden Ayarlar'dan çıkarsa `onDestroy`
	 * çalıştığı anda `unlistenFocus` hâlâ `null` olur, atama SONRADAN yapılır
	 * ve dinleyici kalıcı olarak sızardı — her Ayarlar ziyareti bir tane daha
	 * ekleyerek. Sızan her dinleyici, pencere her odaklandığında ayrı bir
	 * `fetch_account_info` (30 sn'lik köprü çağrısı) başlatıyordu.
	 *
	 * Bu bayrak yarışın iki yakasını da kapatıyor: kayıt geç gelirse anında
	 * geri alınıyor, erken gelirse `onDestroy` onu normal yoldan kaldırıyor.
	 */
	let alive = true;

	onMount(async () => {
		const unlisten = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
			if (focused && error) load();
		});
		if (!alive) {
			unlisten();
			return;
		}
		unlistenFocus = unlisten;
	});

	onDestroy(() => {
		alive = false;
		unlistenFocus?.();
		unlistenFocus = null;
	});

	// --- Rozetler ------------------------------------------------------------
	//
	// Bundle'ın TAMAMINDA yalnızca yedi rozet var; hepsi burada. Rozet
	// bileşeni `src: "/assets/badges/" + name` kuruyor. Dosyaları uzaktan
	// çekmek yerine `static/badges/` altına kopyaladık: uygulama çevrimdışıyken
	// ya da site erişilemezken de kart bozulmasın diye.
	const BADGE_BASE = "/badges/";

	// Sitede "Geliştirici" rozeti iki farklı yerde iki farklı şekilde
	// belirleniyor: profil sayfası sabit bir kimlik listesine bakıyor
	// (`devIds.includes(user.id)`), yorum/kullanıcı kartı ise düz bir
	// `user.developer` bayrağına. İkisini de karşılıyoruz.
	const DEVELOPER_IDS = new Set([
		"7139225897831239681",
		"7012257742945521665",
		"7065578790478417921",
		"7058734119315836929",
		"7081169516327079937",
		"7070579223018082305",
		"7240637539315355664",
		"7275154947379826907",
		"7084901915410042881"
	]);

	interface Badge {
		file: string;
		label: string;
	}

	function str(value: unknown): string {
		return typeof value === "string" ? value : "";
	}

	function bool(value: unknown): boolean {
		return value === true;
	}

	/**
	 * Rozetleri sitedeki SIRAYLA ve aynı koşullarla üretir.
	 *
	 * Profil sayfasındaki koşullar (`.../profile` parçası):
	 *   devIds.includes(user.id)  -> developer.svg        "Geliştirici"
	 *   user.admin                -> shield.svg           "Yönetici"
	 *   user.fansubStaff          -> pencil.svg           "Fansub Yöneticisi"
	 *   user.earlySupporter       -> early-supporter.svg  "Erken Destekçi"
	 *   user.premium.tier == 1    -> lite-premium.svg     "OpenAnime+ Lite Üyesi"
	 *   user.premium.tier == 2    -> standard-premium.svg "OpenAnime+ Standard Üyesi"
	 *   user.premium.tier == 3    -> pro-premium.svg      "OpenAnime+ Pro Üyesi"
	 *
	 * Yorum/kullanıcı kartı parçası aynı yedi rozeti kullanıyor ama düzleştirilmiş
	 * alanlarla: `user.developer` ve `user.premiumTier`. Hangi şekil gelirse
	 * gelsin doğru rozet çıksın diye ikisi de okunuyor.
	 *
	 * Destekçilik iki ayrı eksen — karıştırılmamalı:
	 *   • "Erken Destekçi" tek bir rozet, seviyesi yok (`earlySupporter` bayrağı).
	 *   • OpenAnime+ üyeliğinin ÜÇ kademesi var. Kademe adları sitenin kendi
	 *     plan tablosundan: tier1 = Lite, tier2 = Standard, tier3 = Pro.
	 * Sitenin profil sayfası rozeti yalnızca `tier`e bakarak basıyor
	 * (`premium.active` kontrol edilmiyor), burada da aynısı yapılıyor.
	 */
	function badgesOf(user: AccountInfo): Badge[] {
		const list: Badge[] = [];

		if (bool(user.developer) || DEVELOPER_IDS.has(str(user.id))) {
			list.push({ file: "developer.svg", label: "Geliştirici" });
		}
		if (bool(user.admin)) list.push({ file: "shield.svg", label: "Yönetici" });
		if (bool(user.fansubStaff)) list.push({ file: "pencil.svg", label: "Fansub Yöneticisi" });
		if (bool(user.earlySupporter)) {
			list.push({ file: "early-supporter.svg", label: "Erken Destekçi" });
		}

		const nested = (user.premium as { tier?: unknown } | null | undefined)?.tier;
		const tier = typeof nested === "number" ? nested : user.premiumTier;
		if (tier === 1) list.push({ file: "lite-premium.svg", label: "OpenAnime+ Lite Üyesi" });
		if (tier === 2) list.push({ file: "standard-premium.svg", label: "OpenAnime+ Standard Üyesi" });
		if (tier === 3) list.push({ file: "pro-premium.svg", label: "OpenAnime+ Pro Üyesi" });

		return list;
	}

	$: badges = account ? badgesOf(account) : [];

	$: avatar = account ? str(account.avatar) : "";
	$: banner = account ? str(account.banner) : "";
	$: username = account ? str(account.username) : "";
	$: aboutMe = account ? str(account.about_me ?? account.aboutMe) : "";
	$: userId = account ? str(account.id) : "";

	/**
	 * "Profili openani.me'de aç" düğmesi.
	 *
	 * Önceden bu düğme editörü açıp gömülü önizlemeyi openani.me'nin
	 * `/settings` sayfasına yönlendiriyordu. Artık gömülü önizlemeye hiç
	 * dokunmuyor: sitenin gerçek profil adresini (`/profile/<id>`) sistemin
	 * varsayılan tarayıcısında açıyor — kullanıcı istediği profil aracını
	 * (uzantılar, oturum, imleçler…) kullanabilsin diye.
	 */
	function openProfileInBrowser() {
		if (!userId) return;
		void openUrl(`https://openani.me/profile/${userId}`);
	}

	let loggingOut = false;
	let logoutError = "";

	/**
	 * Oturumu kapatır.
	 *
	 * Onay istemiyoruz: sitenin kendi çıkışı da sormuyor ve işlem geri
	 * alınabilir (tekrar giriş yapılır). Yanlışlıkla basmaya karşı düğme
	 * vurgusuz ve diğer eylemden ayrı duruyor.
	 */
	async function logout() {
		if (loggingOut) return;
		loggingOut = true;
		logoutError = "";
		try {
			await accountLogout();
			// Kart artık giriş yapmamış duruma dönecek; yerel hesabı da
			// düşürüyoruz ki bir sonraki girişte eski kullanıcı bir an için
			// görünmesin.
			account = null;
			onLoggedOut();
		} catch (e) {
			logoutError = typeof e === "string" ? e : String(e);
		} finally {
			loggingOut = false;
		}
	}

	// Durum: profildeki avatarın sağ altındaki küçük yuvarlak. Sitede
	// `statusProps: {text: user.status.text, emoji: user.status.emoji}`.
	$: status = (account?.status ?? null) as { emoji?: unknown; text?: unknown } | null;
	$: statusEmoji = str(status?.emoji);
	$: statusText = str(status?.text);

	// Dekorasyon kimliği 0 ise dekorasyon yok — sitenin dosya deposunda
	// `avatar-decorations/0.png` diye bir varlık bulunmuyor (404), `1.png` var.
	const DECORATION_BASE = "https://static.openani.me/avatar-decorations/";
	$: decoration = typeof account?.avatarDecoration === "number" ? account.avatarDecoration : 0;

	// Kendi `/user` yanıtımız sayaç değil dizi döndürüyor (`followersArray` /
	// `followingArray`); profil sayfası ise `followers`/`following` sayılarını
	// kullanıyor. İkisini de karşılıyoruz.
	function countOf(arrayValue: unknown, numberValue: unknown): number {
		if (Array.isArray(arrayValue)) return arrayValue.length;
		return typeof numberValue === "number" ? numberValue : 0;
	}

	$: followerCount = account ? countOf(account.followersArray, account.followers) : 0;
	$: followingCount = account ? countOf(account.followingArray, account.following) : 0;

	// --- Takipçi / takip edilen diyaloğu -------------------------------------

	let dialogKind: "followers" | "following" | null = null;
	let dialogUsers: FollowUser[] = [];
	let dialogLoading = false;
	let dialogError = "";

	$: dialogTitle = dialogKind === "following" ? "Takip Edilenler" : "Takipçiler";

	async function openFollows(kind: "followers" | "following") {
		if (!userId) return;
		dialogKind = kind;
		dialogUsers = [];
		dialogError = "";
		dialogLoading = true;
		try {
			dialogUsers = await fetchAccountFollows(userId, kind);
		} catch (e) {
			dialogError = String(e);
		} finally {
			dialogLoading = false;
		}
	}

	function closeFollows() {
		dialogKind = null;
	}
</script>

<div class="account">
	{#if loading && !account}
		<div class="state">
			<ProgressRing size={20} />
			<TextBlock variant="caption">Hesap bilgileri getiriliyor…</TextBlock>
		</div>
	{:else if error}
		<StatusBar severity="critical" title="Hesap bilgileri alınamadı" message={error} closable={false} />
		<!--
			İki farklı kurtarma yolu, çünkü iki farklı hata sınıfı var:

			• "Tekrar dene" — önizlemedeki openani.me henüz hazır değilse işe
			  yarar. Hesap isteği o sayfanın İÇİNDEN atılıyor (Vanguard geçidi
			  yüzünden başka türlü mümkün değil); bu kullanıcıya görünmeyen,
			  tamamen teknik bir ayrıntı.

			• "Tekrar giriş yap" — oturumun kendisi düşmüşse (401) gerekir;
			  bu durumda tekrar denemek aynı hatayı verir.
		-->
		<div class="error-actions">
			<Button variant="accent" on:click={onLogin}>
				<Icon name="person" size={14} /><span class="gap">Tekrar giriş yap</span>
			</Button>
			<Button on:click={load}>
				<Icon name="refresh" size={14} /><span class="gap">Tekrar dene</span>
			</Button>
		</div>
	{:else if account}
		<!--
			`fds-theme-dark`: sitenin kendi profil kahramanında `#main`, başlık
			(`h2`) ve about-me metninde tek tek taşınan sınıf. Bu bölge her
			zaman bir fotoğrafın üstünde durduğu için site rengi UYGULAMANIN
			genel temasından bağımsız hep koyu tutuyor — açık temada bile metin
			beyaz kalıyor. `fds-theme-dark` yalnızca `--fds-*` token'larını bu
			alt ağaçta koyu palete çeviren yerel bir kapsam sınıfı (bkz.
			fluent-svelte-extra/theme.css); tek tek her metne eklemek yerine
			tüm `.profile`'a bir kere veriyoruz.
		-->
		<div class="profile fds-theme-dark">
			<div
				class="profile-banner"
				class:profile-banner--empty={!banner}
				style={banner ? `background-image: url(${JSON.stringify(banner)})` : ""}
			></div>

			<div class="profile-actions">
				<Tooltip text="Yenile">
					<IconButton aria-label="Hesap bilgilerini yenile" on:click={load} disabled={loading}>
						<Icon name="refresh" size={16} />
					</IconButton>
				</Tooltip>
			</div>

			<div class="profile-body">
				<div class="avatar-stack">
					{#if avatar}
						<img class="avatar" class:avatar--notched={!!statusEmoji} src={avatar} alt={username} />
					{:else}
						<span class="avatar avatar-fallback" class:avatar--notched={!!statusEmoji}>
							<Icon name="avatar" size={40} />
						</span>
					{/if}
					{#if decoration}
						<img class="decoration" src={`${DECORATION_BASE}${decoration}.png`} alt="" aria-hidden="true" />
					{/if}
					{#if statusEmoji}
						<!--
							Sitedeki davranışın aynısı: normalde yalnızca emoji duruyor,
							üstüne gelince sağa doğru açılıp durum metnini gösteriyor
							(profil DOM'unda `<button class="status">` + `.emoji-wrapper`
							+ `.status-text`). Bu yüzden Tooltip kullanılmıyor.
						-->
						<span class="status-chip" class:status-chip--expandable={!!statusText}>
							<span class="status-emoji">{statusEmoji}</span>
							{#if statusText}<span class="status-text">{statusText}</span>{/if}
						</span>
					{/if}
				</div>

				<div class="profile-text">
					<div class="name-row">
						<!--
							Site kullanıcı adını `<h2 class="type-title-large">` ile basıyor
							(40px/52px) — bizim önceki `variant="subtitle"` (20px) seçimimiz
							boyutu ciddi şekilde küçültüyordu. `titleLarge` = `type-title-large`,
							DOM'daki gerçek sınıfla birebir eşleşiyor.
						-->
						<TextBlock variant="titleLarge" class="text-primary">
							{username || "Bilinmeyen kullanıcı"}
						</TextBlock>
						{#each badges as badge (badge.file)}
							<Tooltip text={badge.label}>
								<img class="badge" src={`${BADGE_BASE}${badge.file}`} alt={badge.label} />
							</Tooltip>
						{/each}
					</div>

					{#if aboutMe}
						<TextBlock variant="body" class="text-secondary">{aboutMe}</TextBlock>
					{/if}

					<!-- Site "N Takipçi" metnini `type-body` (14px) ile basıyor, `caption`
						 (12px) değil — burada da düzeltildi. -->
					<div class="follows">
						<Tooltip text="Takipçileri görmek için tıklayın">
							<button type="button" class="follow-link" on:click={() => openFollows("followers")}>
								<TextBlock variant="body" class="text-tertiary">{followerCount} Takipçi</TextBlock>
							</button>
						</Tooltip>
						<span class="dot">•</span>
						<Tooltip text="Takip edilenleri görmek için tıklayın">
							<button type="button" class="follow-link" on:click={() => openFollows("following")}>
								<TextBlock variant="body" class="text-tertiary">{followingCount} Takip Edilen</TextBlock>
							</button>
						</Tooltip>
					</div>
				</div>
			</div>
		</div>

		{#if logoutError}
			<StatusBar
				severity="critical"
				title="Çıkış yapılamadı"
				message={logoutError}
				closable={false}
			/>
		{/if}

		<div class="account-actions">
			<Button on:click={openProfileInBrowser} disabled={!userId}>
				<Icon name="openExternal" size={16} /><span class="gap">Profili openani.me'de aç</span>
			</Button>
			<Button on:click={logout} disabled={loggingOut}>
				<Icon name="person" size={16} /><span class="gap">
					{loggingOut ? "Çıkış yapılıyor…" : "Çıkış yap"}
				</span>
			</Button>
		</div>
	{/if}
</div>

<ContentDialog
	open={dialogKind !== null}
	title={dialogTitle}
	closeButton
	on:close={closeFollows}
	on:closeByButton={closeFollows}
	on:backdropclick={closeFollows}
>
	{#if dialogLoading}
		<div class="state">
			<ProgressRing size={20} />
			<TextBlock variant="caption">Liste getiriliyor…</TextBlock>
		</div>
	{:else if dialogError}
		<StatusBar severity="critical" title="Liste alınamadı" message={dialogError} closable={false} />
	{:else if dialogUsers.length === 0}
		<TextBlock variant="body" class="text-secondary">
			{dialogKind === "following" ? "Kimseyi takip etmiyorsunuz." : "Kimse sizi takip etmiyor."}
		</TextBlock>
	{:else}
		<div class="follow-list">
			{#each dialogUsers as user (user.id)}
				<div class="follow-row">
					<PersonPicture src={user.avatar} alt={user.username} size={32} />
					<TextBlock variant="body">{user.username}</TextBlock>
				</div>
			{/each}
		</div>
	{/if}
</ContentDialog>

<style>
	.account-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.account {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.state {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.error-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	/* --- Profil kartı ---------------------------------------------------- */
	/*
	   Sitedeki profil düzeni: banner bir şerit DEĞİL, bütün alanın zemini.
	   Görsel üstte tam görünüyor, aşağı indikçe sayfa zeminine karışıyor
	   (sitenin kendi değerleri: `--size: 70vh; --gradient-end: 50%;
	   --opacity: 0.75`). Avatar ve metin bu zeminin üstünde, alta yakın
	   duruyor. Burada aynı fikir kart ölçeğine indirgendi.
	*/

	.profile {
		position: relative;
		display: flex;
		align-items: flex-end;
		min-height: 208px;
		border-radius: var(--fds-overlay-corner-radius);
		border: 1px solid var(--fds-card-stroke-default);
		background-color: var(--fds-card-background-default);
		overflow: hidden;
	}

	.profile-banner {
		position: absolute;
		inset: 0;
		background-size: cover;
		background-position: center;
		opacity: 0.75;
	}

	/* Degradenin işi iki katlı: banner'ı kart zeminine eritmek ve alttaki
	   yazıya okunur bir fon bırakmak. Bu yüzden üst yarı tamamen açık,
	   kararma alt yarıda başlıyor. */
	.profile-banner::after {
		content: "";
		position: absolute;
		inset: 0;
		background: linear-gradient(
			180deg,
			rgba(0, 0, 0, 0) 45%,
			var(--fds-card-background-default) 100%
		);
	}

	.profile-banner--empty {
		background-color: var(--fds-subtle-fill-secondary);
		opacity: 1;
	}

	.profile-actions {
		position: absolute;
		top: 8px;
		right: 8px;
		z-index: 2;
	}

	.profile-body {
		position: relative;
		display: flex;
		align-items: flex-end;
		gap: 14px;
		width: 100%;
		padding: 0 18px 18px;
	}

	.avatar-stack {
		position: relative;
		flex: none;
		width: 88px;
		height: 88px;
		/* Avatarı bannerin fotoğraf kısmına biraz daha taşırıp yalnızca
		   soluk/düz zemine gömülü kalmasını önlüyor.
		   Sitedeki gerçek DOM'da bu değeri veren CSS kuralı elimizde değil
		   (yalnızca sınıf adları görünüyor, kural gövdeleri yok) — o yüzden
		   birebir kopyalanmış bir sayı değil, tipik "hero avatar üst şeride
		   taşar, metin satırı sabit kalır" düzenine göre seçilmiş ölçülü
		   bir değer. `.profile-body`nin `align-items: flex-end` hizasına
		   göre yalnızca avatarı yukarı çekiyor — metin satırı yerinde kalır. */
		margin-top: -16px;
		/* Rozetin çapı — `.status-chip` ve avatardaki çentik (`.avatar--notched`)
		   bunu paylaşıyor. Delik rozetten BÜYÜK tutuluyor, kasıtlı: sitenin
		   gerçek DOM'undaki `--status-size-ratio: 0.28` (rozet çapı) ve
		   `--avatar-size`'a göre ölçülen delik yarıçapı `r: 0.175` (bkz.
		   sitenin `#svg-mask-ksevhq9` maskesi) arasındaki oran 0.175/0.14 = 1.25
		   — yani site de rozetin ETRAFINDA görünür bir pay bırakıyor, arkadaki
		   zemin (fotoğraf + degrade) o payın içinden hafif bir hilal olarak
		   görünüyor. Önceki denemede bunu "kusur" sanıp deliği rozetle aynı
		   boyuta küçültmüştük — o hilal aslında sitenin kendi görünümüydü. */
		--status-d: 28px;
	}

	.avatar {
		width: 88px;
		height: 88px;
		border-radius: 999px;
		object-fit: cover;
		display: block;
		background-color: var(--fds-subtle-fill-secondary);
	}

	/* Sitenin kendi tekniği: avatardan durum rozetinin altında kalan yuvarlak
	   parçayı kesip altını gösteriyorlar (bkz. sitenin `#svg-mask-ksevhq9`:
	   beyaz daire eksi merkezi %85,%85'te, rozetten 1.25× büyük bir daire).
	   Aynısını CSS mask ile yapıyoruz; merkez aynı (%85,%85), yarıçap
	   `--status-d`'den TÜRETİLİYOR (bkz. `.avatar-stack` üzerindeki not) —
	   rozet boyutu değişse bile oran hep 1.25× kalır. */
	.avatar--notched {
		-webkit-mask-image: radial-gradient(
			circle calc(var(--status-d) / 2 * 1.25) at 85% 85%,
			transparent 99%,
			#000 100%
		);
		mask-image: radial-gradient(
			circle calc(var(--status-d) / 2 * 1.25) at 85% 85%,
			transparent 99%,
			#000 100%
		);
	}

	.avatar-fallback {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--fds-text-tertiary);
		box-sizing: border-box;
	}

	/* Dekorasyon avatarın ÜSTÜNE oturan şeffaf bir PNG; sitede de avatardan
	   biraz taşacak şekilde ölçekleniyor. Tıklamayı yutmasın diye
	   pointer-events kapalı. */
	.decoration {
		position: absolute;
		inset: -12%;
		width: 124%;
		height: 124%;
		pointer-events: none;
		user-select: none;
	}

	/*
	   Durum baloncuğu. Sitedeki gibi kapalıyken sadece emoji, üstüne gelince
	   sağa doğru açılıp metni gösteriyor. Sol/üst konumu ve boyutu
	   `.avatar--notched`'teki delikle AYNI hesaptan geliyor (%85,%85 merkez,
	   `--status-d` çapı) — bkz. `.avatar-stack` üzerindeki not. Ayırıcı bir
	   çerçeve/halka YOK; site de kullanmıyor, rozet avatarın kenarında
	   doğrudan keskin bir kesikle duruyor.
	*/
	.status-chip {
		position: absolute;
		left: calc(88px * 0.85 - var(--status-d) / 2);
		top: calc(88px * 0.85 - var(--status-d) / 2);
		z-index: 3;
		display: inline-flex;
		align-items: center;
		height: var(--status-d);
		max-width: var(--status-d);
		padding: 0 2px;
		box-sizing: border-box;
		border-radius: 999px;
		background-color: rgb(228, 233, 232);
		color: #000;
		overflow: hidden;
		white-space: nowrap;
		transition: max-width var(--fds-control-normal-duration, 0.25s) ease;
	}

	.status-emoji {
		flex: none;
		width: 24px;
		text-align: center;
		font-size: 15px;
		line-height: 1;
	}

	.status-text {
		font-size: 12px;
		line-height: 1;
		padding-right: 8px;
		opacity: 0;
		transition: opacity var(--fds-control-normal-duration, 0.25s) ease;
	}

	.status-chip--expandable:hover {
		max-width: 320px;
	}

	.status-chip--expandable:hover .status-text {
		opacity: 1;
	}

	.profile-text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		flex: 1;
		padding-bottom: 4px;
	}

	/* `align-items: baseline` ve `gap: 0.5rem` sitedeki inline style'dan
	   birebir: `style="display: flex; align-items: baseline; gap: 0.5rem;"`. */
	.name-row {
		display: flex;
		align-items: baseline;
		gap: 0.5rem;
		flex-wrap: wrap;
	}

	/* Rozetler renkli degrade SVG'ler — `currentColor` kullanmıyorlar, bu
	   yüzden boyanmıyor, olduğu gibi basılıyorlar. */
	/* Site rozet boyutu için `--size: 24px` kullanıyor (DOM'daki inline
	   style), 20px değil. */
	.badge {
		width: 24px;
		height: 24px;
		display: block;
	}

	.follows {
		display: flex;
		align-items: center;
		gap: 6px;
		margin-top: 2px;
	}

	.dot {
		color: var(--fds-text-tertiary);
		font-size: 12px;
	}

	.follow-link {
		appearance: none;
		background: none;
		border: none;
		padding: 2px 4px;
		margin: 0 -4px;
		border-radius: var(--fds-control-corner-radius);
		cursor: pointer;
		color: inherit;
		font: inherit;
	}

	.follow-link:hover {
		background-color: var(--fds-subtle-fill-secondary);
	}

	.follow-link:active {
		background-color: var(--fds-subtle-fill-tertiary);
	}

	/* --- Takipçi listesi -------------------------------------------------- */

	.follow-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 25rem;
		overflow-y: auto;
	}

	.follow-row {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 4px 2px;
	}

	.gap {
		margin-left: 6px;
	}
</style>
