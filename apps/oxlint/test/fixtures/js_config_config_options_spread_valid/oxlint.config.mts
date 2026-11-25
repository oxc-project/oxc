import { defineConfig } from "#oxlint";

const restrictedGlobals = ["addEventListener", "blur", "screen"];

export default defineConfig({
  rules: {
    "no-restricted-globals": [
      "warn",
      ...restrictedGlobals.map((name) => ({
        name,
        message: `Use globalThis.${name} if intentional.`,
      })),
    ],
  },
});

defineConfig({
  rules: {
    "no-restricted-globals": ["warn", ...restrictedGlobals],
  },
});
