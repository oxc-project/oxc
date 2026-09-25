import repos from "../../repos.json" with { type: "json" };

import type { TestGroup } from "../index.ts";
import type { TestCase } from "../rule_tester.ts";

const group: TestGroup = {
  name: "no-secrets",
  ...repos.no_secrets,

  submoduleName: "no_secrets",
  testFilesDirPath: "tests/lib/rules",

  transformTestFilename(filename: string) {
    // The directory also contains the suite entry point and shared tester setup.
    if (filename !== "no-secrets.ts" && filename !== "no-pattern-match.ts") return null;
    return filename.slice(0, -".ts".length);
  },

  shouldSkipTest(ruleName: string, test: TestCase, _code: string, err: Error): boolean {
    // Oxlint options must be JSON-serializable. Upstream passes RegExp objects,
    // whose flags and source are lost when serialized, changing which text matches.
    if (
      ruleName !== "no-pattern-match"
      || !/^Should have (no|\d+) errors? but had /.test(err.message)
    ) {
      return false;
    }

    const options = test.options?.[0];
    if (typeof options !== "object" || options === null || Array.isArray(options)) return false;
    const { patterns } = options;
    return (
      typeof patterns === "object"
      && patterns !== null
      && Object.values(patterns).some((pattern) => pattern instanceof RegExp)
    );
  },

  ruleTesters: [
    { specifier: "eslint6", propName: "RuleTester" },
    { specifier: "eslint", propName: "RuleTester" },
    { specifier: "eslint8", propName: "RuleTester" },
    { specifier: "eslint9", propName: "RuleTester" },
    { specifier: "eslint10", propName: "RuleTester" },
  ],

  parsers: [],
};

export default group;
