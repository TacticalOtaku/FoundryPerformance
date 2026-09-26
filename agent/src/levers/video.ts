import type { VideoMode } from "../types";

export interface VideoLike {
	paused: boolean;
	pause(): void;
	play(): Promise<void>;
}

/** Видео-текстуры сцены: фон, тайлы, токены (PIXI v7 и v8). */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function collectVideos(canvas: any): HTMLVideoElement[] {
	const out = new Set<HTMLVideoElement>();
	const add = (v: unknown) => {
		if (typeof HTMLVideoElement !== "undefined" && v instanceof HTMLVideoElement) out.add(v);
	};
	const meshes = canvas?.primary?.videoMeshes;
	if (meshes) {
		for (const m of meshes) {
			add(m?.sourceElement);
			add(m?.texture?.baseTexture?.resource?.source);
			add(m?.texture?.source?.resource);
		}
	}
	return [...out];
}

export class VideoController {
	private mode: VideoMode = "play";
	private focused = true;
	private pausedByUs = new Set<VideoLike>();

	constructor(private find: () => VideoLike[]) {}

	setMode(m: VideoMode): void {
		this.mode = m;
		this.sync();
	}

	setFocused(f: boolean): void {
		this.focused = f;
		this.sync();
	}

	sync(): void {
		const shouldPause = this.mode === "static" || (this.mode === "pauseUnfocused" && !this.focused);
		for (const v of this.find()) {
			if (shouldPause) {
				if (!v.paused) {
					v.pause();
					this.pausedByUs.add(v);
				}
			} else if (this.pausedByUs.has(v)) {
				this.pausedByUs.delete(v);
				v.play().catch(() => {});
			}
		}
	}
}
