import type { MockFn, TestGroup } from "../index.ts";
import type { TestCase } from "../rule_tester.ts";

const group: TestGroup = {
  name: "react-hooks",

  submoduleName: "react",
  testFilesDirPath: "packages/eslint-plugin-react-hooks/__tests__",

  transformTestFilename(filename: string) {
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

  prepare(require: NodeJS.Require, mock: MockFn) {
    // Add `default` export to `eslint-plugin-react-hooks` module
    const plugin = require("eslint-plugin-react-hooks") as any;
    plugin.default = plugin;

    // @overlookmotel says: @camc314 added the next block to this script, but it doesn't seem to work on my machine.
    // Presumably it's because we're using different versions of `yarn` (see `init.sh`),
    // but I can't track down the problem exactly. So I'm commenting it out again for now.

    /*
    // Use published plugin build to avoid requiring React compiler workspace artifacts.
    const plugin = require("eslint-plugin-react-hooks-published") as any;
    plugin.default = plugin;
    mock("eslint-plugin-react-hooks", plugin);
    */

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

  ruleTesters: [
    { specifier: "eslint-v7", propName: "RuleTester" },
    { specifier: "eslint-v8", propName: "RuleTester" },
    { specifier: "eslint-v9", propName: "RuleTester" },
  ],

  parsers: [
    { specifier: "babel-eslint", lang: "jsx" },
    { specifier: "@babel/eslint-parser", lang: "jsx" },
    { specifier: "hermes-eslint", lang: "jsx" },
    { specifier: "@typescript-eslint/parser-v2", lang: "tsx" },
    { specifier: "@typescript-eslint/parser-v3", lang: "tsx" },
    { specifier: "@typescript-eslint/parser-v4", lang: "tsx" },
    { specifier: "@typescript-eslint/parser-v5", lang: "tsx" },
  ],
};

export default group;
