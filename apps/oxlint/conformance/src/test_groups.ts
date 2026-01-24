import type { TestGroup } from "./index.ts";
import type { TestCase } from "./rule_tester.ts";

export const TEST_GROUPS: TestGroup[] = [
  // ESLint
  {
    name: "eslint",
    submoduleName: "eslint",
    testFilesDirPath: "tests/lib/rules",

    transformTestFilename: (filename: string) => {
      if (!filename.endsWith(".js")) return null;
      return filename.slice(0, -3);
    },

    shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
      // We cannot support custom parsers
      if (err.message === "Custom parsers are not supported") return true;

      // Skip test cases which start with `/* global */`, `/* globals */`, `/* exported */`, or `/* eslint */` comments.
      // Oxlint does not support defining globals inline.
      // `RuleTester` does not support enabling other rules beyond the rule under test.
      if (code.match(/^\s*\/\*\s*(globals?|exported|eslint)\s/)) return true;

      // Skip test cases which include `// eslint-disable` comments.
      // These are not handled by `RuleTester`.
      if (code.match(/\/\/\s*eslint-disable((-next)?-line)?(\s|$)/)) return true;

      // Tests rely on directives being parsed as plain `StringLiteral`s in ES3.
      // Oxc parser does not support parsing as ES3.
      if (
        (ruleName === "no-eval" ||
          ruleName === "no-invalid-this" ||
          ruleName === "no-unused-expressions") &&
        test.languageOptions?.ecmaVersion === 3
      ) {
        return true;
      }

      // Test relies on scope analysis to follow ES5 semantics where function declarations in blocks are bound in parent scope.
      // TS-ESLint scope manager does not support ES5. Oxc also doesn't support parsing/semantic as ES5.
      if (
        ruleName === "no-use-before-define" &&
        code === '"use strict"; a(); { function a() {} }' &&
        test.languageOptions?.ecmaVersion === 5
      ) {
        return true;
      }

      // Code contains unrecoverable syntax error - `function (x, this: context) {}`
      if (
        ruleName === "no-invalid-this" &&
        code.includes("function (x, this: context) {") &&
        err?.message === "Parsing failed"
      ) {
        return true;
      }

      // TypeScript parser does not support HTML comments
      if (ruleName === "prefer-object-spread" && code.includes("<!--")) return true;

      return false;
    },

    ruleTesters: ["../../../lib/rule-tester/rule-tester.js"],
    parsers: [{ specifier: "@typescript-eslint/parser", lang: "ts" }],
  },

  // React
  {
    name: "react-hooks",
    submoduleName: "react",
    testFilesDirPath: "packages/eslint-plugin-react-hooks/__tests__",

    transformTestFilename: (filename: string) => {
      switch (filename) {
        case "ESLintRuleExhaustiveDeps-test.js":
          return "exhaustive-deps";
        case "ESLintRulesOfHooks-test.js":
          return "rules-of-hooks";
        case "ReactCompilerRuleTypescript-test.ts":
          return "compiler";
        default:
          return null;
      }
    },

    prepare(require, mock) {
      // Add `default` export to `eslint-plugin-react-hooks` module
      const plugin = require("eslint-plugin-react-hooks") as any;
      plugin.default = plugin;

      // Mock `react/packages/eslint-plugin-react-hooks/src/shared/ReactCompiler.ts`
      // to use actual `eslint-plugin-react-hooks` package.
      // This avoids having to build the React compiler.
      const { rules } = plugin;
      const allRules = Object.fromEntries(
        Object.entries(rules).map(([name, rule]) => [name, { rule }]),
      );
      mock("../src/shared/ReactCompiler.ts", { allRules });
    },

    shouldSkipTest(ruleName: string, test: TestCase, code: string, err: Error): boolean {
      // Code is flow syntax
      if (
        ruleName.startsWith("rules-of-hooks/") &&
        err.message === "Parsing failed" &&
        code.match(/^\s*(\/\/[^\n]*\n)*(hook|component) [a-zA-Z]/)
      ) {
        return true;
      }

      // Code is TypeScript, but they're being parsed as JSX
      if (
        ruleName.startsWith("rules-of-hooks/") &&
        err.message === "Parsing failed" &&
        [
          "function Example({ prop }) {\n  const bar = useEffect(<T>(a: T): Hello => {\n    prop();\n  }, [prop]);\n}",
          "function Foo() {\n  const foo = ({}: any);\n  useMemo(() => {\n    console.log(foo);\n  }, [foo]);\n}",
        ].includes(code.trim())
      ) {
        return true;
      }

      return false;
    },

    ruleTesters: ["eslint-v7", "eslint-v8", "eslint-v9"],

    parsers: [
      { specifier: "babel-eslint", lang: "jsx" },
      { specifier: "@babel/eslint-parser", lang: "jsx" },
      { specifier: "hermes-eslint", lang: "jsx" },
      { specifier: "@typescript-eslint/parser-v2", lang: "tsx" },
      { specifier: "@typescript-eslint/parser-v3", lang: "tsx" },
      { specifier: "@typescript-eslint/parser-v4", lang: "tsx" },
      { specifier: "@typescript-eslint/parser-v5", lang: "tsx" },
    ],
  },
];
