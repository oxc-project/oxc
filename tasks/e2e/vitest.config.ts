import { readFileSync } from "node:fs";

import { transformSync } from "oxc-transform";
import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
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
        if (!id.includes("nestjs")) {
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
