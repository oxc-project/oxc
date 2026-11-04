import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite-plus";

const configDir = dirname(fileURLToPath(import.meta.url));

// A relative `root` resolves from the config file's directory, not from `process.cwd()`
export default defineConfig({
  root: "app",
  plugins: [
    {
      name: "assert-root",
      configResolved(config) {
        const expected = resolve(configDir, "app");
        if (resolve(config.root) !== expected) {
          throw new Error(`Expected \`root\` to be ${expected}, got ${config.root}`);
        }
      },
    },
  ],
  fmt: {
    semi: false,
  },
});
