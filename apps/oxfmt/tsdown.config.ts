import { defineConfig } from "tsdown";

export default defineConfig({
  // Build all entry points together to share Prettier chunks
  entry: ["src-js/index.ts", "src-js/cli.ts", "src-js/prettier-worker.ts"],
  format: "esm",
  platform: "node",
  target: "node20",
  dts: true,
  clean: true,
  outDir: "dist",
  shims: false,
  fixedExtension: false,
  noExternal: [
    // Bundle it to control version
    "prettier",
    // Cannot bundle: worker.js runs in separate thread and can't resolve bundled chunks
    // Be sure to add it to "dependencies" in `npm/oxfmt/package.json`!
    // "tinypool",
  ],
});
