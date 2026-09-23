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

function run(rule: Rule, test: RuleTester.InvalidTestCase, config = {}): void {
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
