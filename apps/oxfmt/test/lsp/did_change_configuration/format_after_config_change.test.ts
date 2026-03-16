import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { formatFixtureAfterConfigChange } from "../utils";

const FIXTURES_DIR = join(import.meta.dirname, "fixtures");

const suite = [
  ["config-semi/test.ts", "typescript"],
  ["config-vue-indent/test.vue", "vue"],
  ["config-sort-imports/test.js", "javascript"],
  ["config-sort-tailwindcss/test.tsx", "typescriptreact"],
  ["config-sort-tailwindcss/test.vue", "vue"],
];

describe("LSP formatting after config change", () => {
  describe("config formatting", () => {
    it.each(suite)(
      "should handle default-config to specific config in %s",
      async (path, languageId) => {
        expect(
          await formatFixtureAfterConfigChange(
            FIXTURES_DIR,
            path,
            languageId,
            {
              "fmt.configPath": "empty.json",
            },
            {
              // server should use the `.oxfmtrc.json` in the directory
              "fmt.configPath": null,
            },
          ),
        ).toMatchSnapshot();
      },
    );

    it.each(suite)(
      "should handle specific config to default config in %s",
      async (path, languageId) => {
        expect(
          await formatFixtureAfterConfigChange(
            FIXTURES_DIR,
            path,
            languageId,
            {
              // server should use the `.oxfmtrc.json` in the directory
              "fmt.configPath": null,
            },
            {
              "fmt.configPath": "empty.json",
            },
          ),
        ).toMatchSnapshot();
      },
    );
  });
});
