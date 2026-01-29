import { defineConfig } from "tsdown";

export default defineConfig({
  // Build all entry points together to share Prettier chunks
  entry: ["src-js/index.ts", "src-js/cli.ts", "src-js/cli-worker.ts"],
  format: "esm",
  platform: "node",
  target: "node20",
  dts: true,
  attw: { profile: "esm-only" },
  clean: true,
  outDir: "dist",
  shims: false,
  fixedExtension: false,
  noExternal: [
    // See `/patches/prettier-plugin-tailwindcss@0.7.2.patch` for details.
    // We are using patched version which:
    // - Expose some internal APIs we need
    // - Also updated TS definitions
    // so, we must bundle it
    "prettier-plugin-tailwindcss",

    // Cannot bundle: `cli-worker.js` runs in separate thread and can't resolve bundled chunks
    // Be sure to add it to "dependencies" in `npm/oxfmt/package.json`!
    // "tinypool",
  ],
  // tsdown warns about final bundled modules by `noExternal`.
  // But we know what we are doing, just suppress the warnings.
  inlineOnly: false,
});
