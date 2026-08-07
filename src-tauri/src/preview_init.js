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
