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
  external: [
    // Optional peer plugins that `prettier-plugin-tailwindcss` tries to import.
    // They are not installed and not needed for us,
    // mark as external to suppress "UNRESOLVED_IMPORT" warnings.
    "@prettier/plugin-oxc",
    "@prettier/plugin-hermes",
    "@prettier/plugin-pug",
    "@shopify/prettier-plugin-liquid",
    "@zackad/prettier-plugin-twig",
    "prettier-plugin-astro",
    "prettier-plugin-marko",
    // From `prettier-plugin-svelte`
    "svelte/compiler",
  ],
  noExternal: [
    // Bundle it to control version
    "prettier",

    // Need to bundle plugins, since they depend on Prettier,
    // and must be resolved to the same instance of Prettier at runtime.
    "prettier-plugin-tailwindcss",
    "prettier-plugin-tailwindcss/sorter",
    // Also, it internally loads plugins dynamically, so they also must be bundled
    /^prettier\/plugins\//,

    // Svelte plugin, `svelte/compiler` can be peer dependency in `npm/oxfmt/package.json`
    "prettier-plugin-svelte",
    // "svelte/compiler",

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
  // This itself is not a problem.
  // It's common for packages to expose different code for ESM vs CJS,
  // and rolldown not deduplicating them may be expected behavior (it increases bundle size tho).
  //
  // However, chunk splitting breaks: the duplicated nodes shift modules across chunks,
  // producing broken `__esmMin` lazy-init wrappers at runtime
  // (e.g., `TypeError: init_postcss is not a function`).
  //
  // As a workaround, we need to explicitly alias them to the same entry.
  // Also, use the ESM entry to make deduplication work.
  inputOptions: {
    resolve: {
      alias: {
        "prettier/plugins/babel": require.resolve("prettier/plugins/babel").replace(".js", ".mjs"),
      },
    },
  },
});
