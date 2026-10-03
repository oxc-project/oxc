import { RuleTester } from "../rule_tester.ts";
import repos from "../../repos.json" with { type: "json" };

import type { MockFn, TestGroup } from "../index.ts";
import type { LanguageOptions } from "../rule_tester.ts";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const group: TestGroup = {
  name: "import-x",
  ...repos.import_x,
  submoduleName: "import_x",
  testFilesDirPath: "test/rules",
  cwd: ".",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".spec.ts")) return null;
    // This suite uses Jest lifecycle hooks, ESLint instances, and mutable fixture files.
    if (filename === "no-unused-modules.spec.ts") return null;
    return filename.slice(0, -".spec.ts".length);
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    const parser = require("@typescript-eslint/parser") as TSEslintParser;
    class ImportRuleTester extends RuleTester {
      constructor(config?: { languageOptions?: LanguageOptions }) {
        super({ ...config, languageOptions: { parser, ...config?.languageOptions } });
      }
    }
    mock("@typescript-eslint/rule-tester", { RuleTester: ImportRuleTester });
  },

  ruleTesters: [{ specifier: "eslint", propName: "RuleTester" }],
  parsers: [
    { specifier: "@typescript-eslint/parser", lang: "ts" },
    { specifier: "espree", lang: "jsx" },
    { specifier: "@babel/eslint-parser", lang: "jsx" },
  ],
};

export default group;
