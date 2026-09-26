export class FocusThrottle {
	private isFocused = true;

	constructor(
		private getTicker: () => { maxFPS: number } | undefined,
		private focusedFps: number,
		private unfocusedFps: number
	) {}

	get focused(): boolean {
		return this.isFocused;
	}

	setFps(focused: number, unfocused: number): void {
		this.focusedFps = focused;
		this.unfocusedFps = unfocused;
		this.apply();
	}

	setFocused(f: boolean): void {
		this.isFocused = f;
		this.apply();
	}

	apply(): void {
		const t = this.getTicker();
		if (t) t.maxFPS = this.isFocused ? this.focusedFps : this.unfocusedFps;
	}
}
