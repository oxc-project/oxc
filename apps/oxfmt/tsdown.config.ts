import { defineConfig } from "tsdown";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

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
  // Optional peer plugins that `prettier-plugin-tailwindcss` tries to dynamic import.
  // They are not installed and not needed for us,
  // mark as external to suppress "UNRESOLVED_IMPORT" warnings.
  external: [
    "@prettier/plugin-oxc",
    "@prettier/plugin-hermes",
    "@prettier/plugin-pug",
    "@shopify/prettier-plugin-liquid",
    "@zackad/prettier-plugin-twig",
    "prettier-plugin-astro",
    "prettier-plugin-marko",
  ],
  noExternal: [
    // Bundle it to control version
    "prettier",

    // We are using patched version, so we must bundle it
    // Also, it internally loads plugins dynamically, so they also must be bundled
    "prettier-plugin-tailwindcss",
    "prettier-plugin-tailwindcss/sorter",
    /^prettier\/plugins\//,

    // Svelte plugin and its compiler dependency
    "prettier-plugin-svelte",
    "svelte",

    // Cannot bundle: `cli-worker.js` runs in separate thread and can't resolve bundled chunks
    // Be sure to add it to "dependencies" in `npm/oxfmt/package.json`!
    // "tinypool",
  ],
  // tsdown warns about final bundled modules by `noExternal`.
  // But we know what we are doing, just suppress the warnings.
  inlineOnly: false,
  // XXX: Workaround for rolldown code-splitting bug/limitation(?)
  //
  // `prettier-plugin-svelte` internally does CJS `require("prettier/plugins/babel")`,
  // while Prettier itself uses ESM `import("prettier/plugins/babel")`.
  // Both resolve to the same package, but rolldown treats the CJS and ESM entry points
  // (`babel.js` vs `babel.mjs`) as separate module graph nodes.
  //
  // This causes two problems:
  // 1. Chunk splitting breaks: the duplicated nodes shift modules across chunks,
  //    producing broken `__esmMin` lazy-init wrappers at runtime
  //    (e.g., `TypeError: init_postcss is not a function`).
  // 2. Without the alias, both `babel.js` (CJS, ~395KB) and `babel.mjs` (ESM, ~383KB)
  //    would be bundled, duplicating ~780KB of Babel parser code.
  //
  // Pinning to the `.mjs` (ESM) absolute path forces rolldown to deduplicate:
  // the CJS `require()` is redirected to the same ESM module that Prettier already uses.
  inputOptions: {
    resolve: {
      alias: {
        "prettier/plugins/babel": require.resolve("prettier/plugins/babel").replace(".js", ".mjs"),
      },
    },
  },
});
