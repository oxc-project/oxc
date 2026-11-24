import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    reporters: [
      process.env.CI ? "minimal" : "default",
      ...(process.env.GITHUB_ACTIONS === "true" ? ["github-actions"] : []),
    ],
  },
});
