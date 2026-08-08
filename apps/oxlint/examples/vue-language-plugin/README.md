# Vue language plugin example

Example language plugin for Oxlint, based on [RFC #21936](https://github.com/oxc-project/oxc/discussions/21936).

This is a **minimal illustration** of the `defineLanguagePlugin` contract:

- `defaultFiles: [".vue"]` — extension default (project config can override with `pattern`)
- `parse` / `load` — return a Vue-ish native AST plus an optional virtual TS transform
- pair with a JS rule plugin that visits `VElement` nodes

> **Status:** Config + API for language plugins landed in Oxlint, but the runtime
> parse/load/transform pipeline is still being implemented
> ([umbrella #23207](https://github.com/oxc-project/oxc/issues/23207)).
> This example is ready to plug in once that wiring ships.

## Usage (once runtime support lands)

```ts
// oxlint.config.ts
import { defineConfig } from "oxlint";

export default defineConfig({
  languagePlugins: ["./vue-language-plugin.ts"],
  jsPlugins: ["./vue-rules-plugin.ts"],
  overrides: [
    {
      files: ["**/*.vue"],
      rules: {
        "vue-poc/report-div": "error",
      },
    },
  ],
});
```

## Files

- `vue-language-plugin.ts` — language plugin (`parse` / `load` / `visitorKeys`)
- `vue-rules-plugin.ts` — JS rule plugin that reports `<div>` elements via the Vue AST
- `Example.vue` — sample SFC used in the RFC
