<script lang="ts">
	import { fade, scale } from "svelte/transition";
	import { circOut } from "svelte/easing";

	export let open = false;
	export let appVersion = "v0.1.0";
	export let onClose: () => void;

	// openani.me orijinal Setsuki havuzu ve ağırlık algoritması (1:1 analiz sonucu)
	const setsukiNames = [
		"sitting",
		"standing",
		"jumping",
		"leaning",
		"straight-on",
		"looking-down",
		"pajamas"
	];
	const setsukiWeights = [16, 16, 16, 16, 16, 16, 4];

	function getWeightedRandomSetsuki(): string {
		const pool = setsukiWeights.flatMap((weight, index) => Array(weight).fill(index));
		const selectedIndex = pool.sort(() => Math.random() - 0.5)[0];
		return `/setsuki/${setsukiNames[selectedIndex]}.png`;
	}

	let currentMascot = `/setsuki/${setsukiNames[0]}.png`;

	$: if (open) {
		currentMascot = getWeightedRandomSetsuki();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === "Escape" && open) {
			onClose();
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
	<!-- svelte-ignore a11y-click-events-have-key-events -->
	<!--
		Süreler ve eğri fluent-svelte-extra'nın KENDİ `ContentDialog`'undan
		(takipçi/takip edilen modalı bunu kullanıyor): `content-dialog-smoke`
		`--fds-control-faster-duration` (83ms) ile fade'liyor,
		`content-dialog-container` `--fds-control-fast-duration` (167ms) +
		`circOut` ile scale'liyor (bkz. theme.css). Kütüphanenin `getCSSDuration`
		yardımcısı yalnızca içeride kullanılabiliyor, paketin dışa açtığı
		bir şey değil — o yüzden sabit sayıları elle taşıdık.
	-->
	<div
		class="content-dialog-container svelte-f1dwd4"
		transition:fade={{ duration: 83 }}
		on:click|self={onClose}
		role="presentation"
	>
		<!--
			`transition:scale` KASITLI olarak `.content-dialog-wrap`'te — yani
			dialog kutusunu VE kapatma düğmesini BİRLİKTE sarıyor. Önceki
			sürümde yalnızca `.content-dialog`'un kendi geçişi vardı, düğme ise
			bir kardeş olarak dışarıda kalıp yalnızca üstteki `fade`'den
			opaklık alıyordu — ikisi farklı hızda beliriyor, düğme dialogdan
			önce "zaten oradaymış" gibi görünüyordu. Kütüphanenin kendi
			`ContentDialog`'u (takipçi/takip edilen modalındaki kapatma
			düğmesi) tam olarak bunu yapıyor: `content-dialog-container`
			(dialog + düğme) TEK geçişle birlikte beliriyor (bkz.
			node_modules/fluent-svelte-extra/ContentDialog/ContentDialog.svelte).
			Aynı tekniği burada da uyguluyoruz.
		-->
		<div
			class="content-dialog-wrap"
			transition:scale={{ duration: 167, start: 1.05, easing: circOut }}
		>
			<div
				class="content-dialog size-max svelte-f1dwd4"
				role="dialog"
				aria-modal="true"
				id="about-dialog"
			>
				<div class="content-dialog-body svelte-f1dwd4">
					<div id="main" class="fds-theme-dark svelte-cc3kyp">
						<div id="card" class="svelte-cc3kyp">
							<div class="image-wrapper no-select loaded svelte-zi2j2b" id="logo">
								<img alt="OpenAnime Logo" src="/favicon512_white.png" class="svelte-zi2j2b" />
							</div>
							<div id="info" class="fds-theme-dark svelte-cc3kyp">
								<h4 class="text-block type-subtitle svelte-9tjxrp">OpenAnime Tema Editörü</h4>
								<span class="text-block type-caption text-tertiary svelte-9tjxrp">
									Sürüm {appVersion.startsWith('v') ? appVersion : `v${appVersion}`}
								</span>
							</div>
						</div>
						<div class="image-wrapper no-select loaded svelte-zi2j2b" id="setsuki">
							<img alt="Setsuki" src={currentMascot} class="svelte-zi2j2b" />
						</div>
					</div>

					<div id="content" class="svelte-cc3kyp">
						<h4 class="text-block type-subtitle svelte-9tjxrp">Hakkında</h4>
						<span class="text-block type-body text-tertiary svelte-9tjxrp">
							Bu uygulama, openani.me sitesi için temalar oluşturmanıza, görsel ve kod düzenleyicileri ile
							canlı olarak özelleştirmenize ve .css temaları dışa aktarmanıza olanak sağlar. Üretilen temalar
							sitenin resmî tema sistemi ile %100 uyumludur.
						</span>
						<hr class="horizontal svelte-cc3kyp" />
						<div id="buttons" class="svelte-cc3kyp">
							<a
								role="button"
								href="https://openani.me/tos"
								target="_blank"
								rel="noreferrer"
								class="button style-hyperlink svelte-nqc07q"
								tabindex="0"
							>
								Hizmet Sözleşmesi
							</a>
							<a
								role="button"
								href="https://openani.me/cookies"
								target="_blank"
								rel="noreferrer"
								class="button style-hyperlink svelte-nqc07q"
								tabindex="0"
							>
								Çerez Politikası
							</a>
							<a
								role="button"
								href="https://openani.me/privacy"
								target="_blank"
								rel="noreferrer"
								class="button style-hyperlink svelte-nqc07q"
								tabindex="0"
							>
								Gizlilik Politikası
							</a>
						</div>
					</div>
				</div>
			</div>

			<button id="close-button" aria-label="Close dialog" tabindex="0" class="svelte-f1dwd4" on:click={onClose}>
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
		</div>
	</div>
{/if}

<style>
	.content-dialog-container {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 9999;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--fds-smoke-background-default, rgba(0, 0, 0, 0.45));
		padding: 16px;
	}

	.content-dialog-wrap {
		display: flex;
		flex-direction: row;
		align-items: flex-start;
		justify-content: center;
		gap: 8px;
		max-width: 600px;
		width: 100%;
	}

	.content-dialog {
		position: relative;
		width: 100%;
		max-width: 540px;
		background-color: var(--fds-solid-background-base, #202020);
		border-radius: var(--fds-overlay-corner-radius, 8px);
		border: 1px solid var(--fds-card-stroke-default, rgba(255, 255, 255, 0.08));
		box-shadow: var(--fds-dialog-shadow, 0 16px 32px rgba(0, 0, 0, 0.37));
		overflow: hidden;
		color: var(--fds-text-primary, #fff);
	}

	.content-dialog-body {
		display: flex;
		flex-direction: column;
		padding: 0 !important;
	}

	#main {
		position: relative;
		display: flex;
		justify-content: space-between;
		align-items: center;
		background-image: url("/about-banner-base.png");
		background-size: cover;
		background-position: center;
		border-top-left-radius: var(--fds-overlay-corner-radius, 8px);
		border-top-right-radius: var(--fds-overlay-corner-radius, 8px);
		height: 10rem;
		overflow: hidden;
	}

	#card {
		position: relative;
		display: flex;
		align-items: center;
		gap: 1rem;
		width: fit-content;
		margin-left: 24px;
		margin-bottom: 0;
		z-index: 2;
	}

	#logo {
		width: 3rem;
		height: 3rem;
		flex-shrink: 0;
	}

	#logo img {
		width: 100%;
		height: 100%;
		object-fit: contain;
		border-radius: var(--fds-overlay-corner-radius, 8px);
	}

	#info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		color: var(--fds-text-primary, #ffffff);
	}

	#info h4 {
		margin: 0;
		font-size: 20px;
		font-weight: 600;
		color: #ffffff;
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
	}

	#info span {
		font-size: 12px;
		color: rgba(255, 255, 255, 0.85);
		text-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
	}

	#setsuki {
		position: absolute;
		right: 16px;
		bottom: 0;
		height: 100%;
		aspect-ratio: 1;
		object-fit: contain;
		user-select: none;
		pointer-events: none;
		filter: drop-shadow(0 0 0.5rem hsla(0, 0%, 0%, 0.25));
		z-index: 1;
	}

	#setsuki img {
		height: 100%;
		width: auto;
		object-fit: contain;
	}

	#content {
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	#content h4 {
		margin: 0;
		font-size: 18px;
		font-weight: 600;
		color: var(--fds-text-primary, #ffffff);
	}

	#content span {
		font-size: 13px;
		line-height: 1.5;
		color: var(--fds-text-tertiary, rgba(255, 255, 255, 0.54));
	}

	hr.horizontal {
		border: none;
		border-top: 1px solid var(--fds-divider-stroke-default, rgba(255, 255, 255, 0.08));
		height: 1px;
		margin: 1rem 0;
	}

	#buttons {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		width: 100%;
	}

	#buttons a.button.style-hyperlink {
		box-sizing: border-box;
		height: 32px;
		padding: 0 11px;
		font-size: 14px;
		font-weight: 400;
		line-height: 20px;
		color: var(--fds-accent-text-primary, #60cdff);
		background-color: var(--fds-subtle-fill-transparent, transparent);
		border: 1px solid transparent;
		border-radius: var(--fds-control-corner-radius, 4px);
		text-decoration: none;
		text-align: center;
		transition: background-color var(--fds-control-fast-duration, 0.15s) ease,
			color var(--fds-control-fast-duration, 0.15s) ease;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		white-space: nowrap;
		cursor: pointer;
		user-select: none;
	}

	#buttons a.button.style-hyperlink:hover {
		background-color: var(--fds-subtle-fill-secondary, rgba(255, 255, 255, 0.06));
		color: var(--fds-accent-text-secondary, var(--fds-accent-text-primary, #60cdff));
		text-decoration: none;
	}

	#buttons a.button.style-hyperlink:active {
		background-color: var(--fds-subtle-fill-tertiary, rgba(255, 255, 255, 0.04));
		color: var(--fds-accent-text-tertiary, var(--fds-accent-text-primary, #60cdff));
	}

	#close-button {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		margin-left: 8px;
		padding: 0;
		border: 1px solid var(--fds-surface-stroke-default, rgba(255, 255, 255, 0.08));
		border-radius: var(--fds-overlay-corner-radius, 8px);
		background-color: var(--fds-control-on-image-fill-default, rgba(0, 0, 0, 0.25));
		background-clip: padding-box;
		color: var(--fds-text-primary, #ffffff);
		cursor: pointer;
		flex-shrink: 0;
		transition: background-color var(--fds-control-fast-duration, 0.15s) ease,
			color var(--fds-control-fast-duration, 0.15s) ease;
	}

	#close-button:hover {
		background-color: var(--fds-control-on-image-fill-secondary, rgba(255, 255, 255, 0.08));
	}

	#close-button:active {
		background-color: var(--fds-control-on-image-fill-tertiary, rgba(255, 255, 255, 0.04));
	}
</style>
