import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [
    {
      // Resolve helper imports to this checkout before Vite's package resolution.
      name: "local-oxc-runtime",
      enforce: "pre",
      resolveId: {
        order: "pre",
        handler(source) {
          const match = /^@oxc-project\/runtime\/helpers\/(?:esm\/)?([^/]+?)(?:\.js)?$/.exec(
            source,
          );
          return match
            ? fileURLToPath(
                new URL(`../../npm/runtime/src/helpers/esm/${match[1]}.js`, import.meta.url),
              )
            : null;
        },
      },
    },
  ],
});
