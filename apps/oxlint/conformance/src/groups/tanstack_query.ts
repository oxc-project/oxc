import { describe, it } from "../capture.ts";
import { RuleTester } from "../rule_tester.ts";
import repos from "../../repos.json" with { type: "json" };

import type { MockFn, TestGroup } from "../index.ts";
import type { LanguageOptions, TestCase } from "../rule_tester.ts";

type TSEslintParser = typeof import("@typescript-eslint/parser");

const group: TestGroup = {
  name: "tanstack-query",
  ...repos.tanstack_query,

  submoduleName: "tanstack_query",
  testFilesDirPath: "packages/eslint-plugin-query/src/__tests__",

  transformTestFilename(filename: string) {
    if (!filename.endsWith(".test.ts")) return null;
    // These files test helper functions rather than running rules with RuleTester.
    if (
      filename === "ast-utils.test.ts"
      || filename === "sort-data-by-order.utils.test.ts"
      || filename === "test-utils.test.ts"
    ) {
      return null;
    }
    return filename.slice(0, -".test.ts".length).replace(/\.rule$/, "");
  },

  prepare(require: NodeJS.Require, mock: MockFn) {
    const tsEslintParser = require("@typescript-eslint/parser") as TSEslintParser;

    // The shared helpers import Vitest assertions, but the rule tests don't use them.
    // Vitest hook assignments in the test files are removed by init.sh.
    mock("vitest", { describe, it });

    // TS-ESLint's RuleTester defaults to its TypeScript parser.
    class TsRuleTester extends RuleTester {
      constructor(config?: { languageOptions?: LanguageOptions } | null) {
        super({
          ...config,
          languageOptions: {
            parser: tsEslintParser,
            ...config?.languageOptions,
          },
        });
      }
    }
    mock("@typescript-eslint/rule-tester", { RuleTester: TsRuleTester });
  },

  shouldSkipTest(ruleName: string, test: TestCase, _code: string, err: Error): boolean {
    // These cases require a TypeScript Program and ESTree-to-TS node map.
    // Oxlint does not supply type information to JS plugins, so the rules return
    // without reporting the diagnostic expected by these type-aware cases.
    if (
      ruleName !== "no-void-query-fn"
      && ruleName !== "no-rest-destructuring with type information"
    ) {
      return false;
    }
    const parserOptions = test.languageOptions?.parserOptions;
    return (
      parserOptions != null
      && "project" in parserOptions
      && parserOptions.project === true
      && err.message.startsWith("Should have 1 error but had 0:")
    );
  },

  ruleTesters: [],
  parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
};

export default group;
