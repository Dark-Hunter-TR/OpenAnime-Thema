/**
 * fluent-svelte-extra olayları için tip köprüsü.
 *
 * Kütüphanenin bileşenleri DOM olaylarını `createEventForwarder` ile yeniden
 * yayıyor ve ürettiği tip bildirimlerinde hepsi `CustomEvent<any>` olarak
 * görünüyor. Çalışma anında iletilen nesne gerçekte DOM'un kendi
 * `KeyboardEvent`'i — yani `event.key` her zaman var, yalnızca tip bilgisi
 * kayboluyor.
 *
 * Bu yüzden her çağrı yerinde ayrı ayrı `as` yazmak yerine dönüşümü tek bir
 * yerde topluyoruz: bir gün kütüphane tiplerini düzeltirse değiştirilecek tek
 * dosya burası olur.
 */

/** `on:keydown` ile gelebilecek iki biçim. */
export type ForwardedKeyEvent = KeyboardEvent | CustomEvent<unknown>;

/** İletilen olaydan basılan tuşu okur. */
export function keyOf(event: ForwardedKeyEvent): string {
	return (event as KeyboardEvent).key;
}

/** Enter'a basıldı mı? */
export function isEnter(event: ForwardedKeyEvent): boolean {
	return keyOf(event) === "Enter";
}
