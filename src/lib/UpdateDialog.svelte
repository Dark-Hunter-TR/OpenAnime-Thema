<script lang="ts">
	/**
	 * "Yeni sürüm mevcut" diyaloğu.
	 *
	 * `DialogShell`'i `AboutDialog.svelte` ile PAYLAŞIYOR — aynı banner/logo/
	 * Setsuki başlığı, aynı kapatma animasyonu. Yalnızca `#content` ve durum
	 * makinesi (indirme ilerlemesi, hata, başarı) buraya özel.
	 */
	import { Button, ProgressBar, TextBlock } from "fluent-svelte-extra";

	import DialogShell from "$lib/DialogShell.svelte";
	import { downloadAndInstallUpdate, type DownloadEvent, type Update } from "$lib/updater";

	export let open = false;
	export let update: Update | null = null;
	/** Diyaloğu bu oturum için kapatır — sürümü ATLAMAZ, bir sonraki açılışta
	 * yine sorulur. */
	export let onClose: () => void;
	/** "Daha Sonra Hatırlat" — bu SÜRÜMÜ kalıcı olarak atlar (bkz.
	 * `settings.ts` -> `updateSkipVersion`). Yeni bir sürüm çıkana kadar
	 * tekrar sorulmaz. */
	export let onSkip: () => void;

	type Status = "idle" | "downloading" | "installing" | "success" | "error";

	let status: Status = "idle";
	let percent = 0;
	let errorMessage = "";

	let downloadedBytes = 0;
	let totalBytes = 0;

	// İndirme başladıktan sonra yanlışlıkla kapatılmasın diye kilitliyoruz —
	// installer NSIS ile çalışıp süreci yeniden başlatana kadar diyalog açık
	// kalmalı, X/Escape/arka plan tıklaması hiçbir şey yapmamalı.
	$: closable = status === "idle" || status === "error";

	$: subtitle = update
		? `Sürüm v${update.version}${update.date ? ` — ${formatDate(update.date)}` : ""}`
		: "";

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString("tr-TR");
	}

	function handleEvent(event: DownloadEvent) {
		if (event.event === "Started") {
			status = "downloading";
			downloadedBytes = 0;
			totalBytes = event.data.contentLength ?? 0;
			percent = 0;
		} else if (event.event === "Progress") {
			downloadedBytes += event.data.chunkLength;
			percent = totalBytes > 0 ? Math.min(100, Math.round((downloadedBytes / totalBytes) * 100)) : 0;
		} else if (event.event === "Finished") {
			status = "installing";
			percent = 100;
		}
	}

	async function startDownload() {
		if (!update || status !== "idle") return;
		status = "downloading";
		try {
			await downloadAndInstallUpdate(update, handleEvent);
			status = "success";
			// `restart_app` normalde süreci burada zaten sonlandırıyor; bu satıra
			// yalnızca (ör. hızlı art arda tıklamalarda) yarış durumunda ulaşılır.
		} catch (e) {
			status = "error";
			errorMessage = String(e);
		}
	}
</script>

<DialogShell {open} {onClose} title="Yeni Sürüm Mevcut!" {subtitle} {closable}>
	{#if update}
		<h4 class="text-block type-subtitle svelte-9tjxrp">Sürüm Notları</h4>
		<span class="text-block type-body text-tertiary svelte-9tjxrp changelog">
			{update.body?.trim() || "Herhangi bir sürüm notu bulunmuyor."}
		</span>

		{#if status === "idle" || status === "error"}
			{#if status === "error"}
				<div class="status-line status-error">
					<TextBlock variant="caption">Güncelleme başarısız: {errorMessage}</TextBlock>
				</div>
			{/if}
			<hr class="horizontal svelte-cc3kyp" />
			<div id="buttons" class="action-buttons">
				<Button on:click={onSkip}>Daha Sonra Hatırlat</Button>
				<Button variant="accent" on:click={startDownload}>İndir ve Kur</Button>
			</div>
		{:else}
			<div class="progress-panel">
				<div class="progress-row">
					<TextBlock variant="caption">
						{#if status === "downloading"}
							Güncelleme indiriliyor…
						{:else if status === "installing"}
							İndirme bitti, kuruluyor…
						{:else if status === "success"}
							Kurulum başlatıldı, uygulama yeniden başlatılıyor…
						{/if}
					</TextBlock>
					<!-- `totalBytes` sunucudan hiç gelmeyebilir (bazı CDN'ler
					     Content-Length atlar) — o durumda belirsiz (indeterminate)
					     çubuğa düşüyoruz, yüzde göstermek yanıltıcı olurdu. -->
					{#if status === "downloading" && totalBytes > 0}
						<TextBlock variant="caption" class="text-tertiary">{percent}%</TextBlock>
					{/if}
				</div>
				<ProgressBar value={status === "downloading" && totalBytes > 0 ? percent : undefined} />
			</div>
		{/if}
	{/if}
</DialogShell>

<style>
	.changelog {
		display: block;
		white-space: pre-wrap;
		max-height: 180px;
		overflow-y: auto;
	}

	.action-buttons {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		width: 100%;
	}

	.progress-panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
		background-color: rgba(255, 255, 255, 0.03);
		border: 1px solid var(--fds-divider-stroke-default, rgba(255, 255, 255, 0.08));
		border-radius: var(--fds-control-corner-radius, 4px);
		padding: 12px;
	}

	.progress-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	.status-error {
		color: var(--fds-system-critical, #ff7b72);
	}
</style>
