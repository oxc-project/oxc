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
  define: { "import.meta.vitest": "undefined" },
  deps: {
    // Optional peer plugins that `prettier-plugin-tailwindcss` tries to dynamic import.
    // They are not installed and not needed for us,
    // mark as external to suppress "UNRESOLVED_IMPORT" warnings.
    neverBundle: [
      "@prettier/plugin-oxc",
      "@prettier/plugin-hermes",
      "@prettier/plugin-pug",
      "@shopify/prettier-plugin-liquid",
      "@zackad/prettier-plugin-twig",
      "prettier-plugin-astro",
      "prettier-plugin-marko",
      "prettier-plugin-svelte",
    ],
    alwaysBundle: [
      // Bundle it to control version
      "prettier",

      // Need to bundle plugins, since they depend on Prettier,
      // and must be resolved to the same instance of Prettier at runtime.
      "prettier-plugin-tailwindcss",
      "prettier-plugin-tailwindcss/sorter",
      // Also, it internally loads plugins dynamically, so they also must be bundled
      /^prettier\/plugins\//,

      // Cannot bundle: `cli-worker.js` runs in separate thread and can't resolve bundled chunks
      // Be sure to add it to "dependencies" in `npm/oxfmt/package.json`!
      // "tinypool",
    ],
    // tsdown warns about final bundled modules by `alwaysBundle`.
    // But we know what we are doing, just suppress the warnings.
    onlyBundle: false,
  },
});
