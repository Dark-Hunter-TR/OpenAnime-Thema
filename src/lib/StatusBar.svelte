<script lang="ts">
	/**
	 * Durum / bilgi kutusu.
	 *
	 * `InfoBar`'ı YENİDEN YAZMIYOR, sarıyor. Tek işi, önem derecesine göre doğru
	 * Fluent System Icon'unu `icon` slot'una bağlamak — çünkü `InfoBar`'ın
	 * varsayılanı `InfoBadge`: 16px'lik renkli bir hapın içinde 8px'lik çıplak
	 * bir glif. Sitenin durum kutularında ise 20px'lik daireli ikonlar var
	 * (`CheckmarkCircle`, `Info`, `Warning`, `ErrorCircle`).
	 *
	 * Kutunun rengi, dolgusu ve tipografisi burada değil, `+layout.svelte`'teki
	 * genel `.info-bar` kurallarında: tek bir kutu değil, uygulamadaki BÜTÜN
	 * durum kutuları aynı zemin/kenarlık/metin üçlüsünü kullanmalı.
	 *
	 * Bileşen ayrı bir dosya, çünkü uygulamada 16 çağrı yeri var; ikon eşlemesini
	 * her birine tek tek yazmak, bir gün bir önem derecesi eklendiğinde on altı
	 * yerde unutulacak bir tekrar olurdu.
	 */
	import { InfoBar } from "fluent-svelte-extra";

	import Icon from "$lib/Icon.svelte";
	import type { IconName } from "$lib/icons";

	/** Tip dışa aktarılmıyor: Svelte örnek script'inden tip export edilemiyor
	    (bunun için `context="module"` gerekir) ve dışarıda kullanan yok. */
	type Severity = "information" | "success" | "caution" | "critical" | "attention";

	export let severity: Severity = "information";
	export let title = "";
	export let message = "";
	export let closable = true;

	/**
	 * `attention` ile `information` aynı ikonu paylaşıyor: ikisi de "bilgi"
	 * anlamında, farkları yalnızca vurgu derecesi (renk).
	 */
	const ICONS: Record<Severity, IconName> = {
		information: "statusInfo",
		attention: "statusInfo",
		success: "statusSuccess",
		caution: "statusCaution",
		critical: "statusCritical"
	};
</script>

<InfoBar {severity} {title} {message} {closable}>
	<Icon slot="icon" name={ICONS[severity]} size={20} />
	<slot />
	<!--
		`slot="action"` koşulsuz iletiliyor: Svelte'te bir slot'u `{#if}` içine
		almak `InfoBar`'ın gördüğü `$$slots.action`'ı değiştirmiyor, yani kutu
		her hâlükârda aksiyon sarmalayıcısını çiziyor. Boş kaldığında görünmesin
		diye `+layout.svelte`'te `.info-bar-action:empty { display: none }` var.
	-->
	<svelte:fragment slot="action"><slot name="action" /></svelte:fragment>
</InfoBar>
