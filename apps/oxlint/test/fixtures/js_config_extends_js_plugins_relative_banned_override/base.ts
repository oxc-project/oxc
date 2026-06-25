import { defineConfig } from "#oxlint";

export default defineConfig({
  overrides: [
    {
      files: ["files/**/*.js"],
      jsPlugins: ["./plugin.ts"],
    },
  ],
});
