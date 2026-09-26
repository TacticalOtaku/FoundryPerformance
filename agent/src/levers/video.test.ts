import { describe, expect, it } from "vitest";
import { VideoController, type VideoLike } from "./video";

function vid(paused = false): VideoLike & { plays: number } {
	return {
		paused,
		plays: 0,
		pause() {
			this.paused = true;
		},
		async play() {
			this.paused = false;
			this.plays++;
		}
	};
}

describe("VideoController", () => {
	it("static mode pauses everything", () => {
		const v = [vid(), vid()];
		const c = new VideoController(() => v);
		c.setMode("static");
		expect(v.every((x) => x.paused)).toBe(true);
	});

	it("pauseUnfocused pauses on blur and resumes only what it paused", () => {
		const playing = vid();
		const userPaused = vid(true);
		const c = new VideoController(() => [playing, userPaused]);
		c.setMode("pauseUnfocused");
		expect(playing.paused).toBe(false);
		c.setFocused(false);
		expect(playing.paused).toBe(true);
		c.setFocused(true);
		expect(playing.paused).toBe(false);
		expect(userPaused.paused).toBe(true);
		expect(userPaused.plays).toBe(0);
	});

	it("play mode resumes videos paused by static mode", () => {
		const v = vid();
		const c = new VideoController(() => [v]);
		c.setMode("static");
		c.setMode("play");
		expect(v.paused).toBe(false);
	});
});
