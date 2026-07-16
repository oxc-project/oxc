import { defineConfig } from "#oxlint";

/**
 * Example config wiring the Vue language plugin + a Vue-native rule plugin.
 *
 * Runtime language-plugin execution is not implemented yet (#23207); this file
 * documents the intended configuration shape from RFC #21936.
 */
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
