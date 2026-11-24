import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [
    {
      // Execute fixtures that preserve standard decorators for a downstream compiler.
      name: "downstream-decorators",
      enforce: "pre",
      transform: {
        filter: { id: /-downstream-decorators-exec\.test\.js$/ },
        async handler(code) {
          const { transformAsync } = await import("@babel/core");
          const { default: decorators } = await import("@babel/plugin-proposal-decorators");
          return (
            await transformAsync(code, {
              babelrc: false,
              configFile: false,
              plugins: [[decorators, { version: "2023-11" }]],
            })
          )?.code;
        },
      },
    },
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
