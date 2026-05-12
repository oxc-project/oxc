import { createRequire } from "node:module";
import { defineConfig } from "tsdown";

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
      // prettier-plugin-svelte's peer dependency; must be installed by the user
      "svelte/compiler",
    ],
    alwaysBundle: [
      // Bundle it to control version
      "prettier",
      // Plugins are using these internally, so we need to bundle them to avoid duplicates.
      "prettier/doc",
      /^prettier\/plugins\//,

      // Need to bundle plugins, since they depend on Prettier,
      // and must be resolved to the same instance of Prettier at runtime.
      "prettier-plugin-tailwindcss",
      "prettier-plugin-tailwindcss/sorter",
      "prettier-plugin-svelte",

      // Cannot bundle: `cli-worker.js` runs in separate thread and can't resolve bundled chunks
      // Be sure to add it to "dependencies" in `npm/oxfmt/package.json`!
      // "tinypool",
    ],
    // tsdown warns about final bundled modules by `alwaysBundle`.
    // But we know what we are doing, just suppress the warnings.
    onlyBundle: false,
  },
  inputOptions: {
    resolve: {
      alias: {
        // NOTE: `prettier-plugin-svelte` is written in CJS,
        // and tsdown(rolldown) does not deduplicate CJS imports with the ESM imports.
        // So we need to alias it to the ESM version to avoid duplicates.
        prettier: require.resolve("prettier").replace("index.cjs", "index.mjs"),
        "prettier/doc": require.resolve("prettier/doc").replace(".js", ".mjs"),
        "prettier/plugins/babel": require.resolve("prettier/plugins/babel").replace(".js", ".mjs"),
      },
    },
  },
});
