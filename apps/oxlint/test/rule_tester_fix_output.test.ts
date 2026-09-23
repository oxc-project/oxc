import { describe, expect, it } from "vitest";
import { RuleTester } from "../src-js/package/rule_tester.ts";

import type { Rule } from "../src-js/plugins.ts";

function replacementRule(output: string, suggestion = false): Rule {
  return {
    meta: { fixable: "code", hasSuggestions: true, schema: [] },
    create(context) {
      return {
        Program(node) {
          context.report({
            node,
            message: "Replace source",
            ...(suggestion
              ? {
                  suggest: [
                    { desc: "Replace source", fix: (fixer) => fixer.replaceText(node, output) },
                  ],
                }
              : { fix: (fixer) => fixer.replaceText(node, output) }),
          });
        },
      };
    },
  };
}

function run(rule: Rule, test: RuleTester.InvalidTestCase, config: RuleTester.Config = {}): void {
  RuleTester.describe = (_name, fn) => fn();
  RuleTester.it = (_name, fn) => fn();
  new RuleTester(config).run("fix-output", rule, { valid: [], invalid: [test] });
}

function checkFix(output: string, test: Partial<RuleTester.InvalidTestCase> = {}): void {
  run(replacementRule(output), { code: "foo;", errors: 1, output, ...test });
}

describe("RuleTester fix output parsing", () => {
  it("rejects invalid autofix syntax even when it matches expected output", () => {
    expect(() => checkFix("const = ;")).toThrow(/Autofix: Parsing failed in fixed code/);
  });

  it("rejects semantic errors in autofix output", () => {
    expect(() => checkFix("let a; let a;")).toThrow(/Autofix: Parsing failed in fixed code/);
  });

  it("rejects invalid suggestion output with its index", () => {
    expect(() =>
      run(replacementRule("const = ;", true), {
        code: "foo;",
        errors: [
          {
            message: "Replace source",
            suggestions: [{ desc: "Replace source", output: "const = ;" }],
          },
        ],
      }),
    ).toThrow(/Suggestion at index 0: Parsing failed in fixed code/);
  });

  it("validates every suggestion independently", () => {
    const rule: Rule = {
      meta: { hasSuggestions: true, schema: [] },
      create(context) {
        return {
          Program(node) {
            context.report({
              node,
              message: "Replace source",
              suggest: ["bar;", "("].map((output) => ({
                desc: output,
                fix: (fixer) => fixer.replaceText(node, output),
              })),
            });
          },
        };
      },
    };
    expect(() =>
      run(rule, {
        code: "foo;",
        errors: [
          {
            message: "Replace source",
            suggestions: [
              { desc: "bar;", output: "bar;" },
              { desc: "(", output: "(" },
            ],
          },
        ],
      }),
    ).toThrow(/Suggestion at index 1: Parsing failed in fixed code/);
  });

  it("accepts valid output without invoking the rule again", () => {
    let calls = 0;
    const rule: Rule = {
      meta: { fixable: "code", schema: [] },
      create(context) {
        calls++;
        return {
          Program(node) {
            context.report({
              node,
              message: "Replace source",
              fix: (fixer) => fixer.replaceText(node, "bar;"),
            });
          },
        };
      },
    };
    run(rule, { code: "foo;", errors: 1, output: "bar;" });
    expect(calls).toBe(1);
  });

  it("validates the final recursive fix pass", () => {
    const rule: Rule = {
      meta: { fixable: "code", schema: [] },
      create(context) {
        return {
          Program(node) {
            const output = context.sourceCode.text === "foo;" ? "bar;" : "(";
            context.report({
              node,
              message: "Replace source",
              fix: (fixer) => fixer.replaceText(node, output),
            });
          },
        };
      },
    };
    expect(() => run(rule, { code: "foo;", errors: 1, output: "(", recursive: 1 })).toThrow(
      /Autofix: Parsing failed in fixed code/,
    );
  });
});

describe("RuleTester output parser configuration", () => {
  it.each([
    { filename: "file.ts", output: "let value: number = 1;" },
    { filename: "file.jsx", output: "<div />;" },
    { filename: "file.tsx", output: "const value: JSX.Element = <div />;" },
    { filename: "file.d.ts", output: "declare const value: number;" },
  ])("uses the language of $filename", ({ filename, output }) => {
    expect(() =>
      checkFix(output, {
        filename,
        code: filename.endsWith(".d.ts") ? "declare const foo: number;" : "foo;",
      }),
    ).not.toThrow();
  });

  it("uses parser options when there is no filename", () => {
    expect(() =>
      checkFix("let value: number = 1;", {
        languageOptions: { parserOptions: { lang: "ts" } },
      }),
    ).not.toThrow();
  });

  it("uses JSX parser options when there is no filename", () => {
    expect(() =>
      checkFix("<div />;", {
        languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
      }),
    ).not.toThrow();
  });

  it("gives the filename precedence over the language option", () => {
    expect(() =>
      checkFix("let value: number = 1;", {
        filename: "file.js",
        languageOptions: { parserOptions: { lang: "ts" } },
      }),
    ).toThrow(/Autofix: Parsing failed in fixed code/);
  });

  it("inherits parser options from the constructor", () => {
    const output = "let value: number = 1;";
    expect(() =>
      run(
        replacementRule(output),
        { code: "foo;", errors: 1, output },
        {
          languageOptions: { parserOptions: { lang: "ts" } },
        },
      ),
    ).not.toThrow();
  });

  it("rejects module syntax in script output", () => {
    expect(() => checkFix("export {};", { languageOptions: { sourceType: "script" } })).toThrow(
      /Autofix: Parsing failed in fixed code/,
    );
  });

  it("accepts module syntax in module output", () => {
    expect(() =>
      checkFix("export {};", { languageOptions: { sourceType: "module" } }),
    ).not.toThrow();
  });

  it("detects the source type from unambiguous output", () => {
    expect(() =>
      checkFix("export {};", { languageOptions: { sourceType: "unambiguous" } }),
    ).not.toThrow();
  });

  it("accepts commonjs output", () => {
    expect(() =>
      checkFix("return;", { languageOptions: { sourceType: "commonjs" } }),
    ).not.toThrow();
  });

  it("honors the default module source type in ESLint compatibility mode", () => {
    expect(() => checkFix("with ({}) {}", { eslintCompat: true })).toThrow(
      /Autofix: Parsing failed in fixed code/,
    );
  });

  it("allows script output that would be invalid in a module", () => {
    expect(() =>
      checkFix("with ({}) {}", { languageOptions: { sourceType: "script" } }),
    ).not.toThrow();
  });

  it("honors ignored semantic errors", () => {
    expect(() =>
      checkFix("let a; let a;", {
        languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
      }),
    ).not.toThrow();
  });

  it("still rejects fatal syntax when nonfatal errors are ignored", () => {
    expect(() =>
      checkFix("(", {
        languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
      }),
    ).toThrow(/Autofix: Parsing failed in fixed code/);
  });

  it("preserves BOM and Unicode when applying and parsing fixes", () => {
    expect(() => checkFix('"😀";', { code: '\uFEFF"🦀";', output: '\uFEFF"😀";' })).not.toThrow();
  });

  it("uses the same parser configuration for suggestions", () => {
    const output = "let value: number = 1;";
    expect(() =>
      run(replacementRule(output, true), {
        filename: "file.ts",
        code: "foo;",
        errors: [{ message: "Replace source", suggestions: [{ desc: "Replace source", output }] }],
      }),
    ).not.toThrow();
  });

  it("keeps original diagnostic state intact across suggestions and autofixes", () => {
    const rule: Rule = {
      meta: { fixable: "code", hasSuggestions: true, schema: [] },
      create(context) {
        return {
          Identifier(node) {
            context.report({
              node,
              message: "Replace identifier",
              fix: (fixer) => fixer.replaceText(node, "bar"),
              suggest: [{ desc: "Rename", fix: (fixer) => fixer.replaceText(node, "baz") }],
            });
          },
        };
      },
    };
    expect(() =>
      run(rule, {
        code: "foo; qux;",
        output: "bar; bar;",
        errors: [
          {
            message: "Replace identifier",
            column: 0,
            suggestions: [{ desc: "Rename", output: "baz; qux;" }],
          },
          {
            message: "Replace identifier",
            column: 5,
            suggestions: [{ desc: "Rename", output: "foo; baz;" }],
          },
        ],
      }),
    ).not.toThrow();
  });

  it("runs the after hook when output parsing fails", () => {
    let afterCalls = 0;
    expect(() =>
      checkFix("(", {
        after() {
          afterCalls++;
        },
      }),
    ).toThrow(/Autofix: Parsing failed in fixed code/);
    expect(afterCalls).toBe(1);
  });
});

describe("RuleTester output parser diagnostics", () => {
  it("rejects invalid regular expression syntax", () => {
    expect(() => checkFix("const re = /(/;")).toThrow(/Autofix: Parsing failed in fixed code/);
  });

  it("rejects recoverable parser diagnostics by default", () => {
    expect(() => checkFix("const value;")).toThrow(/Autofix: Parsing failed in fixed code/);
  });

  it("honors ignored recoverable parser diagnostics", () => {
    expect(() =>
      checkFix("const value;", {
        languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
      }),
    ).not.toThrow();
  });
});

describe("RuleTester combined fixes", () => {
  it("validates the complete fix batch rather than each partial edit", () => {
    const rule: Rule = {
      meta: { fixable: "code", schema: [] },
      create(context) {
        return {
          Program(node) {
            context.report({
              node,
              message: "Declare value",
              fix: (fixer) => fixer.replaceTextRange([0, 3], "const value"),
            });
            context.report({
              node,
              message: "Assign value",
              fix: (fixer) => fixer.replaceTextRange([4, 5], "="),
            });
          },
        };
      },
    };
    expect(() =>
      run(rule, { code: "foo + bar;", errors: 2, output: "const value = bar;" }),
    ).not.toThrow();
  });
});
