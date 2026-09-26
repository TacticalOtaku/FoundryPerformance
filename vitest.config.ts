import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		include: ["agent/src/**/*.test.ts", "src/**/*.test.ts"],
		environment: "node"
	}
});
