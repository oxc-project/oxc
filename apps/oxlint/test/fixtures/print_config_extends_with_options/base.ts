import { defineConfig } from "#oxlint";

export default defineConfig({
  rules: {
    "getter-return": ["error", { "allowImplicit": true }],
    "no-console": ["warn", { "allow": ["info", "warn", "error"] }],
  },
});
