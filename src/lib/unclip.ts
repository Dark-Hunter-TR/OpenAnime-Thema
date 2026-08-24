/**
 * Açılır katmanları üst kutuların `overflow` kırpmasından kurtaran eylem.
 *
 * Sorun `Tooltip.svelte`'te anlatılanla aynı ama farklı bileşenlerde:
 * fluent-svelte-extra'nın `Flyout`, `MenuFlyout` ve `ComboBox`'ı açılır
 * katmanı tetikleyicinin yanında, `position: absolute` bir kardeş olarak
 * açıyor. Katman böylece normal akışın parçası oluyor ve `overflow` taşıyan
 * her üst kutu onu kesiyor:
 *
 *   * Editör paneli (`.panel`) ve ayarlar sayfası `overflow-y: auto` taşıyor —
 *     CSS'te bir eksen `visible` dışındaysa diğeri de `auto`ya düştüğü için
 *     yatayda da kırpıyorlar. Panelin altına yakın bir renk alanının paleti,
 *     bir açılır listenin son maddeleri bu yüzden görünmüyordu.
 *   * Ana ekranda proje kartlarının "⋮" menüsü, kartlar ızgarasının kaydırma
 *     kutusu içinde kalıyordu.
 *
 * Neden yalnızca `position: fixed` yetmiyor: sabit konumlu bir kutu, üst
 * kutulardan biri `transform`/`filter`/`will-change` taşıyorsa görünüm alanına
 * değil O KUTUYA göre konumlanır. Ana ekrandaki `.card:hover` tam olarak bir
 * `transform` uyguluyor ve menü açıkken fare çoğu zaman kartın üstünde; katman
 * yerinde bırakılsaydı menü kartın hover durumuna göre yer değiştirirdi. Bu
 * yüzden katman `document.body`ye TAŞINIYOR — orada üstünde dönüşüm taşıyan
 * bir ata kalmıyor.
 *
 * Kullanımı: kırpan kutunun içindeki bileşeni `display: contents` bir kapsayıcı
 * ile sarmalayıp bu eylemi ona vermek. `contents` seçildi ki kapsayıcı hiçbir
 * kutu üretmesin ve mevcut yerleşim birebir korunsun.
 *
 *   <div class="unclip" use:unclip>
 *     <Flyout …>…</Flyout>
 *   </div>
 */

/** Kütüphanenin akış içinde açtığı katmanlar. */
const OVERLAY_SELECTOR = ".flyout-anchor, .menu-flyout-anchor, .combo-box-dropdown";

/**
 * ComboBox'ın açılır listesi — kendi yolundan gitmesi GEREKEN katman.
 *
 * ## Neden ayrı ele alınıyor
 *
 * Diğer iki katmanda düğümün kendisine satır içi stil yazıp konumu oradan
 * veriyoruz. ComboBox'ta bu ÇALIŞMIYOR, çünkü listenin `<ul>`'u kütüphanede
 * düz bir `style` attribute'u taşıyor:
 *
 *     style="--fds-menu-offset: {menuOffset}px; …"
 *
 * `menuOffset` reaktif; Svelte onu her güncellediğinde `setAttribute("style", …)`
 * çağrılıyor ve bu, attribute'un TAMAMINI yeniden yazıyor — yani buradan
 * yazdığımız `position: fixed`, `inset`, `left`, `top` siliniyor. Liste
 * stylesheet'teki `position: absolute`e geri düşüyor; `document.body` içinde
 * konumlandırılmış bir atası kalmadığı için kapsayıcı bloğu görünüm alanı
 * oluyor ve `inline-size: calc(100% + 8px)` PENCERE genişliğine çözülüyor.
 * Ekranda görülen "liste sol üst köşede, pencere boyunca uzuyor" tam olarak
 * bu (1200px'lik pencerede ölçüldü: liste 1208px, x = -5).
 *
 * ## Çözüm: düğüme hiç dokunmamak
 *
 * Liste, konumlandırılmış bir SARMALAYICININ içine alınıyor ve konum o
 * sarmalayıcıya yazılıyor. Listenin kendi satır içi stiline hiç
 * dokunulmadığı için kütüphane onu istediği kadar yeniden yazabilir; konum
 * artık orada tutulmuyor. Üstelik listenin kendi kuralları da böylece doğru
 * çalışır hâle geliyor: `position: absolute` sarmalayıcıya göre çözülüyor,
 * `inline-size: calc(100% + 8px)` ise sarmalayıcının genişliğine — ki onu
 * tetikleyicinin genişliğine eşitliyoruz. Yani kütüphanenin amaçladığı
 * geometri, hiçbir değeri ezmeden geri geliyor.
 */
const COMBO_SELECTOR = ".combo-box-dropdown";

/** Görünür alanın kenarına bırakılan asgari pay. */
const MARGIN = 8;

interface Tracked {
	node: HTMLElement;
	/** Katmanın taşınmadan önceki kutusu — konum bundan hesaplanıyor. */
	reference: HTMLElement;
	/** Taşımadan önceki satır içi `position`; geri yüklemek için. */
	previousPosition: string;
	/**
	 * Yalnızca ComboBox listesinde dolu: konumu taşıyan kutu (bkz.
	 * `COMBO_SELECTOR`). Doluysa katmanın kendi stiline dokunulmuyor.
	 */
	wrapper?: HTMLElement;
	/** Sarmalayıcı boşaldığında (liste kapandığında) temizliği tetikler. */
	wrapperObserver?: MutationObserver;
	/** Kütüphane listenin stilini tazeleyince yeniden sığdırmayı tetikler. */
	styleObserver?: MutationObserver;
}

export function unclip(host: HTMLElement) {
	const tracked = new Map<HTMLElement, Tracked>();

	const reposition = () => {
		for (const entry of tracked.values()) place(entry);
	};

	const adopt = (node: HTMLElement) => {
		if (tracked.has(node)) return;

		// Referans, katmanın ŞU ANKİ ebeveyni: kütüphanenin sarmalayıcısı,
		// yani tetikleyici düğmenin kutusu. Taşımadan önce yakalanmalı.
		const reference = node.parentElement;
		if (!reference) return;

		// ComboBox listesi kendi stilini yeniden yazdığı için ayrı yoldan
		// gidiyor: düğüme hiç dokunmuyoruz, konumu bir sarmalayıcı taşıyor.
		if (node.matches(COMBO_SELECTOR)) {
			const wrapper = document.createElement("div");
			wrapper.style.position = "fixed";

			const entry: Tracked = {
				node,
				reference,
				previousPosition: node.style.position,
				wrapper
			};
			tracked.set(node, entry);

			document.body.appendChild(wrapper);
			wrapper.appendChild(node);

			// Kapanışı sarmalayıcıdan izliyoruz: liste artık `document.body`nin
			// DOĞRUDAN çocuğu değil, dolayısıyla aşağıdaki gövde gözlemcisi onu
			// göremez. Sarmalayıcı boşaldığı an katman kapanmış demektir.
			const wrapperObserver = new MutationObserver(() => {
				if (wrapper.childElementCount === 0) release(node);
			});
			wrapperObserver.observe(wrapper, { childList: true });
			entry.wrapperObserver = wrapperObserver;

			// Listenin kendi `style`ını izliyoruz. Kütüphane `--fds-menu-offset`i
			// menü açıldıktan SONRA, bir `tick()` gecikmesiyle hesaplıyor
			// (ComboBox -> `openMenu`); ilk ölçümümüz o yüzden bayat kalıyor ve
			// kısa pencerelerde liste alt kenardan taşıyordu. Stil her
			// değiştiğinde yeniden sığdırıyoruz.
			//
			// Geri besleme riski yok: bu yolda listenin stiline hiç yazmıyoruz,
			// yalnızca sarmalayıcıyı oynatıyoruz.
			const styleObserver = new MutationObserver(() => placeWrapper(entry));
			styleObserver.observe(node, { attributes: true, attributeFilter: ["style"] });
			entry.styleObserver = styleObserver;

			place(entry);
			return;
		}

		const entry: Tracked = { node, reference, previousPosition: node.style.position };
		tracked.set(node, entry);

		document.body.appendChild(node);
		node.style.position = "fixed";
		// Kütüphane yerleşimi `inset-*` ve `transform` ile yapıyor; taşınan
		// katmanda ikisi de yanlış sonuç verir çünkü artık başka bir kutunun
		// içinde değil.
		node.style.inset = "auto";
		node.style.transform = "none";
		// Açılış animasyonu `transform: var(--fds-flyout-transform)` ile
		// bitiyor ve CSS animasyonları satır içi stili ezdiği için yalnızca
		// `transform: none` yazmak yetmiyor: hizalaması `center` olan bir
		// katman animasyon boyunca yarım genişlik kayar, sonra yerine
		// zıplardı. Değişkenin kendisini nötrleyerek animasyonun bitiş karesi
		// de dönüşümsüz oluyor.
		node.style.setProperty("--fds-flyout-transform", "none");
		place(entry);
	};

	const release = (node: HTMLElement) => {
		const entry = tracked.get(node);
		if (!entry) return;
		tracked.delete(node);

		if (entry.wrapper) {
			// Katmanın kendi stiline hiç yazmadık; geri yüklenecek bir şey yok.
			entry.wrapperObserver?.disconnect();
			entry.styleObserver?.disconnect();
			entry.wrapper.remove();
			return;
		}

		node.style.position = entry.previousPosition;
		node.remove();
	};

	// Katmanlar açılıp kapandıkça DOM'a girip çıkıyor; hangi anda olacağını
	// bileşen belirlediği için gözlemleyerek yakalıyoruz.
	//
	// Burada YALNIZCA eklenenlere bakılıyor, kaldırılanlara DEĞİL. Sebebi ince
	// ama belirleyici: `adopt` düğümü gövdeye taşırken, bu taşıma eski ebeveyn
	// için bir "kaldırıldı" kaydı üretiyor. O kaydı da işleseydik — ki önce
	// öyleydi — kendi taşımamızı "katman kapandı" sanıp düğümü DOM'dan
	// siliyorduk: menü açıldığı anda yok oluyordu. Gerçek kapanmayı aşağıdaki
	// gövde gözlemcisi görüyor.
	const observer = new MutationObserver((records) => {
		for (const record of records) {
			for (const added of record.addedNodes) {
				if (!(added instanceof HTMLElement)) continue;
				if (added.matches(OVERLAY_SELECTOR)) adopt(added);
				else added.querySelectorAll<HTMLElement>(OVERLAY_SELECTOR).forEach(adopt);
			}
		}
	});
	observer.observe(host, { childList: true, subtree: true });

	// Gerçek kapanma: katmanı kapatan bileşen artık `host`un değil
	// `document.body`nin çocuğunu siliyor. Burada yalnızca kaydı düşürüyoruz;
	// düğümü Svelte zaten kaldırdı.
	const bodyObserver = new MutationObserver((records) => {
		for (const record of records) {
			for (const removed of record.removedNodes) {
				if (removed instanceof HTMLElement) tracked.delete(removed);
			}
		}
	});
	bodyObserver.observe(document.body, { childList: true });

	// `capture: true`: kırpan kutu pencere değil içerideki kaydırılabilir
	// panel; kabarcık aşamasında onun kaydırması duyulmaz.
	window.addEventListener("scroll", reposition, true);
	window.addEventListener("resize", reposition);

	return {
		destroy() {
			observer.disconnect();
			bodyObserver.disconnect();
			window.removeEventListener("scroll", reposition, true);
			window.removeEventListener("resize", reposition);
			for (const node of [...tracked.keys()]) release(node);
		}
	};
}

/** Katmanı türüne göre doğru yerleştiriciye yollar. */
function place(entry: Tracked) {
	if (entry.wrapper) {
		placeWrapper(entry);
		return;
	}
	placeNode(entry);
}

/**
 * ComboBox listesini taşıyan sarmalayıcıyı yerleştirir.
 *
 * Sarmalayıcı tetikleyicinin dikdörtgenine oturtuluyor ve GENİŞLİĞİ ona
 * eşitleniyor — listenin `inline-size: calc(100% + 8px)` kuralının doğru
 * sonuç vermesi için gereken tek şey bu.
 *
 * Dikey yerleşimi kütüphaneye BIRAKIYORUZ: liste `inset-block-start:
 * var(--fds-menu-offset)` ile seçili maddeyi tetikleyicinin üstüne
 * hizalıyor, ki Fluent'in amaçladığı davranış bu. Biz yalnızca sonucu ölçüp
 * görünür alanın dışına taşan kadarını geri kaydırıyoruz. Ofseti kendimiz
 * yeniden hesaplamıyoruz — kütüphanenin iç matematiğini burada
 * tekrarlamak, sürüm değiştiğinde sessizce yanlışa düşerdi.
 */
function placeWrapper({ node, reference, wrapper }: Tracked) {
	if (!wrapper) return;

	const a = reference.getBoundingClientRect();
	wrapper.style.inlineSize = `${a.width}px`;
	wrapper.style.left = `${Math.round(a.left)}px`;
	wrapper.style.top = `${Math.round(a.top)}px`;

	// Liste mutlak konumlu, yani sarmalayıcı ölçülemez (yüksekliği 0).
	// Görünür alan denetimi bu yüzden listenin KENDİ kutusundan yapılıyor.
	const t = node.getBoundingClientRect();
	const vw = document.documentElement.clientWidth;
	const vh = document.documentElement.clientHeight;

	let dx = 0;
	if (t.left < MARGIN) dx = MARGIN - t.left;
	else if (t.right > vw - MARGIN) dx = vw - MARGIN - t.right;

	// Sıra önemli: liste görünür alandan uzunsa iki koşul da doğru olur ve
	// üst kenarı içeride tutmak alt kenarı kurtarmaya yeğdir — listenin
	// kendi `max-block-size: 504px` sınırı taşmayı zaten sınırlıyor.
	let dy = 0;
	if (t.bottom > vh - MARGIN) dy = vh - MARGIN - t.bottom;
	if (t.top + dy < MARGIN) dy = MARGIN - t.top;

	if (dx !== 0 || dy !== 0) {
		wrapper.style.left = `${Math.round(a.left + dx)}px`;
		wrapper.style.top = `${Math.round(a.top + dy)}px`;
	}
}

/**
 * Katmanı referans kutusunun altına (yer yoksa üstüne) yerleştirip görünür
 * alana sıkıştırır.
 *
 * Yatay hizalama referansın SOL kenarından başlıyor; sığmıyorsa sağ kenara
 * hizalanıyor. Kütüphanenin `alignment` seçeneğini burada okumuyoruz çünkü
 * sonuç zaten görünür alana sıkıştırılıyor: kullanıcı açısından fark eden şey
 * katmanın tetikleyiciye yapışık ve tamamen görünür olması.
 *
 * Konum doğrudan katmanın satır içi stiline yazılıyor. Bu yol yalnızca kendi
 * `style` attribute'unu yeniden yazmayan katmanlar için geçerli — ComboBox
 * listesi bu yüzden `placeWrapper`'dan geçiyor (bkz. `COMBO_SELECTOR`).
 */
function placeNode({ node, reference }: Tracked) {
	const a = reference.getBoundingClientRect();
	const t = node.getBoundingClientRect();
	const vw = document.documentElement.clientWidth;
	const vh = document.documentElement.clientHeight;
	const gap = 4;

	const below = a.bottom + gap;
	const above = a.top - gap - t.height;
	// Aşağıda yer yoksa yukarı çevir — ama yukarısı daha da darsa aşağıda kal.
	const top = below + t.height > vh - MARGIN && above >= MARGIN ? above : below;

	let left = a.left;
	if (left + t.width > vw - MARGIN) left = a.right - t.width;

	node.style.left = `${Math.round(
		Math.min(Math.max(MARGIN, left), Math.max(MARGIN, vw - MARGIN - t.width))
	)}px`;
	node.style.top = `${Math.round(
		Math.min(Math.max(MARGIN, top), Math.max(MARGIN, vh - MARGIN - t.height))
	)}px`;
}
