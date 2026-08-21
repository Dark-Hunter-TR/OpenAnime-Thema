<script lang="ts">
	/**
	 * openani.me giriş diyaloğu — sitenin kendi auth modalının birebir kopyası.
	 *
	 * ## Neden `DialogShell` KULLANMIYOR
	 *
	 * `AboutDialog`/`UpdateDialog` ortak kabuğu banner + logo + Setsuki maskotlu
	 * tek sütunlu bir kart. Sitenin giriş modalı ise iki sütunlu: solda tam
	 * yükseklikte bir görsel (`/login-banner.jpg`), sağda form. Yapı temelden
	 * farklı olduğu için kabuğu zorlamak yerine burada kendi iskeletini kuruyor.
	 *
	 * Yerleşim ve ölçüler sitenin canlı CSS'inden alındı
	 * (`#page` / `#background` / `#auth` / `#box` kuralları):
	 *
	 *   .auth-dialog        -> 75% x 75%
	 *   #background         -> width: clamp(25rem, 40%, 30rem)
	 *   #box                -> width: clamp(25rem, 40%, 30rem), min-height: 75%
	 *
	 * Sitedeki `@media (width <= 1096px) { #background { display: none } }`
	 * kuralı da taşındı — dar pencerede görsel gizlenip form tüm genişliği alır.
	 *
	 * ## İki adımlı akış
	 *
	 * Site önce yalnızca e-posta soruyor ("Devam Et"), sonra parola ekranına
	 * geçiyor. E-posta adımı SUNUCUYA GİTMİYOR — sitenin kendi kodunda da
	 * tamamen istemci tarafı bir geçiş; doğrulama `POST /user/auth` ile ikinci
	 * adımda, e-posta ve parola birlikte gönderilerek yapılıyor.
	 *
	 * ## QR ile giriş
	 *
	 * Üçüncü bir adım: `GET /dag` SSE akışı bir QR görseli üretiyor, kullanıcı
	 * telefonundaki OpenAnime uygulamasıyla okutunca akış `success` olayıyla
	 * token'ı gönderiyor. Kod kısa aralıklarla yenilendiği için `qr` olayı
	 * birden çok kez geliyor. Akış da Vanguard'ın arkasında, yani o da aynı
	 * köprüden geçiyor (`$lib/theme.ts` -> `accountQrNext`).
	 *
	 * ## Kayıt olma BİLEREK yok
	 *
	 * Bu bir tema editörü; hesap açma yeri değil. Uygulama yalnızca var olan
	 * bir hesapla giriş yapılmasını sağlıyor.
	 *
	 * ## İstek nereden çıkıyor
	 *
	 * Form burada, ama `POST /user/auth` önizlemedeki openani.me sayfasının
	 * İÇİNDEN atılıyor: Vanguard geçidi `Gateway-Token` başlığı olmayan her
	 * isteği 401'liyor ve o başlık yalnızca sayfanın `/osc.wasm` ile imzalanan
	 * yamalı `fetch`'i tarafından eklenebiliyor. Zincir: bu dosya ->
	 * `$lib/theme.ts` -> `lib.rs` -> `preview.rs` -> `preview_init.js`.
	 *
	 * ## Parola burada kalmıyor
	 *
	 * `password` yalnızca bu bileşenin ömrü boyunca bir yerel değişken; yanıt
	 * gelir gelmez siliniyor. Dönen erişim token'ı uygulamaya hiç uğramıyor —
	 * sayfa onu doğrudan çereze yazıyor.
	 */
	import { Button, ProgressRing, TextBlock, TextBox } from "fluent-svelte-extra";
	import { circOut } from "svelte/easing";
	import { fade, scale } from "svelte/transition";

	import { accountLogin, accountQrNext, loginErrorText, stopAccountQr } from "$lib/theme";

	export let open = false;
	export let onClose: () => void;
	/** Giriş başarılı olduğunda çağrılır — çağıran taraf oturum durumunu
	 * tazeleyip hesap kartını yeniliyor. */
	export let onSuccess: () => void;
	/** Parola sıfırlama ve e-posta doğrulama akışlarının uygulamada karşılığı
	 * YOK (ikisi de Cloudflare Turnstile captcha'sı istiyor); bunlar kullanıcıyı
	 * önizlemedeki gerçek siteye yolluyor. */
	export let onOpenSite: () => void;

	/** Sitedeki `login-email` / `login-password` / `login-qr` sayfalarının
	 * karşılığı. */
	let step: "email" | "password" | "qr" = "email";

	let email = "";
	let password = "";
	let busy = false;
	let errorMessage = "";
	/** Giriş başarılı ama hesabın e-postası doğrulanmamış. */
	let needsVerify = false;

	// İstek sürerken kapatmayı kilitliyoruz: köprü yanıtı geldiğinde çerez
	// yazılıyor ve diyalog o anda yok olmuş olursa kullanıcı giriş yaptığını
	// hiç öğrenemezdi.
	$: closable = !busy;

	// Sitedeki `disabled` mantığının aynısı: e-posta adımında adres, parola
	// adımında parola dolu olmadan düğme pasif.
	$: canContinue = email.trim().length > 0;
	$: canSubmit = !busy && password.length > 0;

	function reset() {
		// Diyalog kapanırken QR akışı açık kalırsa sayfada boşuna bir SSE
		// bağlantısı sürerdi.
		stopQr();
		step = "email";
		email = "";
		password = "";
		errorMessage = "";
		needsVerify = false;
		qrImage = "";
		qrCompleting = false;
	}

	function close() {
		reset();
		onClose();
	}

	function requestClose() {
		if (closable) close();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape" && open) requestClose();
	}

	/** E-posta adımı: sunucuya gitmez, yalnızca parola adımına geçer. */
	function continueToPassword() {
		if (!canContinue) return;
		errorMessage = "";
		step = "password";
	}

	// --- QR ile giriş -----------------------------------------------------
	//
	// Akış Rust üzerinden önizlemedeki sayfada duruyor (`GET /dag`, SSE).
	// Buradaki döngü yalnızca olayları tüketiyor: `idle` gelirse tekrar sorar,
	// `qr` gelirse görseli tazeler, `success`/`error` ile biter.

	/** Gösterilen QR görselinin kaynağı; boşken "hazırlanıyor" durumundayız. */
	let qrImage = "";
	/** Kod okutuldu, oturum kuruluyor. */
	let qrCompleting = false;
	/**
	 * O an çalışan döngünün kimliği.
	 *
	 * Döngü `await` sınırlarında askıya giriyor; kullanıcı bu sırada adımı
	 * değiştirebilir ya da diyaloğu kapatabilir. Her tur bu kimliği kontrol
	 * ederek eski bir döngünün yeni durumun üzerine yazmasını engelliyoruz.
	 */
	let qrRun = 0;

	async function startQr() {
		errorMessage = "";
		qrImage = "";
		qrCompleting = false;
		step = "qr";

		const run = ++qrRun;
		while (run === qrRun) {
			let event;
			try {
				event = await accountQrNext();
			} catch (e) {
				if (run !== qrRun) return;
				errorMessage = loginErrorText(typeof e === "string" ? e : String(e));
				return;
			}
			if (run !== qrRun) return;

			if (event.kind === "idle") continue;
			if (event.kind === "qr") {
				qrImage = event.image ?? "";
				continue;
			}
			if (event.kind === "error") {
				errorMessage = loginErrorText(event.message ?? "QR ile giriş başarısız.");
				return;
			}
			if (event.kind === "success") {
				qrCompleting = true;
				stopQr();
				if (event.verified === false) {
					needsVerify = true;
					return;
				}
				reset();
				onSuccess();
				return;
			}
		}
	}

	/** Döngüyü durdurur ve sayfadaki akışı kapatır. */
	function stopQr() {
		qrRun++;
		void stopAccountQr();
	}

	function backToEmail() {
		stopQr();
		qrImage = "";
		qrCompleting = false;
		errorMessage = "";
		step = "email";
	}

	async function submit() {
		if (!canSubmit) return;
		busy = true;
		errorMessage = "";
		needsVerify = false;

		try {
			const outcome = await accountLogin(email, password);

			// Parolayı yanıt gelir gelmez düşürüyoruz; sonraki adımların hiçbiri
			// ona ihtiyaç duymuyor.
			password = "";

			if (!outcome.verified) {
				// Oturum AÇILDI (çerez yazıldı), yalnızca e-posta doğrulanmamış.
				// Diyaloğu kapatmıyoruz ki kullanıcı ne yapması gerektiğini görsün.
				needsVerify = true;
				return;
			}

			reset();
			onSuccess();
		} catch (e) {
			// Rust hem kendi hatalarını (önizleme yok, geçit kurulmadı) hem de
			// sunucunun ham İngilizce mesajını aynı kanaldan döndürüyor;
			// `loginErrorText` yalnızca tanıdıklarını çeviriyor.
			errorMessage = loginErrorText(typeof e === "string" ? e : String(e));
		} finally {
			busy = false;
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<!-- Geçiş süreleri `DialogShell` ile AYNI (83ms fade + 167ms scale/circOut)
	     ki iki diyalog arasında ritim değişmesin. -->
	<div
		class="smoke"
		transition:fade={{ duration: 83 }}
		on:click|self={requestClose}
		role="presentation"
	>
		<div
			class="auth-dialog"
			transition:scale={{ duration: 167, start: 1.05, easing: circOut }}
			role="dialog"
			aria-modal="true"
			aria-label="Giriş Yap"
		>
			<div id="page">
				<!-- Sitedeki `#background`: soldaki tam yükseklikte görsel.
				     Kaynak `static/login-banner.jpg` — sitenin kendi dosyasının
				     kopyası. Uzaktan (`https://openani.me/login-banner.jpg`)
				     çekmek de mümkündü ama o zaman çevrimdışıyken boş kalırdı. -->
				<div id="background">
					<img alt="" src="/login-banner.jpg" />
				</div>

				<div id="auth">
					<div id="box">
						{#if needsVerify}
							<h3 class="text-block type-title">Son bir adım</h3>
							<span class="text-block type-body text-tertiary">
								Giriş yapıldı, ancak e-posta adresin doğrulanmamış.
							</span>
							<div class="stack">
								<hr class="horizontal" />
								<div class="notice notice-warn">
									<TextBlock variant="caption">
										Doğrulama kodunu girmek için siteyi aç; doğruladıktan sonra hesabın
										burada da tam olarak çalışacak.
									</TextBlock>
								</div>
								<Button type="button" variant="accent" on:click={onOpenSite}>Siteyi Aç</Button>
								<Button type="button" variant="hyperlink" on:click={close}>Kapat</Button>
							</div>
						{:else}
							<h3 class="text-block type-title">
								{step === "qr" ? "QR kod ile giriş yap" : "Giriş Yap"}
							</h3>
							<span class="text-block type-body text-tertiary">
								{step === "qr"
									? "Telefonundaki OpenAnime uygulamasıyla QR kodu okutarak giriş yap."
									: "Devam etmek için OpenAnime'ye giriş yap."}
							</span>

							{#if step === "email"}
								<!-- Gerçek `form`: Enter ile gönderme buradan geliyor.
								     `on:keydown` dinlemek de mümkündü ama
								     fluent-svelte-extra forward ettiği DOM olaylarını
								     `CustomEvent` olarak tiplediği için tip hatası
								     veriyor — sitenin modalı da form kullanıyor. -->
								<form on:submit|preventDefault={continueToPassword}>
									<hr class="horizontal" />
									<h5 class="text-block type-body-strong">E-posta Adresi</h5>
									<TextBox bind:value={email} type="email" placeholder="mail@example.com" />

									{#if errorMessage}
										<div class="notice notice-error">
											<TextBlock variant="caption">{errorMessage}</TextBlock>
										</div>
									{/if}

									<Button type="submit" variant="accent" disabled={!canContinue}>Devam Et</Button>
									<Button type="button" variant="hyperlink" on:click={startQr}>
										QR kod ile giriş yap
									</Button>
								</form>
							{:else if step === "password"}
								<form on:submit|preventDefault={submit}>
									<hr class="horizontal" />
									<h5 class="text-block type-body-strong">Şifre</h5>
									<TextBox
										bind:value={password}
										type="password"
										placeholder="********"
										disabled={busy}
									/>

									{#if errorMessage}
										<div class="notice notice-error">
											<TextBlock variant="caption">{errorMessage}</TextBlock>
										</div>
									{/if}

									<Button type="submit" variant="accent" disabled={!canSubmit}>
										{busy ? "Giriş yapılıyor…" : "Giriş Yap"}
									</Button>
									<!-- Parola sıfırlama da captcha'lı (`/user/resetpass/send`). -->
									<Button type="button" variant="hyperlink" on:click={onOpenSite} disabled={busy}>
										Şifreni mi unuttun?
									</Button>
									<Button
										type="button"
										variant="hyperlink"
										on:click={() => (step = "email")}
										disabled={busy}
									>
										Geri
									</Button>
								</form>
							{:else}
								<!-- QR adımı. Görsel sunucudan HAZIR geliyor
								     ({"type":"qr","data":"<kaynak>"}), yani bir QR
								     kodlayıcı kütüphanesine ihtiyaç yok. -->
								<div class="qr-step">
									<hr class="horizontal" />
									<div class="qr-frame">
										{#if qrImage}
											<img src={qrImage} alt="QR kod ile giriş" draggable="false" />
										{:else}
											<ProgressRing size={48} />
										{/if}
									</div>

									<TextBlock variant="caption" class="text-tertiary">
										{#if qrCompleting}
											Giriş tamamlanıyor…
										{:else if !qrImage}
											QR kod hazırlanıyor…
										{:else}
											QR kod kısa aralıklarla yenilenir.
										{/if}
									</TextBlock>

									{#if errorMessage}
										<div class="notice notice-error">
											<TextBlock variant="caption">{errorMessage}</TextBlock>
										</div>
										<Button type="button" variant="accent" on:click={startQr}>
											Yeniden dene
										</Button>
									{/if}

									<Button type="button" variant="hyperlink" on:click={backToEmail}>
										E-posta ile giriş yap
									</Button>
								</div>
							{/if}
						{/if}
					</div>
				</div>
			</div>

			{#if closable}
				<button id="close-button" aria-label="Diyaloğu kapat" type="button" on:click={close}>
					<svg
						aria-hidden="true"
						xmlns="http://www.w3.org/2000/svg"
						width="12"
						height="12"
						viewBox="0 0 1024 1024"
					>
						<path
							fill="currentColor"
							d="M512,584.5L87.5,1009C77.5,1019 65.5,1024 51.5,1024C36.8333,1024 24.5833,1019.08 14.75,1009.25C4.91667,999.417 0,987.167 0,972.5C0,958.5 5,946.5 15,936.5L439.5,512L15,87.5C5,77.5 0,65.3334 0,51C0,44 1.33333,37.3334 4,31C6.66667,24.6667 10.3333,19.25 15,14.75C19.6667,10.25 25.1667,6.66669 31.5,4C37.8333,1.33337 44.5,0 51.5,0C65.5,0 77.5,5 87.5,15L512,439.5L936.5,15C946.5,5 958.667,0 973,0C980,0 986.583,1.33337 992.75,4C998.917,6.66669 1004.33,10.3334 1009,15C1013.67,19.6667 1017.33,25.0834 1020,31.25C1022.67,37.4167 1024,44 1024,51C1024,65.3334 1019,77.5 1009,87.5L584.5,512L1009,936.5C1019,946.5 1024,958.5 1024,972.5C1024,979.5 1022.67,986.167 1020,992.5C1017.33,998.833 1013.75,1004.33 1009.25,1009C1004.75,1013.67 999.333,1017.33 993,1020C986.667,1022.67 980,1024 973,1024C958.667,1024 946.5,1019 936.5,1009Z"
						></path>
					</svg>
				</button>
			{/if}
		</div>
	</div>
{/if}

<style>
	.smoke {
		position: fixed;
		inset: 0;
		z-index: 9999;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--fds-smoke-background-default, rgba(0, 0, 0, 0.45));
	}

	/* Sitedeki `.auth-dialog` ölçüsü: 75% x 75%. `max-*` sınırları bizde ek —
	   editör penceresi büyük ekranlarda tam ekran olabiliyor ve sınırsız
	   bırakılsa diyalog gereksiz devasa görünürdü. */
	.auth-dialog {
		position: relative;
		inline-size: 75%;
		block-size: 75%;
		max-inline-size: 1100px;
		max-block-size: 720px;
		background-color: var(--fds-solid-background-base, #202020);
		border-radius: var(--fds-overlay-corner-radius, 8px);
		border: 1px solid var(--fds-card-stroke-default, rgba(255, 255, 255, 0.08));
		box-shadow: var(--fds-dialog-shadow, 0 16px 32px rgba(0, 0, 0, 0.37));
		overflow: hidden;
		color: var(--fds-text-primary, #fff);
	}

	#page {
		display: flex;
		height: 100%;
		width: 100%;
		overflow: hidden;
	}

	#background {
		position: relative;
		display: flex;
		width: clamp(25rem, 40%, 30rem);
		background-color: var(--fds-solid-background-base, #202020);
		background-image: linear-gradient(#000, #0d2a3f);
		user-select: none;
		pointer-events: none;
	}

	#background img {
		object-fit: cover;
		width: 100%;
		height: 100%;
		flex: 1;
	}

	/* Sitenin kendi kuralı. Dar pencerede görsel gizlenir, form tüm genişliği
	   alır — önizlemede sitenin giriş modalının görselsiz görünmesinin sebebi
	   de tam olarak budur (önizleme paneli 1096px'ten dar). */
	@media (width <= 1096px) {
		#background {
			display: none;
		}
	}

	#auth {
		position: relative;
		display: flex;
		margin: auto;
		flex: 1;
		justify-content: center;
		align-items: center;
		flex-direction: column;
		height: 100%;
		padding: 0 1rem;
		background-color: var(--fds-layer-on-acrylic-background-default, #2b2b2b);
	}

	#box {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		justify-content: center;
		align-items: center;
		border-radius: 1rem;
		padding-inline: 1rem;
		width: clamp(20rem, 80%, 30rem);
		max-height: 100%;
		overflow-y: auto;
	}

	.qr-frame {
		display: flex;
		align-items: center;
		justify-content: center;
		align-self: center;
		width: 220px;
		height: 220px;
		/* QR'ın okunabilmesi için zemin her temada beyaz olmalı; koyu temada
		   token zeminine bırakılsaydı kontrast tersine döner ve telefon kodu
		   okuyamazdı. */
		background-color: #fff;
		border-radius: var(--fds-control-corner-radius, 4px);
		padding: 8px;
		box-sizing: border-box;
	}

	.qr-frame img {
		width: 100%;
		height: 100%;
		object-fit: contain;
		-webkit-user-drag: none;
		user-select: none;
	}

	#box form,
	#box .qr-step,
	#box .stack {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		width: 100%;
	}

	#box :global(.text-box-container) {
		width: 100%;
	}

	/* Sitenin `.text-block` tipografisi; ölçüler --fds-* token'larından. */
	.type-title {
		font-size: var(--fds-title-font-size, 28px);
		font-weight: 600;
		margin: 0;
		text-align: center;
	}

	.type-body {
		font-size: var(--fds-body-font-size, 14px);
		text-align: center;
	}

	.type-body-strong {
		font-size: var(--fds-body-font-size, 14px);
		font-weight: 600;
		margin: 0;
	}

	.text-tertiary {
		color: var(--fds-text-tertiary, rgba(255, 255, 255, 0.54));
	}

	hr.horizontal {
		width: 100%;
		border: none;
		border-top: 1px solid var(--fds-divider-stroke-default, rgba(255, 255, 255, 0.08));
		margin: 0.5rem 0 0;
	}

	.notice {
		border-radius: var(--fds-control-corner-radius, 4px);
		padding: 8px 12px;
	}

	.notice-error {
		background-color: var(--fds-system-fill-color-critical-background, rgba(255, 99, 71, 0.1));
		color: var(--fds-system-fill-color-critical, #ff99a4);
	}

	.notice-warn {
		background-color: var(--fds-system-fill-color-caution-background, rgba(255, 244, 206, 0.08));
		color: var(--fds-system-fill-color-caution, #fce100);
	}

	#close-button {
		position: absolute;
		top: 12px;
		right: 12px;
		z-index: 2;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border: none;
		border-radius: var(--fds-control-corner-radius, 4px);
		background-color: var(--fds-subtle-fill-transparent, transparent);
		color: var(--fds-text-primary, #fff);
		cursor: default;
	}

	#close-button:hover {
		background-color: var(--fds-subtle-fill-secondary, rgba(255, 255, 255, 0.06));
	}

	#close-button:active {
		background-color: var(--fds-subtle-fill-tertiary, rgba(255, 255, 255, 0.04));
	}
</style>
