// A rule option that contains a lone UTF-16 surrogate.
// `JSON.stringify` writes it as the escape `\ud800`, which the Rust side cannot hold in a `String`.
// The config must still load; the lone surrogate is replaced by U+FFFD.
import { defineConfig } from "#oxlint";

export default defineConfig({
  rules: {
    "no-restricted-globals": ["error", { name: "event", message: "lone surrogate: \ud800" }],
  },
});
