import { defineConfig } from "vite";

export default defineConfig({
	build: {
		lib: {
			entry: "agent/src/main.ts",
			formats: ["iife"],
			name: "FoundryPerformanceAgent",
			fileName: () => "agent.js"
		},
		outDir: "agent/dist",
		emptyOutDir: true,
		target: "es2022",
		minify: true
	}
});
