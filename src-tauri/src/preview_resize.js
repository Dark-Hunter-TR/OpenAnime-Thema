// Önizleme webview'i yeniden boyutlandığında `webview.eval()` ile çalıştırılır
// (preview.rs -> soft_resize). Yer tutucu "__OA_REPAIR__" Rust tarafında
// true/false ile değiştirilir; sitenin 768px mobil eşiğinin geçilip
// geçilmediğini söyler.
//
// ## Eşik geçilmediyse: yalnızca haber ver
//
// Tablet <-> Masaüstü geçişi ya da splitter sürüklemesi sitenin mobil dalını
// değiştirmiyor. Webview gerçekten yeniden boyutlandığı için native `resize`
// zaten tetikleniyor; buradaki ek atış, WebView2'nin yeni bounds'u bir kare
// gecikmeyle uygulamasına karşı emniyet payı.
//
// ## Eşik geçildiyse: sayfayı yeniden yükle
//
// Sitenin masaüstü yerleşimi, açılışta kurulan bir yükseklik zincirine bağlı:
//
//     html[data-overlayscrollbars~=body] > body { height: 100% }
//       #page       { display:flex; height:100%; overflow:hidden }
//         #page > div { display:flex; height:100% }
//           .sidebar  { min-width:4.5rem; height:100% }
//
// Kaydırmanın sahibi OverlayScrollbars ve o, sayfa açılırken O ANKİ genişliğe
// göre kuruluyor. Mobil eşiği geçildiğinde site `body.mobile-patches`
// (`height:100vh; overflow-y:auto`) ile native kaydırmaya dönüyor; geri
// dönüldüğünde ise zincirin tepesi yeniden kurulmuyor. Body'nin kesin
// yüksekliği kalmayınca `#page{height:100%}` `auto`ya düşüyor, `.sidebar`
// belge boyu uzuyor ve `justify-content:space-between` yüzünden ortası
// bomboş kalıyor — ekranda "sol menü kayboldu" olarak görünen şey bu.
//
// ### Neden CSS ile onarmıyoruz
//
// Burada bir zamanlar `html,body{overflow-y:auto!important;height:auto!important}`
// enjekte ediliyordu. `height:auto` zinciri onarmıyor, KIRIYOR — hatanın bir
// kısmı doğrudan o satırdan geliyordu. Doğru yükseklikleri (`height:100%`)
// zorla geri yazmak ise kaydırma sahibini geri getirmiyor: OverlayScrollbars
// viewport elemanı yokken `#page{overflow:hidden}` ile sayfada kaydırılabilir
// hiçbir şey kalmıyor. Yani dışarıdan tek bir stille iki şeyi birden
// (yerleşim + kaydırma) tutarlı biçimde kurmanın yolu yok; sitenin kendi
// önyükleme zincirini çalıştırmak gerekiyor.
//
// Onu tetikleyebileceğimiz tek yol yeniden yükleme. Bedeli, yalnızca
// Mobil <-> (Tablet|Masaüstü) geçişinde görünen ~1 saniyelik splash ekranı;
// Tablet <-> Masaüstü geçişi eşiği geçmediği için kesintisiz kalıyor.
// `preview_init.js` her navigasyonda yeniden enjekte edildiğinden tema,
// oturum ve köprüler yeniden yüklemeden etkilenmiyor.
(function () {
	var CROSSED = "__OA_REPAIR__";

	/**
	 * Sitenin kendi duyarlı mekanizmaları:
	 *  - mobil algılama: `window.innerWidth < 768` + `resize` dinleyicisi
	 *    (bundle'da `Xt.set(window.innerWidth<768)`; `mobile-patches` gövde
	 *    sınıfı bu store'a abone)
	 *  - bazı bileşenler `matchMedia` kullanıyor; sorgular kendiliğinden
	 *    yeniden değerlendirilir, dinleyicilerin sıraya girmesi için kare
	 *    bırakıyoruz
	 */
	function fire() {
		window.dispatchEvent(new Event("resize"));
		window.dispatchEvent(new Event("orientationchange"));
	}

	// Her çalıştırmanın kendi kimliği. `window` üzerinde tutuluyor çünkü bu
	// betik her `eval` çağrısında SIFIRDAN bir IIFE olarak çalışıyor; yerel bir
	// değişken turlar arasında paylaşılmazdı.
	//
	// Amaç: art arda gelen boyut değişimlerinde (kullanıcı Mobil -> Masaüstü ->
	// Mobil diye hızlıca tıkladığında) yalnızca SONUNCU turun yeniden yükleme
	// yapması. Aradaki turlar kendiliğinden susuyor.
	var run = (window.__OA_RESIZE_RUN__ = (window.__OA_RESIZE_RUN__ || 0) + 1);

	function stale() {
		return run !== window.__OA_RESIZE_RUN__;
	}

	// WebView2 yeni bounds'u hemen uygulamıyor; iki kare bekleyip olayı
	// gönderiyoruz, 120ms'lik ikinci atış ağır sayfalar için emniyet payı.
	requestAnimationFrame(function () {
		requestAnimationFrame(function () {
			if (stale()) return;
			fire();
			setTimeout(function () {
				if (stale()) return;
				fire();
				if (!CROSSED) return;
				// Kısa bekleme, art arda tıklamaların tek bir yeniden yüklemede
				// birleşmesi için: bu sürede yeni bir tur başlarsa `stale()`
				// bunu iptal ediyor.
				setTimeout(function () {
					if (stale()) return;
					window.location.reload();
				}, 250);
			}, 120);
		});
	});
})();
