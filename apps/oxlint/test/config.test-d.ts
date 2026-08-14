import { defineConfig } from "../src-js/package/config.ts";

const restrictedGlobals = ["addEventListener", "blur", "screen"];

defineConfig({
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
