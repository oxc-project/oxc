import { defineConfig } from "vitest/config";

export default defineConfig({
  define: {
    DEBUG: "true",
    CONFORMANCE: "false",
  },
  test: {
    projects: [
      {
        extends: true,
        test: {
          name: "tests",
          include: ["test/**/*.test.ts"],
          exclude: ["test/**/*.isolated.test.ts"],
        },
      },
      {
        // Tests matching `*.isolated.test.ts` run in separate child processes (via `forks` pool)
        // to prevent side effects (e.g. patching `globalThis.WeakMap`) from leaking into other unit tests.
        extends: true,
        test: {
          name: "isolated",
          include: ["test/**/*.isolated.test.ts"],
          pool: "forks",
        },
      },
    ],
  },
});
