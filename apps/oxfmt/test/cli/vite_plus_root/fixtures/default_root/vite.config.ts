import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite-plus";

const configDir = dirname(fileURLToPath(import.meta.url));

// The CLI runs in the parent directory, so `root` must not come from `process.cwd()`
export default defineConfig({
  plugins: [
    {
      name: "assert-root",
      configResolved(config) {
        if (resolve(config.root) !== configDir) {
          throw new Error(`Expected \`root\` to be ${configDir}, got ${config.root}`);
        }
      },
    },
  ],
  fmt: {
    semi: false,
  },
});
