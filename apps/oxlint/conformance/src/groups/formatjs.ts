import assert from "node:assert";
import { RuleTester } from "../rule_tester.ts";
import repos from "../../repos.json" with { type: "json" };

import type { MockFn, TestGroup } from "../index.ts";

const group: TestGroup = {
  name: "formatjs",
  ...repos.formatjs,
  submoduleName: "formatjs",
  testFilesDirPath: ".oxlint-conformance/tests",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    // The emoji helper and inline message type tests do not use RuleTester.
    if (filename === "emoji-utils.test.ts") return null;
    return filename.slice(0, -".test.ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Keep the RuleTester cases in `message-recognition.test.ts`, but exclude
    // its separate bundler integration suite, which does not exercise a lint rule.
    mock("vitest", {
      describe(name: string) {
        assert.equal(name, "unplugin message recognition parity");
      },
    });

    // Preserve the upstream tester options without its Vitest hook assignments.
    const vueParser = require("vue-eslint-parser");
    const testers = {
      ruleTester: new RuleTester({
        languageOptions: { sourceType: "module", parserOptions: { lang: "tsx" } },
      }),
      vueRuleTester: new RuleTester({
        languageOptions: {
          parser: vueParser,
          ecmaVersion: 6,
          sourceType: "module",
          parserOptions: { ecmaFeatures: { globalReturn: false, jsx: false } },
        },
      }),
    };
    mock("./util.ts", testers);
  },

  ruleTesters: [],
  parsers: [],
};

export default group;
