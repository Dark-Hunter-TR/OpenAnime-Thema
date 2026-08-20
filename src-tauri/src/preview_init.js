/**
 * Sayfayı openani.me'ye HER ZAMAN "görünür" göster.
 *
 * Bu, hesap bilgisinin editöre girmeden gelmesini sağlayan asıl düzeltme.
 * Sitenin önyükleme zinciri şöyle başlıyor (herkese açık bundle):
 *
 *     function F(){ const p = document.prerendering === true;
 *                   return document.visibilityState === "visible" && !p; }
 *     async function I(){ F() || await new Promise(resolve => {
 *         // visibilitychange / prerenderingchange / pageshow + 50ms'lik interval
 *     }); }
 *     // ... await I();  →  await ie();  →  if (w) await N();   // ← geçit kurulumu
 *
 * Yani `/osc.wasm` indirilmesi, `POST /session/init` ve `gateway-token`
 * üretimi — hepsi sayfa GÖRÜNÜR olana kadar bekliyor. Önizleme webview'i
 * uygulama açılışında `hide()` edildiği için `visibilityState` "hidden"
 * kalıyor, `I()` hiç çözülmüyor ve geçit hiç kurulmuyor. Kullanıcı editörü
 * açtığı anda webview görünür oluyor, zincir çözülüyor ve hesap bilgisi
 * ancak o zaman gelebiliyor. (Ölçüldü: gizliyken `ready:"complete"`,
 * `tok:true`, ama `gw:false`.)
 *
 * Bu yüzden görünürlüğü sayfaya karşı sabitliyoruz. Webview YİNE de native
 * olarak gizli — yalnızca sayfanın kendi kapısı açılıyor. Betik sitenin
 * kendi kodundan ÖNCE çalıştığı için `I()` ilk kontrolünde geçiyor.
 *
 * Yan etkisi bilinçli: sayfa artık kendini hiç arka planda saymıyor, yani
 * sitenin 8 dakikalık oturum tazeleme mutex'i ve 35 saniyelik geçit
 * imzalaması gizliyken de çalışmaya devam ediyor — zaten istediğimiz şey.
 */
(function () {
	function force(name, value) {
		try {
			Object.defineProperty(document, name, {
				configurable: true,
				get: function () {
					return value;
				}
			});
		} catch (e) {}
	}
	force("visibilityState", "visible");
	force("hidden", false);
})();

(function () {
	var INITIAL_CSS = "__OA_INITIAL_CSS__";
	var INITIAL_MODE = "__OA_INITIAL_MODE__";

	function modeToSiteValue(mode) {
		return mode === "light" ? "1" : mode === "dark" ? "2" : "0";
	}
	try {
		localStorage.setItem("theme_content", INITIAL_CSS);
		localStorage.setItem("theme", modeToSiteValue(INITIAL_MODE));
	} catch (e) {}
	var currentCss = INITIAL_CSS;
	var currentMode = INITIAL_MODE;
	window.__OA_THEME_APPLY__ = function (css) {
		currentCss = css;
		var el = document.querySelector("style[themeStyle]");
		if (!el) {
			el = document.createElement("style");
			el.setAttribute("themeStyle", "true");
			el.type = "text/css";
			(document.head || document.documentElement).appendChild(el);
		}
		if (el.textContent !== css) el.textContent = css;
		try {
			localStorage.setItem("theme_content", css);
		} catch (e) {}
		return true;
	};
	window.__OA_THEME_MODE__ = function (mode) {
		currentMode = mode;
		var root = document.documentElement;
		root.classList.remove("fds-theme-light", "fds-theme-dark");
		if (mode === "light") root.classList.add("fds-theme-light");
		else if (mode === "dark") root.classList.add("fds-theme-dark");
		try {
			localStorage.setItem("theme", modeToSiteValue(mode));
		} catch (e) {}
		return true;
	};
	function resync() {
		if (currentCss) window.__OA_THEME_APPLY__(currentCss);
		window.__OA_THEME_MODE__(currentMode);
	}

	if (document.readyState === "loading") {
		document.addEventListener("DOMContentLoaded", resync, { once: true });
	} else {
		resync();
	}
})();

/**
 * Hesap köprüsü: openani.me'nin `/user` uç noktasını SAYFANIN KENDİSİNDEN çağırır.
 *
 * Neden Rust'tan (reqwest ile) değil de buradan?
 *
 * `api.openani.me` "Vanguard" adlı bir geçidin arkasında. Kimlik doğrulama
 * GEREKTİRMEYEN uç noktalar bile `Gateway-Token` başlığı olmadan 401 +
 * `{"code":"error.unauthorized", "message":"Unauthorized access is denied by
 * OpenAnime Vanguard."}` döndürüyor. Yani `Authorization` doğru olsa bile
 * uygulama dışarıdan istek atamıyor.
 *
 * O başlığı sitenin app.html'indeki satır içi script ekliyor: `window.fetch`'i
 * sarmalayıp aynı ana alan adına giden her isteğe
 * `Gateway-Token: sessionStorage['gateway-token'] || getCookie('gateway-token')`
 * koyuyor. Başlığın değeri ise `/osc.wasm` (TinyGo, HMAC-SHA256) tarafından
 * imzalanıyor; sayfa 35 saniyede bir yeniden imzalıyor, 5 dakikada bir de
 * `POST /session/init` ile yeni bir sessionId alıyor. Modülü tutan
 * `window.ActionManager` referansı yakalandıktan hemen sonra siliniyor.
 *
 * Sonuç: token kopyalanıp Rust'ta saklanamaz (35 sn'de bayatlar) ve imza
 * yeniden üretilemez. Tek doğru yol isteği sayfanın yamalı `fetch`'iyle
 * atmak — bunun için de openani.me'nin render ediliyor olması gerekiyor.
 *
 * Erişim token'ına da elimizle dokunmuyoruz: `POST /user/refresh` yeni bir
 * `refreshToken` DÖNDÜRÜYOR (rotasyon) ve sayfa bunu
 * `navigator.locks.request("openanime_token_refresh_lock")` kilidi + 5
 * dakikalık `oa_last_refresh_timestamp` penceresiyle koruyor. Paralel
 * tazeleme yapmak oturumu düşürebilirdi; sadece `window.getCookie("token")`
 * ile o an geçerli olanı okuyoruz.
 */
(function () {
	var API_BASE = "__OA_API_BASE__";
	var EVENT = "oa://account-result";

	function cookie(name) {
		try {
			return (window.getCookie && window.getCookie(name)) || "";
		} catch (e) {
			return "";
		}
	}

	function gatewayToken() {
		try {
			var stored = sessionStorage.getItem("gateway-token");
			if (stored) return stored;
		} catch (e) {}
		return cookie("gateway-token");
	}

	// Geçit henüz kurulmamış olabilir: wasm indirilip çalıştırılana ve ilk
	// `/session/init` dönene kadar bir şey yok. Sabit bir bekleme yerine
	// yokluyoruz — sayfa yeni açıldıysa bu birkaç saniye sürebiliyor.
	function waitForGateway(triesLeft) {
		if (gatewayToken()) return Promise.resolve(true);
		if (triesLeft <= 0) return Promise.resolve(false);
		return new Promise(function (resolve) {
			setTimeout(function () {
				resolve(waitForGateway(triesLeft - 1));
			}, 250);
		});
	}

	function requestPath(token, path) {
		// DİKKAT: `window.fetch` — sitenin YAMALI olanı. Yerel bir kopyaya
		// (ör. yukarıda saklanmış bir referansa) düşersek Gateway-Token
		// eklenmez ve istek 401 olur.
		return window.fetch(API_BASE + path, { headers: { Authorization: token } }).then(function (r) {
			return r.text().then(function (body) {
				return { status: r.status, body: body };
			});
		});
	}

	// `path` her zaman Rust tarafından üretiliyor ve orada doğrulanıyor
	// (bkz. `lib.rs` -> `bridge_get` / `fetch_account_follows`); burası onu
	// serbest bir URL gibi ele almamalı.
	window.__OA_API_FETCH__ = function (requestId, path) {
		function reply(payload) {
			payload.id = requestId;
			try {
				window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {
					event: EVENT,
					payload: payload
				});
			} catch (e) {
				// Köprü yoksa (capability eksik/başka bir origin) yapabileceğimiz
				// bir şey yok; Rust tarafı zaman aşımına düşer.
			}
		}

		var token = cookie("token");
		if (!token) {
			reply({ stage: "no-session" });
			return;
		}

		// 401 VE 400 ikisi de "tazelenebilir" sayılıyor: uygulama arka plana
		// alınıp geri getirildiğinde Chromium'un arka plan zamanlayıcı
		// kısıtlaması (background timer throttling) sitenin kendi 35sn'lik
		// yeniden imzalama ve 5dk'lık `POST /session/init` döngülerini
		// geciktiriyor/durduruyor; öne geldiğimizde geçit oturumu bayatlamış
		// olabiliyor. Bayat bir oturumla yapılan istek her zaman 401 dönmüyor
		// — Vanguard bazen 400 ("Bad Request") ile de reddediyor, gözlemlendi.
		function recoverable(status) {
			return status === 401 || status === 400;
		}

		waitForGateway(40) // ~10 sn
			.then(function (ready) {
				if (!ready) {
					reply({ stage: "no-gateway" });
					return null;
				}
				return requestPath(token, path);
			})
			.then(function (res) {
				if (!res || !recoverable(res.status)) {
					if (res) reply({ stage: "done", status: res.status, body: res.body });
					return null;
				}
				// İlk yeniden deneme: geçit token'ının tam yenilendiği ana denk
				// gelmiş olmak ya da sitenin auth mutex'inin erişim token'ını o
				// sırada tazeliyor olması gibi saniyeler içinde geçen, masum
				// sebepler için.
				return new Promise(function (resolve) {
					setTimeout(resolve, 2500);
				})
					.then(function () {
						return requestPath(cookie("token") || token, path);
					})
					.then(function (retry) {
						if (!recoverable(retry.status)) {
							reply({ stage: "done", status: retry.status, body: retry.body });
							return null;
						}
						// İki deneme de aynı bayat oturumu kullanmış olabilir —
						// `V()` (sitenin 35sn'lik döngüsü) yalnızca AYNI sessionId'yi
						// yeniden imzalıyor, süresi dolmuş bir oturumu diriltmiyor.
						// Yeni bir sessionId almanın (`POST /session/init`) tek yolu
						// sitenin kendi önyükleme zinciri — biz o zinciri elle
						// tetikleyemiyoruz (imzalayıcı WASM'a referansımız yok).
						// Son çare: sayfayı yeniliyoruz, bu zinciri baştan
						// çalıştırıyor. `preview_init.js` her navigasyonda yeniden
						// enjekte edildiği için köprü de kendini toparlıyor.
						reply({ stage: "reloading" });
						setTimeout(function () {
							window.location.reload();
						}, 50);
						return null;
					});
			})
			.catch(function (e) {
				reply({ stage: "error", message: String(e) });
			});
	};
})();
