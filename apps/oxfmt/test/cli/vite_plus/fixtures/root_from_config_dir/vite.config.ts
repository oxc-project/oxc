import { resolve } from "node:path";
import { defineConfig } from "vite-plus";

// The CLI runs in the parent directory, so `root` must not come from `process.cwd()`
export default defineConfig({
  plugins: [
    {
      name: "assert-root",
      configResolved({ root }) {
        if (resolve(root) !== import.meta.dirname) throw new Error(`Unexpected \`root\`: ${root}`);
      },
    },
  ],
  fmt: {
    semi: false,
  },
});
