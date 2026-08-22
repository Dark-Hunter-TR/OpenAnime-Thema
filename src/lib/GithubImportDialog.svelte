<script lang="ts">
	/**
	 * GitHub'dan tema içe aktarma diyaloğu.
	 *
	 * `Launcher.svelte`'den ÇIKARILDI çünkü artık iki yerden açılıyor: ana
	 * ekrandaki "GitHub'dan içe aktar" düğmesi ve Editör'e açık proje olmadan
	 * girildiğinde çıkan başlangıç seçicisi (`+page.svelte`). Ana ekranın
	 * içinde kalsaydı ikinci çağıran onu ancak ana ekrana geçerek açabilirdi;
	 * kullanıcı "GitHub'dan içe aktar" dediğinde araya bir görünüm değişimi
	 * girmesi gereksiz bir sıçrama olurdu.
	 *
	 * İki adımlı: önce bağlantı çözümlenip aday `.css` dosyaları bulunuyor,
	 * sonra seçilen dosya indirilip projeye dönüştürülüyor.
	 */
	import {
		Button,
		ComboBox,
		ContentDialog,
		ProgressRing,
		TextBlock,
		TextBox
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

	/** Diyalog açık mı. `bind:open` ile kullanılmalı. */
	export let open = false;
	export let onImport: (payload: { css: string; name: string; source: string }) => void;

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

	function reset() {
		importUrl = "";
		importName = "";
		importError = "";
		importBusy = false;
		candidates = [];
		chosenPath = "";
		importSource = "";
	}

	// Diyalog her AÇILDIĞINDA sıfırlanıyor. Kapanışta sıfırlamak yetmezdi:
	// diyalog dışarıdan `bind:open` ile de açılabiliyor ve bir önceki denemenin
	// hata metni ya da yarım kalan aday listesi ekranda kalırdı.
	let wasOpen = false;
	$: if (open && !wasOpen) reset();
	$: wasOpen = open;

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
			open = false;
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

<ContentDialog bind:open title="GitHub'dan tema içe aktar" size="standard">
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
		<Button on:click={() => (open = false)}>Vazgeç</Button>
	</svelte:fragment>
</ContentDialog>

<style>
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
