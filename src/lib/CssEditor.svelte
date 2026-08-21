<script lang="ts">
	import { createEventDispatcher, onDestroy, onMount } from "svelte";
	import { EditorState } from "@codemirror/state";
	import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
	import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
	import { bracketMatching, syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
	import { css as cssLang } from "@codemirror/lang-css";
	import {
		autocompletion,
		completionKeymap,
		type Completion,
		type CompletionContext
	} from "@codemirror/autocomplete";

	import { FDS_TOKENS, SITE_CLASSES, SITE_IDS, SITE_VARS } from "$lib/catalog.generated";

	export let value = "";

	const dispatch = createEventDispatcher<{ change: string }>();

	let host: HTMLDivElement;
	let view: EditorView | undefined;

	// --- Otomatik tamamlama kaynakları --------------------------------------
	// Hepsi catalog.generated.ts'ten geliyor; o da fluent-svelte-extra'nın
	// gerçek CSS'i ve openani.me'nin canlı bundle'larından üretiliyor.

	const tokenOptions: Completion[] = FDS_TOKENS.map((t) => ({
		label: t.name,
		type: "variable",
		detail: t.group,
		info: [
			t.root ? `varsayılan: ${t.root}` : null,
			t.light ? `açık: ${t.light}` : null,
			t.dark ? `koyu: ${t.dark}` : null
		]
			.filter(Boolean)
			.join("\n")
	}));

	const siteVarOptions: Completion[] = SITE_VARS.map((name) => ({
		label: name,
		type: "variable",
		detail: "site (Stable API değil)"
	}));

	const varOptions = [...tokenOptions, ...siteVarOptions];

	const classOptions: Completion[] = SITE_CLASSES.map((name) => ({
		label: name,
		type: "class",
		detail: "site class'ı"
	}));

	const idOptions: Completion[] = SITE_IDS.map((name) => ({
		label: name,
		type: "class",
		detail: "site id'si"
	}));

	function completeOpenAnime(ctx: CompletionContext) {
		const variable = ctx.matchBefore(/--[\w-]*/);
		if (variable) return { from: variable.from, options: varOptions, validFor: /^--[\w-]*$/ };

		const cls = ctx.matchBefore(/\.[\w-]*/);
		if (cls) return { from: cls.from + 1, options: classOptions, validFor: /^[\w-]*$/ };

		const id = ctx.matchBefore(/#[\w-]*/);
		if (id) return { from: id.from + 1, options: idOptions, validFor: /^[\w-]*$/ };

		return null;
	}

	// --- Görünüm -------------------------------------------------------------
	// Kısıt gereği ayrı bir CSS dosyası yazmıyoruz: CodeMirror'ın kendi tema
	// API'si üzerinden doğrudan --fds-* token'larına bağlanıyoruz. Böylece
	// editör, kullanıcının o an düzenlediği temaya göre renk değiştiriyor.
	const fluentTheme = EditorView.theme({
		"&": {
			height: "100%",
			fontSize: "var(--fds-caption-font-size)",
			color: "var(--fds-text-primary)",
			backgroundColor: "var(--fds-control-fill-default)",
			border: "1px solid var(--fds-control-stroke-default)",
			borderRadius: "var(--fds-control-corner-radius)"
		},
		"&.cm-focused": { outline: "none", borderColor: "var(--fds-accent-default)" },
		".cm-scroller": { fontFamily: "Consolas, 'Cascadia Mono', ui-monospace, monospace" },
		".cm-gutters": {
			backgroundColor: "transparent",
			color: "var(--fds-text-tertiary)",
			border: "none"
		},
		".cm-activeLine": { backgroundColor: "var(--fds-subtle-fill-secondary)" },
		".cm-cursor": { borderLeftColor: "var(--fds-text-primary)" },
		".cm-selectionBackground, ::selection": { backgroundColor: "var(--fds-accent-tertiary)" },
		".cm-tooltip": {
			backgroundColor: "var(--fds-solid-background-quarternary)",
			border: "1px solid var(--fds-surface-stroke-flyout)",
			borderRadius: "var(--fds-overlay-corner-radius)",
			color: "var(--fds-text-primary)"
		},
		".cm-tooltip-autocomplete > ul > li[aria-selected]": {
			backgroundColor: "var(--fds-subtle-fill-secondary)",
			color: "var(--fds-text-primary)"
		}
	});

	onMount(() => {
		view = new EditorView({
			parent: host,
			state: EditorState.create({
				doc: value,
				extensions: [
					lineNumbers(),
					history(),
					bracketMatching(),
					highlightActiveLine(),
					syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
					cssLang(),
					autocompletion({ override: [completeOpenAnime], activateOnTyping: true }),
					keymap.of([...defaultKeymap, ...historyKeymap, ...completionKeymap]),
					EditorView.lineWrapping,
					fluentTheme,
					EditorView.updateListener.of((update) => {
						if (!update.docChanged) return;
						const text = update.state.doc.toString();
						// Dışarıdan gelen program kaynaklı güncellemeyi tekrar
						// dışarı yollamıyoruz; yalnızca kullanıcı yazdığında.
						if (text !== value) dispatch("change", text);
					})
				]
			})
		});
	});

	onDestroy(() => view?.destroy());

	// Dışarıdan (görsel kontrollerden) gelen değişikliği editöre yansıt.
	// Metin zaten aynıysa dokunma — aksi hâlde imleç her tuşta başa zıplar.
	$: if (view && value !== view.state.doc.toString()) {
		view.dispatch({
			changes: { from: 0, to: view.state.doc.length, insert: value }
		});
	}
</script>

<div class="cm-host" bind:this={host}></div>

<style>
	/* Sadece yerleşim; görsel kararların tamamı yukarıdaki fluentTheme'de
	   --fds-* token'larına bağlı. */
	.cm-host {
		height: 320px;
		overflow: hidden;
	}
</style>
