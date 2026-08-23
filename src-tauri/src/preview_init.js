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
 * kendi kodundan önce çalıştığı için `I()` ilk kontrolünde geçiyor.
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

/**
 * Tema enjeksiyonu.
 *
 * Editörün CSS'i KENDİ `<style>` etiketimize yazılır ve sitenin özel tema
 * sistemine (`localStorage.theme_content`) HİÇ dokunulmaz. İkisi de bilinçli
 * ve ikisi de bir hatanın karşılığı:
 *
 *   1. Önceden CSS `localStorage.theme_content`'e yazılıyordu — yani editörün
 *      çıktısı sitenin kendi "Özel Tema" yuvasına KURULUYORDU. Site açılışta o
 *      yuvayı okuyup içeriği kendi `<style>` etiketine basıyor ve o etiket
 *      bizimkinden SONRA geliyor. Sonuç: önizlemede görünen CSS artık bizim
 *      değil sitenin elindeki kopyaydı. İçe aktarılan tema göründüğü hâlde
 *      kullanıcının sonraki değişiklikleri hiç yansımıyordu — biz kendi
 *      etiketimizi güncellerken ekranı sitenin kopyası boyuyordu.
 *
 *   2. Aynı sebeple `style[themeStyle]` de kullanılmıyor: o etiket SİTENİN.
 *      Paylaşınca sahibi belirsizleşiyor ve site istediği anda üzerine
 *      yazabiliyor.
 *
 * Önizleme zaten gerçek zamanlı: her değişiklikte Rust tarafı bu fonksiyonu
 * çağırıyor. Siteye bir şey "kurmaya" gerek yok — kurulduğunda düzenlenebilir
 * de olmuyor, çünkü artık sitenin verisi.
 */
(function () {
	var INITIAL_CSS = "__OA_INITIAL_CSS__";
	var INITIAL_MODE = "__OA_INITIAL_MODE__";

	/** Bizim etiketimizin işareti — sitenin `themeStyle`'ı ile karışmasın. */
	var MARK = "data-oa-preview";

	function modeToSiteValue(mode) {
		return mode === "light" ? "1" : mode === "dark" ? "2" : "0";
	}

	// Sitenin özel tema yuvasını boşalt.
	//
	// Yalnızca "yazmamak" yetmiyor: önceki sürümler oraya yazdığı için yuvada
	// eski bir tema kalmış olabilir ve site onu açılışta yükleyip bizimkinin
	// üstüne biner. Bu betik sitenin kendi kodundan ÖNCE çalıştığı için burada
	// silmek, sitenin o veriyi hiç görmemesini sağlıyor.
	try {
		localStorage.removeItem("theme_content");
		localStorage.setItem("theme", modeToSiteValue(INITIAL_MODE));
	} catch (e) {}

	var currentCss = INITIAL_CSS;
	var currentMode = INITIAL_MODE;

	/**
	 * Bizim stil etiketimizi döndürür ve `<head>`in SONUNDA tutar.
	 *
	 * Sıra önemli: eşit özgüllükte sonra gelen kural kazanıyor. Site kendi
	 * stillerini sonradan ekleyebildiği için, etiketimiz sona düşmediyse geri
	 * taşınıyor. Zaten sondaysa dokunulmuyor — bir `<style>`ı yeniden eklemek
	 * stil sayfasını söküp yeniden kurmak demek ve bu, kaydırıcı sürüklenirken
	 * gözle görülür bir titremeye yol açardı.
	 */
	function styleElement() {
		var el = document.querySelector("style[" + MARK + "]");
		if (!el) {
			el = document.createElement("style");
			el.setAttribute(MARK, "");
			el.type = "text/css";
		}
		var parent = document.head || document.documentElement;
		if (el.parentNode !== parent || parent.lastElementChild !== el) {
			parent.appendChild(el);
		}
		return el;
	}

	window.__OA_THEME_APPLY__ = function (css) {
		currentCss = css;
		var el = styleElement();
		if (el.textContent !== css) el.textContent = css;
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

	/**
	 * Giriş köprüsü: `POST /user/auth`'u SAYFANIN KENDİSİNDEN çağırır.
	 *
	 * Sözleşme sitenin kendi giriş modalından birebir alındı:
	 *
	 *   POST {API_BASE}/user/auth   {"email": ..., "password": ...}
	 *     -> { token, refreshToken, verified, ... }   (başarı)
	 *     -> { error: "<İngilizce mesaj>" }           (hata)
	 *
	 * Başarıdan sonra site erişim bilgilerini ÇEREZE kendisi yazıyor; sunucu
	 * `Set-Cookie` döndürmüyor. Bu yüzden aynı iki çağrıyı biz de yapmak
	 * zorundayız, aksi hâlde istek başarılı olur ama oturum hiçbir yerde
	 * açılmazdı. `window.setCookie` sayfanın kendi global yardımcısı (app.html)
	 * ve alan adını doğru hesaplıyor; `document.cookie`'yi elle yazmak
	 * `.openani.me` yerine yalnızca `openani.me`ye yazardı.
	 *
	 * Süreler de siteyle aynı: token 7 gün, refreshToken 90 gün.
	 *
	 * DİKKAT: `token`/`refreshToken` yanıtta Rust'a GÖNDERİLMİYOR. Uygulamanın
	 * onlara ihtiyacı yok — oturumun açıldığının kanıtı çerezin kendisi ve onu
	 * `preview_login_state` zaten çerez kavanozundan okuyor. IPC'ye koymak
	 * kimlik bilgisini gereksiz yere bir katman daha dolaştırmak olurdu.
	 */
	window.__OA_API_LOGIN__ = function (requestId, email, password) {
		function reply(payload) {
			payload.id = requestId;
			try {
				window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {
					event: EVENT,
					payload: payload
				});
			} catch (e) {}
		}

		function postLogin() {
			// `window.fetch` — sitenin YAMALI olanı; Gateway-Token'ı o ekliyor.
			return window
				.fetch(API_BASE + "/user/auth", {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ email: email, password: password })
				})
				.then(function (r) {
					return r
						.json()
						.catch(function () {
							return null;
						})
						.then(function (data) {
							return { status: r.status, data: data };
						});
				});
		}

		/**
		 * Yanıt, kimlik bilgisinin değil GEÇİDİN reddi mi?
		 *
		 * Ayrım kritik: yanlış parolayı tekrar denemenin anlamı yok, bayat bir
		 * geçit oturumunu tekrar denemenin ise var. Durum kodu tek başına
		 * ayırt etmiyor — sitenin kendi modali da durum koduna değil gövdeye
		 * bakıyor: yanlış parola 200 + {"error":"Invalid password"} olarak
		 * geliyor, Vanguard reddi ise 401/400 + {"code":"error.unauthorized"}.
		 */
		function gatewayRejected(res) {
			if (!res) return false;
			// SIRA ÖNEMLİ: Vanguard'ın reddi
			// {"code":"error.unauthorized","error":"Unauthorized","message":"...Vanguard."}
			// yani `error` alanı ONDA DA dolu. Önce `error`a bakılırsa geçit reddi
			// "parola hatası" sanılır ve kurtarma hiç çalışmaz. Ayırt edici olan
			// `code`. (Doğrudan curl ile doğrulandı.)
			if (res.data && res.data.code === "error.unauthorized") return true;
			if (res.data && res.data.error) return false;
			return res.status === 401 || res.status === 400;
		}

		function finish(res) {
			// Sunucu hatayı 200 gövdesinde de döndürebiliyor.
			if (res.data && res.data.error) {
				reply({ stage: "done", ok: false, message: String(res.data.error) });
				return;
			}
			if (!res.data || !res.data.token) {
				reply({ stage: "done", ok: false, message: "sunucu " + res.status + " döndürdü" });
				return;
			}

			try {
				window.setCookie("token", res.data.token, "7");
				if (res.data.refreshToken) {
					window.setCookie("refreshToken", res.data.refreshToken, "90");
				}
			} catch (e) {
				reply({ stage: "done", ok: false, message: "oturum çerezi yazılamadı" });
				return;
			}

			// `verified` false ise hesap açıldı ama e-posta doğrulanmamış.
			// Sitenin akışı burada doğrulama adımına geçiyor; bizde böyle bir
			// ekran yok, durumu olduğu gibi bildirip kullanıcıyı siteye
			// yönlendirmek arayüzün işi.
			reply({ stage: "done", ok: true, verified: res.data.verified !== false });
		}

		waitForGateway(40) // ~10 sn
			.then(function (ready) {
				if (!ready) {
					reply({ stage: "no-gateway" });
					return null;
				}
				return postLogin();
			})
			.then(function (res) {
				if (!res) return null;
				if (!gatewayRejected(res)) {
					finish(res);
					return null;
				}

				// Buraya düşmenin en sık sebebi: giriş diyaloğu açıkken önizleme
				// GİZLİ oluyor (bkz. `+page.svelte` -> `modalOpen`) ve Chromium'un
				// arka plan zamanlayıcı kısıtlaması sitenin 35 sn'lik yeniden
				// imzalama döngüsünü durduruyor; elimizdeki geçit oturumu
				// bayatlıyor. `__OA_API_FETCH__` ile aynı kurtarma: önce kısa bir
				// bekleyip tekrar dene.
				return new Promise(function (resolve) {
					setTimeout(resolve, 2500);
				})
					.then(postLogin)
					.then(function (retry) {
						if (!gatewayRejected(retry)) {
							finish(retry);
							return null;
						}
						// İki deneme de aynı bayat oturumu kullandı. Yeni bir
						// sessionId almanın tek yolu sitenin önyükleme zinciri;
						// onu ancak sayfayı yenileyerek tetikleyebiliyoruz.
						// Parola burada YENİDEN GÖNDERİLMİYOR — kullanıcı sayfa
						// toparlandıktan sonra tekrar dener.
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

	/**
	 * Çıkış köprüsü: oturumu kapatır.
	 *
	 * Sitenin kendi `logout()` fonksiyonunun aynısı:
	 *
	 *   fetch(`${API}/user/logout`, {method:"POST",
	 *         body: JSON.stringify({refreshToken: getCookie("refreshToken")})})
	 *   setCookie("token", "", -1); setCookie("refreshToken", "", -1)
	 *
	 * ## Sıra neden böyle
	 *
	 * Sitede de o `fetch` BEKLENMİYOR (await yok) — çıkışın kullanıcı açısından
	 * gerçekleşmesi çerezlerin silinmesine bağlı, sunucu isteğine değil. Aynı
	 * mantığı koruyoruz: önce çerezleri siliyoruz ve yanıtı hemen döndürüyoruz,
	 * `POST /user/logout` arkada en iyi çaba olarak gidiyor. Böylece geçit
	 * bayatsa bile çıkış YİNE de çalışıyor — tersi olsaydı, oturumu kapatmak
	 * isteyen kullanıcı geçidin toparlanmasını beklemek zorunda kalırdı.
	 *
	 * `refreshToken` çerez silinmeden önce okunuyor; sunucunun onu iptal
	 * edebilmesi için değere ihtiyacı var.
	 *
	 * Sonunda sayfa yenileniyor: çerezler gitse de sitenin kendi bellekteki
	 * kullanıcı store'u hâlâ dolu olurdu ve önizleme kullanıcıyı giriş yapmış
	 * göstermeye devam ederdi. Sitenin `logout()`'u da aynı sebeple ana sayfaya
	 * yönlendiriyor.
	 */
	/**
	 * QR (DAG) ile giriş köprüsü.
	 *
	 * Sitenin akışı: `GET {API_BASE}/dag` bir SSE akışı döndürüyor ve üç tür
	 * olay geliyor —
	 *
	 *   {"type":"qr",      "data": "<görsel kaynağı>"}   -> ekranda göster
	 *   {"type":"success", "data": {token, refreshToken, verified}}
	 *   {"type":"error",   "data"|"error": "<mesaj>"}
	 *
	 * QR kod kısa aralıklarla yenileniyor, yani `qr` olayı birden çok kez
	 * geliyor. Başarıda çerezleri sitenin yaptığı gibi biz yazıyoruz.
	 *
	 * ## Neden kuyruk + yoklama
	 *
	 * Köprünün geri kalanı tek istek/tek yanıt. Bu akış ise SÜREKLİ; üstelik
	 * kullanıcının telefonuyla kodu okutması dakikalar sürebilir. Olayları
	 * kuyruğa alıp `__OA_API_DAG_NEXT__` ile teker teker teslim ediyoruz:
	 * arayüz bir döngüde çağırıyor, kuyruk boşsa ~25 sn yoklanıp `idle`
	 * dönülüyor ve arayüz yeniden çağırıyor.
	 *
	 * `idle` yolu şart: Rust tarafındaki bekleme zaman aşımına uğrasaydı,
	 * o sırada gelen bir olay kimsenin dinlemediği bir istek kimliğine
	 * gönderilip KAYBOLURDU. Kuyrukta beklettiğimiz için hiçbir olay düşmüyor.
	 */
	var dag = null;

	function dagPush(msg) {
		if (dag) dag.queue.push(msg);
	}

	function dagStop() {
		if (!dag) return;
		try {
			dag.controller.abort();
		} catch (e) {}
		dag = null;
	}

	/** Küçük bir SSE ayrıştırıcısı — yalnızca `data:` satırlarını topluyor. */
	function sseParser(onData) {
		var buf = "";
		var SEP = "\n\n";
		return function feed(chunk) {
			// Satır sonlarını tek biçime indiriyoruz; SSE hem CRLF hem LF
			// kullanabiliyor ve ayırıcıyı iki kez aramak gereksiz.
			buf = (buf + chunk).split("\r\n").join("\n").split("\r").join("\n");
			for (;;) {
				var idx = buf.indexOf(SEP);
				if (idx === -1) return;
				var raw = buf.slice(0, idx);
				buf = buf.slice(idx + SEP.length);
				var data = [];
				raw.split("\n").forEach(function (line) {
					if (line.indexOf("data:") === 0) data.push(line.slice(5).replace(/^ /, ""));
				});
				if (data.length) onData(data.join("\n"));
			}
		};
	}

	function dagComplete(payload) {
		// Sitedeki `dh()` ile aynı: gövde dize gelirse JSON dene, olmazsa
		// tamamını token say.
		var d = payload;
		if (typeof d === "string") {
			try {
				d = JSON.parse(d);
			} catch (e) {
				d = { token: payload };
			}
		}
		if (d && d.error) {
			dagPush({ kind: "error", message: String(d.error) });
			return;
		}
		if (!d || !d.token) {
			dagPush({ kind: "error", message: "QR kod ile giriş tamamlanamadı." });
			return;
		}
		try {
			window.setCookie("token", d.token, "7");
			if (d.refreshToken) window.setCookie("refreshToken", d.refreshToken, "90");
		} catch (e) {
			dagPush({ kind: "error", message: "oturum çerezi yazılamadı" });
			return;
		}
		dagPush({ kind: "success", verified: d.verified !== false });
	}

	function dagHandle(session, text) {
		if (session !== dag) return;
		var msg;
		try {
			msg = JSON.parse(text);
		} catch (e) {
			dagPush({ kind: "error", message: "QR kod verisi okunamadı." });
			return;
		}
		if (msg.type === "qr") {
			dagPush({ kind: "qr", image: String(msg.data || "") });
		} else if (msg.type === "error") {
			dagPush({ kind: "error", message: String(msg.data || msg.error || "Bilinmeyen hata") });
		} else if (msg.type === "success") {
			dagComplete(msg.data);
		}
	}

	function dagStart() {
		var session = { controller: new AbortController(), queue: [] };
		dag = session;

		waitForGateway(40) // ~10 sn
			.then(function (ready) {
				if (session !== dag) return null;
				if (!ready) {
					dagPush({ kind: "error", message: "QR kod oluşturulamadı." });
					return null;
				}
				// `window.fetch` — sitenin YAMALI olanı; Gateway-Token'ı o ekliyor.
				return window.fetch(API_BASE + "/dag", { signal: session.controller.signal });
			})
			.then(function (r) {
				if (!r) return null;
				if (session !== dag) return null;
				if (!r.ok || !r.body) {
					dagPush({ kind: "error", message: "QR kod oluşturulamadı." });
					return null;
				}
				var reader = r.body.getReader();
				var decoder = new TextDecoder();
				var feed = sseParser(function (text) {
					dagHandle(session, text);
				});
				function pump() {
					return reader.read().then(function (res) {
						if (session !== dag) return null;
						if (res.done) {
							feed(decoder.decode());
							return null;
						}
						feed(decoder.decode(res.value, { stream: true }));
						return pump();
					});
				}
				return pump();
			})
			.catch(function (e) {
				// Akış bilerek iptal edildiyse (diyalog kapandı) hata değil.
				if (session !== dag || (e && e.name === "AbortError")) return;
				dagPush({ kind: "error", message: "QR kod akışı başlatılamadı." });
			});

		return session;
	}

	window.__OA_API_DAG_NEXT__ = function (requestId) {
		function reply(payload) {
			payload.id = requestId;
			payload.stage = "done";
			try {
				window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {
					event: EVENT,
					payload: payload
				});
			} catch (e) {}
		}

		if (!dag) dagStart();
		var session = dag;

		var waited = 0;
		(function poll() {
			// Oturum arada durdurulduysa (diyalog kapandı) sessizce bitir.
			if (session !== dag) {
				reply({ kind: "idle" });
				return;
			}
			if (session.queue.length) {
				reply(session.queue.shift());
				return;
			}
			if (waited >= 25000) {
				// Rust tarafındaki bekleme 30 sn; ondan önce yanıt vermeliyiz ki
				// arayüz zaman aşımı hatası görmesin, sadece döngüyü sürdürsün.
				reply({ kind: "idle" });
				return;
			}
			waited += 250;
			setTimeout(poll, 250);
		})();
	};

	window.__OA_API_DAG_STOP__ = function () {
		dagStop();
	};

	window.__OA_API_LOGOUT__ = function (requestId) {
		function reply(payload) {
			payload.id = requestId;
			try {
				window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {
					event: EVENT,
					payload: payload
				});
			} catch (e) {}
		}

		var refreshToken = cookie("refreshToken");

		try {
			window.setCookie("token", "", -1);
			window.setCookie("refreshToken", "", -1);
		} catch (e) {
			reply({ stage: "done", ok: false, message: "oturum çerezi silinemedi" });
			return;
		}

		reply({ stage: "done", ok: true });

		function finish() {
			setTimeout(function () {
				window.location.reload();
			}, 50);
		}

		if (!refreshToken) {
			finish();
			return;
		}

		// Kısa bir geçit beklemesi (~2 sn): giriş yolundaki ~10 sn burada fazla
		// olurdu, çünkü bu isteğin başarısı çıkışın gerçekleşmesi için şart değil.
		waitForGateway(8)
			.then(function (ready) {
				if (!ready) return null;
				return window.fetch(API_BASE + "/user/logout", {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({ refreshToken: refreshToken })
				});
			})
			.catch(function () {
				// Sunucu iptali başarısız oldu; yerel oturum yine de kapandı.
			})
			.then(finish);
	};
})();
