import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { setPreviewVisible } from "$lib/projects";

const MAX_LOCK_MS = 90_000;

let running = false;

export function installEasterEgg(): () => void {
	const onKeyDown = (event: KeyboardEvent) => {
		if (running) return;
		if (!event.ctrlKey || !event.altKey || event.shiftKey || event.metaKey) return;
		if (event.code !== "KeyL") return;

		event.preventDefault();
		event.stopPropagation();
		void play();
	};

	window.addEventListener("keydown", onKeyDown, true);
	return () => window.removeEventListener("keydown", onKeyDown, true);
}

async function play(): Promise<void> {
	running = true;

	let data: string;
	try {
		data = await invoke<string>("easter_egg_open");
	} catch {
		running = false;
		return;
	}

	const previousFullscreen = await getCurrentWindow()
		.isFullscreen()
		.catch(() => false);

	const swallow = (event: Event) => {
		event.preventDefault();
		event.stopPropagation();
	};

	const overlay = document.createElement("div");
	overlay.style.cssText =
		"position:fixed;inset:0;z-index:2147483647;background:#000;" +
		"display:flex;align-items:center;justify-content:center";

	const url = URL.createObjectURL(new Blob([decode(data)], { type: "video/mp4" }));

	const video = document.createElement("video");
	video.src = url;
	video.autoplay = true;
	video.controls = false;
	video.style.cssText = "width:100%;height:100%;object-fit:contain";
	overlay.appendChild(video);

	let finished = false;
	let timer: ReturnType<typeof setTimeout> | undefined;

	const finish = async () => {
		if (finished) return;
		finished = true;
		if (timer) clearTimeout(timer);

		window.removeEventListener("keydown", swallow, true);
		window.removeEventListener("keyup", swallow, true);
		overlay.remove();
		URL.revokeObjectURL(url);

		await getCurrentWindow()
			.setFullscreen(previousFullscreen)
			.catch(() => {});
		await invoke("easter_egg_close").catch(() => {});
		running = false;
	};

	video.addEventListener("ended", () => void finish());
	video.addEventListener("error", () => void finish());

	window.addEventListener("keydown", swallow, true);
	window.addEventListener("keyup", swallow, true);
	document.body.appendChild(overlay);

	await setPreviewVisible(false).catch(() => {});
	await getCurrentWindow()
		.setFullscreen(true)
		.catch(() => {});

	try {
		await video.play();
	} catch {
		video.muted = true;
		try {
			await video.play();
		} catch {
			await finish();
			return;
		}
	}

	timer = setTimeout(() => void finish(), MAX_LOCK_MS);
}

function decode(base64: string): Uint8Array {
	const binary = atob(base64.trim());
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) {
		bytes[i] = binary.charCodeAt(i);
	}
	return bytes;
}
