// Önizleme webview'i yeniden boyutlandığında `webview.eval()` ile çalıştırılır
// (preview.rs -> soft_resize). Yer tutucu "__OA_REPAIR__" Rust tarafında
// true/false ile değiştirilir.
//
// Amaç: görünüm (Mobil/Tablet/Masaüstü) değişiminde TAM SAYFA YENİDEN YÜKLEME
// yapmadan sitenin kendi duyarlı mekanizmalarını tetiklemek.
(function () {
	// Eşik (768px) geçildi mi? Yalnızca o durumda kaydırma onarımı denenir.
	var REPAIR = "__OA_REPAIR__";
	var STYLE_ID = "__oa_scroll_repair__";

	/**
	 * Sitenin kendi mekanizmaları:
	 *  - mobil algılama: `window.innerWidth < 768` + `resize` dinleyicisi
	 *  - bazı bileşenler `matchMedia` kullanıyor; sorgular otomatik yeniden
	 *    değerlendirilir, dinleyicilerin sıraya girmesi için kare bırakıyoruz
	 */
	function fire() {
		window.dispatchEvent(new Event("resize"));
		window.dispatchEvent(new Event("orientationchange"));
	}

	function repairStyle() {
		return document.getElementById(STYLE_ID);
	}

	function setRepair(on) {
		var el = repairStyle();
		if (!on) {
			if (el) el.remove();
			return;
		}
		if (!el) {
			el = document.createElement("style");
			el.id = STYLE_ID;
			(document.head || document.documentElement).appendChild(el);
		}
		// Site kaydırmayı OverlayScrollbars'a devrettiği için html/body'de
		// `overflow: hidden`. O devir kurulamadıysa native kaydırmayı geri
		// açıyoruz — sayfanın kilitli kalmasındansa sade bir çubuk yeğdir.
		el.textContent = "html,body{overflow-y:auto!important;height:auto!important;}";
	}

	/** Sayfada gerçekten kaydırılabilen bir kap var mı? */
	function isScrollable() {
		var vp = document.querySelector("[data-overlayscrollbars-viewport]");
		if (vp && vp.scrollHeight > vp.clientHeight + 4) return true;

		var doc = document.documentElement;
		if (
			doc.scrollHeight > window.innerHeight + 4 &&
			getComputedStyle(doc).overflowY !== "hidden"
		) {
			return true;
		}

		var nodes = document.querySelectorAll("body *");
		for (var i = 0; i < nodes.length; i++) {
			var el = nodes[i];
			if (el.scrollHeight > el.clientHeight + 4) {
				var oy = getComputedStyle(el).overflowY;
				if (oy === "auto" || oy === "scroll") return true;
			}
		}
		return false;
	}

	function check() {
		var body = document.body;
		if (!body) return;

		// ÖNCE kendi onarımımızı kaldır, SONRA ölç. Aksi hâlde bir önceki
		// geçişten kalan stil "sayfa kaydırılabiliyor" gibi görünmeye yol
		// açar ve onarım kalıcı olarak takılı kalırdı.
		var had = !!repairStyle();
		setRepair(false);

		var contentTall = body.scrollHeight > window.innerHeight + 4;
		var broken = contentTall && !isScrollable();
		setRepair(broken);

		// Site kendi kaydırmasını geri kurduysa sessizce normale dönüyoruz.
		if (had && !broken) fire();
	}

	// WebView2 yeni bounds'u hemen uygulamıyor; iki kare bekleyip olayı
	// gönderiyoruz, 120ms'lik ikinci atış ağır sayfalar için emniyet payı.
	requestAnimationFrame(function () {
		requestAnimationFrame(function () {
			fire();
			setTimeout(function () {
				fire();
				if (!REPAIR) return;
				// Sitenin kendi yeniden kurulumuna zaman tanı, sonra ölç.
				setTimeout(check, 180);
			}, 120);
		});
	});
})();
