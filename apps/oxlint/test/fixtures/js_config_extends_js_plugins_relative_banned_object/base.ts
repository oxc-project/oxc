import { defineConfig } from "#oxlint";

export default defineConfig({
  jsPlugins: [{ name: "basic-custom-plugin-js", specifier: "./plugin.ts" }],
});
