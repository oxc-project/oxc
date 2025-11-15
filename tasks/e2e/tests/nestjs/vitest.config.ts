import { transformSync } from "oxc-transform";
import { defineConfig } from "vitest/config";
import { readFileSync } from "node:fs";

export default defineConfig({
	test: {
		include: ["**/*.e2e-spec.ts"],
		globals: true,
		root: "./",
	},
	plugins: [
		{
			name: "oxc",
			async load(id) {
				if (!id.endsWith(".ts")) {
					return;
				}
				// Use oxc-transform to transform TypeScript files
				const resolved = await this.resolve(id);
				if (!resolved) {
					return;
				}

				const code = readFileSync(resolved.id).toString();
				const result = transformSync(id, code, {
					target: "es2020",
					decorator: {
						legacy: true,
						emitDecoratorMetadata: true,
					},
				});
				return {
					code: result.code,
					map: result.map,
				};
			},
		},
	],
});
