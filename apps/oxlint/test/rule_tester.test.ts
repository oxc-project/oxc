import { beforeEach, describe, expect, it, vi } from "vitest";
import { RuleTester } from "../src-js/package/rule_tester.ts";

import type { Rule } from "../src-js/plugins.ts";

/**
 * Test case.
 */
interface Case {
  path: string[];
  fn: () => void;
  only: boolean;
}

// Current test cases
const cases: Case[] = [];

// Current test case stack
const caseStack: string[] = [];

// Set up `RuleTester` to use these `describe` and `it` functions
function describeHook(name: string, fn: () => void): void {
  caseStack.push(name);
  try {
    return fn();
  } finally {
    caseStack.pop();
  }
}
RuleTester.describe = describeHook;

function itHook(name: string, fn: () => void): void {
  cases.push({ path: caseStack.concat([name]), fn, only: false });
}
itHook.only = function itOnlyHook(name: string, fn: () => void): void {
  cases.push({ path: caseStack.concat([name]), fn, only: true });
};
RuleTester.it = itHook;

/**
 * Run all current test cases.
 * @returns Array containing errors for each test case
 */
function runCases(): (Error | null)[] {
  const errors: (Error | null)[] = [];
  for (const testCase of cases) {
    let error = null;
    try {
      testCase.fn();
    } catch (err) {
      error = err as Error;
    }
    errors.push(error);
  }
  return errors;
}

// Simple rule which flags identifiers named "foo"
const simpleRule: Rule = {
  create(context) {
    return {
      Identifier(node) {
        if (node.name === "foo") context.report({ message: "No foo!", node });
      },
    };
  },
};

// Rule which flags all identifiers, and uses message IDs to report messages
const messageIdsRule: Rule = {
  meta: {
    messages: {
      noFoo: "No foo!",
      noBar: "No bar! {{name}}",
      noIdentifiers: "No identifiers! {{name}}",
    },
  },
  create(context) {
    return {
      Identifier(node) {
        if (node.name === "foo") {
          context.report({ messageId: "noFoo", node });
        } else if (node.name === "bar") {
          context.report({ messageId: "noBar", node });
        } else {
          context.report({ messageId: "noIdentifiers", data: { name: node.name }, node });
        }
      },
    };
  },
};

// Tests
describe("RuleTester", () => {
  beforeEach(() => {
    RuleTester.resetDefaultConfig();
    cases.length = 0;
    caseStack.length = 0;
  });

  describe("can be constructed", () => {
    it("with no config", () => {
      expect(() => new RuleTester()).not.toThrow();
    });

    it("with config", () => {
      expect(() => new RuleTester({})).not.toThrow();
    });
  });

  it("generates test cases", () => {
    const tester = new RuleTester();
    tester.run("my-rule", simpleRule, {
      valid: [
        "valid code string",
        {
          code: "valid code from object",
        },
        {
          name: "valid case name",
          code: "let x = 1;",
        },
      ],
      invalid: [
        {
          code: "invalid code from object",
          errors: 1,
        },
        {
          name: "invalid case name",
          code: "let x = 1;",
          errors: 1,
        },
      ],
    });

    expect(cases).toEqual([
      { path: ["my-rule", "valid", "valid code string"], fn: expect.any(Function), only: false },
      {
        path: ["my-rule", "valid", "valid code from object"],
        fn: expect.any(Function),
        only: false,
      },
      { path: ["my-rule", "valid", "valid case name"], fn: expect.any(Function), only: false },
      {
        path: ["my-rule", "invalid", "invalid code from object"],
        fn: expect.any(Function),
        only: false,
      },
      { path: ["my-rule", "invalid", "invalid case name"], fn: expect.any(Function), only: false },
    ]);
  });

  describe("config", () => {
    it("can be set globally", () => {
      const config = { eslintCompat: true };
      RuleTester.setDefaultConfig(config);
      expect(RuleTester.getDefaultConfig()).toBe(config);
    });

    it("is reset to default by `resetDefaultConfig`", () => {
      RuleTester.setDefaultConfig({ eslintCompat: true });
      expect(RuleTester.getDefaultConfig()).toHaveProperty("eslintCompat", true);

      RuleTester.resetDefaultConfig();
      expect(RuleTester.getDefaultConfig()).not.toHaveProperty("eslintCompat");
    });

    it("cannot permanently change default config", () => {
      const defaultConfig = RuleTester.getDefaultConfig();
      defaultConfig.eslintCompat = true;
      expect(RuleTester.getDefaultConfig()).toBe(defaultConfig);

      RuleTester.resetDefaultConfig();
      expect(RuleTester.getDefaultConfig()).not.toHaveProperty("eslintCompat");
    });
  });

  describe("tests valid cases", () => {
    it("which are correct", () => {
      const tester = new RuleTester();
      tester.run("no-foo", simpleRule, {
        valid: ["let x;", "let y;"],
        invalid: [],
      });

      expect(runCases()).toEqual([null, null]);
    });

    it("which are incorrect", () => {
      const tester = new RuleTester();
      tester.run("no-foo", simpleRule, {
        valid: ["let foo;", "foo.foo;", "let foo;", "let foo;"],
        invalid: [],
      });

      expect(runCases()).toMatchInlineSnapshot(`
        [
          [AssertionError: Should have no errors but had 1: [
          {
            ruleId: 'rule-to-test/no-foo',
            message: 'No foo!',
            messageId: null,
            severity: 1,
            nodeType: 'Identifier',
            line: 1,
            column: 4,
            endLine: 1,
            endColumn: 7,
            fixes: null,
            suggestions: null
          }
        ]

        1 !== 0
        ],
          [AssertionError: Should have no errors but had 2: [
          {
            ruleId: 'rule-to-test/no-foo',
            message: 'No foo!',
            messageId: null,
            severity: 1,
            nodeType: 'Identifier',
            line: 1,
            column: 0,
            endLine: 1,
            endColumn: 3,
            fixes: null,
            suggestions: null
          },
          {
            ruleId: 'rule-to-test/no-foo',
            message: 'No foo!',
            messageId: null,
            severity: 1,
            nodeType: 'Identifier',
            line: 1,
            column: 4,
            endLine: 1,
            endColumn: 7,
            fixes: null,
            suggestions: null
          }
        ]

        2 !== 0
        ],
          [AssertionError: Detected duplicate test case],
          [AssertionError: Detected duplicate test case],
        ]
      `);
    });
  });

  describe("tests invalid cases", () => {
    describe("which are correct", () => {
      it("with error count", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            { code: "let foo;", errors: 1 },
            { code: "foo.foo;", errors: 2 },
          ],
        });

        expect(runCases()).toEqual([null, null]);
      });

      it("with error messages", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            {
              code: "let foo;",
              errors: ["No foo!"],
            },
            {
              code: "foo.foo;",
              errors: ["No foo!", "No foo!"],
            },
            {
              code: "let foo = 1;",
              errors: ["No foo!"],
            },
            {
              code: "foo.foo = 2;",
              errors: [/^No foo!$/, /o [fg]o/],
            },
            {
              code: "let foo = 3;",
              errors: [{ message: "No foo!" }],
            },
            {
              code: "foo.foo = 4;",
              errors: [{ message: "No foo!" }, { message: "No foo!" }],
            },
            {
              code: "let foo = 5;",
              errors: [{ message: "No foo!" }],
            },
            {
              code: "foo.foo = 6;",
              errors: [{ message: /^No foo!$/ }, { message: /o [fg]o/ }],
            },
          ],
        });
        expect(runCases()).toEqual([null, null, null, null, null, null, null, null]);
      });

      it("with message IDs and data", () => {
        const tester = new RuleTester();
        tester.run("no-foo", messageIdsRule, {
          valid: [],
          invalid: [
            {
              code: "let foo;",
              errors: [
                {
                  messageId: "noFoo",
                },
              ],
            },
            {
              code: "qux.bing;",
              errors: [
                // Without data
                {
                  messageId: "noIdentifiers",
                },
                // With data
                {
                  messageId: "noIdentifiers",
                  data: { name: "bing" },
                },
              ],
            },
          ],
        });
        expect(runCases()).toEqual([null, null]);
      });

      it("with location", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            {
              code: "let foo = 1;\nfoo = 2;",
              errors: [
                {
                  message: "No foo!",
                  line: 1,
                },
                {
                  message: "No foo!",
                  line: 2,
                },
              ],
            },
            {
              code: "let foo = 1;\nfoo = 3;",
              errors: [
                {
                  message: "No foo!",
                  column: 4,
                },
                {
                  message: "No foo!",
                  column: 0,
                },
              ],
            },
            {
              code: "let foo = 1;\nfoo = 4;",
              errors: [
                {
                  message: "No foo!",
                  endLine: 1,
                },
                {
                  message: "No foo!",
                  endLine: 2,
                },
              ],
            },
            {
              code: "let foo = 1;\nfoo = 5;",
              errors: [
                {
                  message: "No foo!",
                  endColumn: 7,
                },
                {
                  message: "No foo!",
                  endColumn: 3,
                },
              ],
            },
            {
              code: "let foo = 1;\nfoo = 6;",
              errors: [
                {
                  message: "No foo!",
                  line: 1,
                  column: 4,
                  endLine: 1,
                  endColumn: 7,
                },
                {
                  message: "No foo!",
                  line: 2,
                  column: 0,
                  endLine: 2,
                  endColumn: 3,
                },
              ],
            },
          ],
        });
        expect(runCases()).toEqual([null, null, null, null, null]);
      });
    });

    describe("which are incorrect", () => {
      it("with error count", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            { code: "let x;", errors: 1 },
            { code: "let foo;", errors: 2 },
          ],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Should have 1 error but had 0: []

          0 !== 1
          ],
            [AssertionError: Should have 2 errors but had 1: [
            {
              ruleId: 'rule-to-test/no-foo',
              message: 'No foo!',
              messageId: null,
              severity: 1,
              nodeType: 'Identifier',
              line: 1,
              column: 4,
              endLine: 1,
              endColumn: 7,
              fixes: null,
              suggestions: null
            }
          ]

          1 !== 2
          ],
          ]
        `);
      });

      it("with error messages", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            {
              code: "let foo;",
              errors: ["wrong message"],
            },
            {
              code: "foo.foo;",
              errors: ["again wrong", "so very wrong"],
            },
            {
              code: "let foo = 1;",
              errors: [/^NO foo!$/],
            },
            {
              code: "foo.foo = 2;",
              errors: [/wrong/, /so very wrong/],
            },
          ],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Expected values to be strictly equal:
          + actual - expected

          + 'No foo!'
          - 'wrong message'
          ],
            [AssertionError: Expected values to be strictly equal:
          + actual - expected

          + 'No foo!'
          - 'again wrong'
          ],
            [AssertionError: Expected 'No foo!' to match /^NO foo!$/],
            [AssertionError: Expected 'No foo!' to match /wrong/],
          ]
        `);
      });

      it("with error messages in objects", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            {
              code: "let foo;",
              errors: [{ message: "wrong message" }],
            },
            {
              code: "foo.foo;",
              errors: [{ message: "again wrong" }, { message: "so very wrong" }],
            },
            {
              code: "let foo = 1;",
              errors: [{ message: /^NO foo!$/ }],
            },
            {
              code: "foo.foo = 2;",
              errors: [{ message: /wrong/ }, { message: /so very wrong/ }],
            },
          ],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Expected values to be strictly equal:
          + actual - expected

          + 'No foo!'
          - 'wrong message'
          ],
            [AssertionError: Expected values to be strictly equal:
          + actual - expected

          + 'No foo!'
          - 'again wrong'
          ],
            [AssertionError: Expected 'No foo!' to match /^NO foo!$/],
            [AssertionError: Expected 'No foo!' to match /wrong/],
          ]
        `);
      });

      it("with message IDs and data", () => {
        const tester = new RuleTester();
        tester.run("no-foo", messageIdsRule, {
          valid: [],
          invalid: [
            // Wrong message ID
            {
              code: "foo",
              errors: [
                {
                  messageId: "noIdentifiers",
                },
              ],
            },
            {
              code: "bar",
              errors: [
                {
                  messageId: "noFoo",
                },
              ],
            },
            // Wrong data
            {
              code: "qux",
              errors: [
                {
                  messageId: "noIdentifiers",
                  data: { name: "not qux" },
                },
              ],
            },
            // Missing data
            {
              code: "whoosh",
              errors: [
                {
                  messageId: "noIdentifiers",
                  data: { x: "whoosh" },
                },
              ],
            },
            // Missing placeholder
            {
              code: "let bar",
              errors: [
                {
                  messageId: "noBar",
                },
              ],
            },
            {
              code: "bar = 1",
              errors: [
                {
                  messageId: "noBar",
                  data: { name: "bar" },
                },
              ],
            },
          ],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: messageId 'noFoo' does not match expected messageId 'noIdentifiers'
          + actual - expected

          + 'noFoo'
          - 'noIdentifiers'
               ^
          ],
            [AssertionError: messageId 'noBar' does not match expected messageId 'noFoo'

          'noBar' !== 'noFoo'
          ],
            [AssertionError: Hydrated message "No identifiers! not qux" does not match "No identifiers! qux"
          + actual - expected

          + 'No identifiers! qux'
          - 'No identifiers! not qux'
                             ^
          ],
            [AssertionError: Hydrated message "No identifiers! {{name}}" does not match "No identifiers! whoosh"
          + actual - expected

          + 'No identifiers! whoosh'
          - 'No identifiers! {{name}}'
                             ^
          ],
            [AssertionError: The reported message has an unsubstituted placeholder 'name'. Please provide the missing value via the \`data\` property.],
            [AssertionError: Hydrated message "No bar! bar" does not match "No bar! {{name}}"
          + actual - expected

          + 'No bar! {{name}}'
          - 'No bar! bar'
                     ^
          ],
          ]
        `);
      });

      it("with location", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            {
              code: "let foo = 1;",
              errors: [
                {
                  message: "No foo!",
                  line: 2,
                },
              ],
            },
            {
              code: "let foo = 2;",
              errors: [
                {
                  message: "No foo!",
                  column: 2,
                },
              ],
            },
            {
              code: "let foo = 3;",
              errors: [
                {
                  message: "No foo!",
                  endLine: 2,
                },
              ],
            },
            {
              code: "let foo = 4;",
              errors: [
                {
                  message: "No foo!",
                  endColumn: 4,
                },
              ],
            },
            {
              code: "let foo = 5;",
              errors: [
                {
                  message: "No foo!",
                  line: 2,
                  column: 2,
                  endLine: 3,
                  endColumn: 4,
                },
              ],
            },
          ],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Actual error location does not match expected error location.
          + actual - expected

            {
          +   line: 1
          -   line: 2
            }
          ],
            [AssertionError: Actual error location does not match expected error location.
          + actual - expected

            {
          +   column: 4
          -   column: 2
            }
          ],
            [AssertionError: Actual error location does not match expected error location.
          + actual - expected

            {
          +   endLine: 1
          -   endLine: 2
            }
          ],
            [AssertionError: Actual error location does not match expected error location.
          + actual - expected

            {
          +   endColumn: 7
          -   endColumn: 4
            }
          ],
            [AssertionError: Actual error location does not match expected error location.
          + actual - expected

            {
          +   column: 4,
          +   endColumn: 7,
          +   endLine: 1,
          +   line: 1
          -   column: 2,
          -   endColumn: 4,
          -   endLine: 3,
          -   line: 2
            }
          ],
          ]
        `);
      });

      it("duplicate code", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [],
          invalid: [
            {
              code: "let foo;",
              errors: 1,
            },
            {
              code: "let foo;",
              errors: 1,
            },
            {
              code: "let foo;",
              errors: 1,
            },
          ],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            null,
            [AssertionError: Detected duplicate test case],
            [AssertionError: Detected duplicate test case],
          ]
        `);
      });
    });

    describe("fixes", () => {
      // Rule which flags identifiers named "foo", and provides a fix to rename them to "bar"
      const fixRule: Rule = {
        meta: {
          fixable: "code",
        },
        create(context) {
          return {
            Identifier(node) {
              if (node.name !== "foo") return;

              context.report({
                message: "No foo!",
                node,
                fix(fixer) {
                  return fixer.replaceText(node, "bar");
                },
              });
            },
          };
        },
      };

      describe("which are correct", () => {
        it("with `output: null` when no fixes", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                output: null,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("without `output` when no fixes", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `output` matching fixed code", () => {
          const tester = new RuleTester();
          tester.run("no-foo", fixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                output: "let bar;",
                errors: 1,
              },
              {
                code: "let foo = foo;",
                output: "let bar = bar;",
                errors: 2,
              },
            ],
          });
          expect(runCases()).toEqual([null, null]);
        });
      });

      describe("which are incorrect", () => {
        it("with `output` not matching fixed code", () => {
          const tester = new RuleTester();
          tester.run("no-foo", fixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                output: "let qux;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Output is incorrect
          + actual - expected

          + 'let bar;'
          - 'let qux;'
                 ^
          ],
          ]
        `);
        });

        it("with `output: null` when there are fixes", () => {
          const tester = new RuleTester();
          tester.run("no-foo", fixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                output: null,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Expected no autofixes to be suggested
          + actual - expected

          + 'let bar;'
          - 'let foo;'
                 ^
          ],
          ]
        `);
        });

        it("without `output` when there are fixes", () => {
          const tester = new RuleTester();
          tester.run("no-foo", fixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: The rule fixed the code. Please add \`output\` property.
          + actual - expected

          + 'let bar;'
          - 'let foo;'
                 ^
          ],
          ]
        `);
        });

        it("with `output` same as `code` (fixable rule)", () => {
          const tester = new RuleTester();
          tester.run("no-foo", fixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                output: "let foo;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Output is incorrect
          + actual - expected

          + 'let bar;'
          - 'let foo;'
                 ^
          ],
          ]
        `);
        });

        it("with `output` same as `code` (non-fixable rule)", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                output: "let foo;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
          [
            [AssertionError: Test property \`output\` matches \`code\`. If no autofix is expected, set output to \`null\`.],
          ]
        `);
        });
      });

      describe("recursive", () => {
        // Rule which requires multiple fix passes. It removes trailing `_` from identifiers one at a time.
        // e.g. `foo__` -> `foo_` -> `foo`
        const multiPassFixRule: Rule = {
          meta: {
            fixable: "code",
          },
          create(context) {
            return {
              Identifier(node) {
                if (!node.name.endsWith("_")) return;

                context.report({
                  message: "No trailing underscore!",
                  node,
                  fix(fixer) {
                    return fixer.replaceText(node, node.name.slice(0, -1));
                  },
                });
              },
            };
          },
        };

        it("without `recursive` applies only one fix pass", () => {
          const tester = new RuleTester();
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo__;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `recursive: false`", () => {
          const tester = new RuleTester();
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo__;",
                recursive: false,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `recursive: 0`", () => {
          const tester = new RuleTester();
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo__;",
                recursive: 0,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `recursive: 1` applies two fix passes total", () => {
          const tester = new RuleTester();
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo_;",
                recursive: 1,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `recursive: 2` applies three fix passes total", () => {
          const tester = new RuleTester();
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo;",
                recursive: 2,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `recursive: true` applies all fix passes", () => {
          const tester = new RuleTester();
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo;",
                recursive: true,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({ recursive: true });
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("set globally", () => {
          RuleTester.setDefaultConfig({ recursive: true });
          const tester = new RuleTester();
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo;",
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("test case overrides `RuleTester` options", () => {
          const tester = new RuleTester({ recursive: true });
          tester.run("no-trailing-underscore", multiPassFixRule, {
            valid: [],
            invalid: [
              {
                code: "let foo___;",
                output: "let foo__;",
                recursive: false,
                errors: 1,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });
      });

      describe("adjacent fixes", () => {
        // Rule which produces two diagnostics with adjacent fix ranges for identifier `ab`.
        // Fix 1 replaces `a` (first char), fix 2 replaces `b` (second char).
        // End of fix 1's range === start of fix 2's range.
        //
        // In standard mode, adjacent fixes are not considered overlapping, so both apply.
        // In ESLint compat mode, adjacent fixes are considered overlapping (matching ESLint),
        // so only the first fix applies.
        const adjacentFixesRule: Rule = {
          meta: {
            fixable: "code",
          },
          create(context) {
            return {
              Identifier(node) {
                if (node.name !== "ab") return;

                const start = node.range[0];
                const mid = start + 1;
                const end = node.range[1];
                context.report({
                  message: "Fix a",
                  node,
                  fix(fixer) {
                    return fixer.replaceTextRange([start, mid], "x");
                  },
                });
                context.report({
                  message: "Fix b",
                  node,
                  fix(fixer) {
                    return fixer.replaceTextRange([mid, end], "y");
                  },
                });
              },
            };
          },
        };

        it("adjacent fixes applied in standard mode", () => {
          const tester = new RuleTester();
          tester.run("adjacent-fixes", adjacentFixesRule, {
            valid: [],
            invalid: [
              {
                code: "let ab;",
                output: "let xy;",
                errors: 2,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("only first adjacent fix applied in ESLint compat mode", () => {
          const tester = new RuleTester();
          tester.run("adjacent-fixes", adjacentFixesRule, {
            valid: [],
            invalid: [
              {
                code: "let ab;",
                output: "let xb;",
                eslintCompat: true,
                errors: 2,
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });
      });
    });

    describe("suggestions", () => {
      // Rule which suggests renaming `foo` to `bar` or `qux`
      const suggestRule: Rule = {
        meta: {
          hasSuggestions: true,
        },
        create(context) {
          return {
            Identifier(node) {
              if (node.name !== "foo") return;

              context.report({
                message: "No foo!",
                node,
                suggest: [
                  {
                    desc: "Rename to bar",
                    fix(fixer) {
                      return fixer.replaceText(node, "bar");
                    },
                  },
                  {
                    desc: "Rename to qux",
                    fix(fixer) {
                      return fixer.replaceText(node, "qux");
                    },
                  },
                ],
              });
            },
          };
        },
      };

      // Same as above rule, but using `messageId`s
      const suggestMessageIdRule: Rule = {
        meta: {
          hasSuggestions: true,
          messages: {
            noFoo: "No foo!",
            renameToBar: "Rename to bar",
            renameTo: "Rename to {{name}}",
          },
        },
        create(context) {
          return {
            Identifier(node) {
              if (node.name !== "foo") return;

              context.report({
                messageId: "noFoo",
                node,
                suggest: [
                  {
                    messageId: "renameToBar",
                    fix(fixer) {
                      return fixer.replaceText(node, "bar");
                    },
                  },
                  {
                    messageId: "renameTo",
                    data: { name: "qux" },
                    fix(fixer) {
                      return fixer.replaceText(node, "qux");
                    },
                  },
                ],
              });
            },
          };
        },
      };

      describe("which are correct", () => {
        it("with `desc`", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    message: "No foo!",
                    suggestions: [
                      { desc: "Rename to bar", output: "let bar;" },
                      { desc: "Rename to qux", output: "let qux;" },
                    ],
                  },
                ],
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `messageId`", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestMessageIdRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    messageId: "noFoo",
                    suggestions: [
                      { messageId: "renameToBar", output: "let bar;" },
                      { messageId: "renameTo", data: { name: "qux" }, output: "let qux;" },
                    ],
                  },
                ],
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("with `suggestions: null` when no suggestions", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    message: "No foo!",
                    suggestions: null,
                  },
                ],
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });

        it("without `suggestions` (not checked)", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [{ message: "No foo!" }],
              },
            ],
          });
          expect(runCases()).toEqual([null]);
        });
      });

      describe("which are incorrect", () => {
        it("wrong suggestion count", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    message: "No foo!",
                    suggestions: [{ desc: "Rename to bar", output: "let bar;" }],
                  },
                ],
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
              [
                [AssertionError: Error should have 1 suggestion. Instead found 2 suggestions.

              2 !== 1
              ],
              ]
            `);
        });

        it("wrong `desc`", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    message: "No foo!",
                    suggestions: [
                      { desc: "Wrong description", output: "let bar;" },
                      { desc: "Rename to qux", output: "let qux;" },
                    ],
                  },
                ],
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
              [
                [AssertionError: Suggestion at index 0: \`desc\` should be "Wrong description" but got "Rename to bar" instead
              + actual - expected

              + 'Rename to bar'
              - 'Wrong description'
              ],
              ]
            `);
        });

        it("wrong `output`", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    message: "No foo!",
                    suggestions: [
                      { desc: "Rename to bar", output: "let qux;" },
                      { desc: "Rename to qux", output: "let qux;" },
                    ],
                  },
                ],
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
              [
                [AssertionError: Suggestion at index 0: Expected the applied suggestion fix to match the test suggestion output
              + actual - expected

              + 'let bar;'
              - 'let qux;'
                     ^
              ],
              ]
            `);
        });

        it("`output` same as original source code", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    message: "No foo!",
                    suggestions: [
                      { desc: "Rename to bar", output: "let foo;" },
                      { desc: "Rename to qux", output: "let qux;" },
                    ],
                  },
                ],
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
              [
                [AssertionError: Suggestion at index 0: Expected the applied suggestion fix to match the test suggestion output
              + actual - expected

              + 'let bar;'
              - 'let foo;'
                     ^
              ],
              ]
            `);
        });

        it("wrong `messageId`", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestMessageIdRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    messageId: "noFoo",
                    suggestions: [
                      { messageId: "renameTo", output: "let bar;" },
                      { messageId: "renameTo", data: { name: "qux" }, output: "let qux;" },
                    ],
                  },
                ],
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
              [
                [AssertionError: Suggestion at index 0: messageId 'renameToBar' does not match expected messageId 'renameTo'
              + actual - expected

              + 'renameToBar'
              - 'renameTo'
                         ^
              ],
              ]
            `);
        });

        it("`suggestions: null` when rule has suggestions", () => {
          const tester = new RuleTester();
          tester.run("no-foo", suggestRule, {
            valid: [],
            invalid: [
              {
                code: "let foo;",
                errors: [
                  {
                    message: "No foo!",
                    suggestions: null,
                  },
                ],
              },
            ],
          });
          expect(runCases()).toMatchInlineSnapshot(`
              [
                [AssertionError: Rule produced suggestions],
              ]
            `);
        });
      });
    });
  });

  it("errors when parsing failure", () => {
    const tester = new RuleTester();
    tester.run("no-foo", simpleRule, {
      valid: ["let"],
      invalid: [{ code: "let", errors: 1 }],
    });
    expect(runCases()).toMatchInlineSnapshot(`
      [
        [Error: Parsing failed],
        [Error: Parsing failed],
      ]
    `);
  });

  it("runs `before` and `after` hooks", () => {
    const validTests = [
      // Passes
      {
        code: "let x;",
        before: vi.fn(),
        after: vi.fn(),
      },
      // Fails
      {
        code: "let foo;",
        before: vi.fn(),
        after: vi.fn(),
      },
    ];

    const invalidTests = [
      // Passes
      {
        code: "let foo;",
        before: vi.fn(),
        after: vi.fn(),
        errors: 1,
      },
      // Fails
      {
        code: "let x;",
        before: vi.fn(),
        after: vi.fn(),
        errors: 2,
      },
    ];

    const tester = new RuleTester();
    tester.run("no-foo", simpleRule, {
      valid: validTests,
      invalid: invalidTests,
    });
    runCases();

    for (const test of [...validTests, ...invalidTests]) {
      expect(test.before).toHaveBeenCalledExactlyOnceWith();
      expect(test.before.mock.contexts[0]).toBe(test);
      expect(test.after).toHaveBeenCalledExactlyOnceWith();
      expect(test.after.mock.contexts[0]).toBe(test);
    }
  });

  describe("ESLint compat mode", () => {
    it("enabled globally", () => {
      RuleTester.setDefaultConfig({ eslintCompat: true });

      const tester = new RuleTester();
      tester.run("no-foo", simpleRule, {
        valid: [],
        invalid: [
          {
            code: "foo;",
            errors: [
              {
                message: "No foo!",
                line: 1,
                column: 1,
                endLine: 1,
                endColumn: 4,
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    it("enabled in `RuleTester` options", () => {
      const tester = new RuleTester({ eslintCompat: true });
      tester.run("no-foo", simpleRule, {
        valid: [],
        invalid: [
          {
            code: "foo;",
            errors: [
              {
                message: "No foo!",
                line: 1,
                column: 1,
                endLine: 1,
                endColumn: 4,
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    it("enabled in in individual test cases", () => {
      const tester = new RuleTester();
      tester.run("no-foo", simpleRule, {
        valid: [],
        invalid: [
          {
            code: "foo = 1;",
            // Default: eslintCompat: false
            errors: [
              {
                message: "No foo!",
                line: 1,
                column: 0,
                endLine: 1,
                endColumn: 3,
              },
            ],
          },
          {
            code: "foo = 1;",
            eslintCompat: false,
            errors: [
              {
                message: "No foo!",
                line: 1,
                column: 0,
                endLine: 1,
                endColumn: 3,
              },
            ],
          },
          {
            code: "foo = 1;",
            eslintCompat: true,
            errors: [
              {
                message: "No foo!",
                line: 1,
                column: 1, // 1 not 0
                endLine: 1,
                endColumn: 4, // 4 not 3
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null, null, null]);
    });
  });

  describe("parsing options", () => {
    describe("sourceType", () => {
      describe("default", () => {
        const reportSourceTypeRule: Rule = {
          create(context) {
            return {
              Program(node) {
                context.report({
                  message: `sourceType: ${context.languageOptions.sourceType}`,
                  node,
                });
              },
            };
          },
        };

        it("unambiguous without ESLint compatibility mode", () => {
          const tester = new RuleTester();
          tester.run("source-type", reportSourceTypeRule, {
            valid: [],
            invalid: [
              // No ESM syntax, parsed as script, so `with` is allowed
              {
                code: "with (obj) {}",
                errors: ["sourceType: script"],
              },
              // Has ESM syntax, parsed as module
              {
                code: "import x from 'foo';",
                errors: ["sourceType: module"],
              },
            ],
          });
          expect(runCases()).toEqual([null, null]);
        });

        it("module with ESLint compatibility mode", () => {
          const tester = new RuleTester({ eslintCompat: true });
          tester.run("source-type", reportSourceTypeRule, {
            // Parsed as module, `with` is not allowed, so parse error
            valid: ["with (obj) {}"],
            // Has ESM syntax, successfully parsed as module
            invalid: [
              {
                code: "import x from 'foo';",
                errors: ["sourceType: module"],
              },
            ],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
              null,
            ]
          `);
        });
      });

      describe("module", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { sourceType: "module" },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: ["with (obj) {}", "import x from 'foo';"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
              null,
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["with (obj) {}", "import x from 'foo';"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
              null,
            ]
          `);
        });

        it("set in `RuleTester` options, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { sourceType: "script" },
          });

          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["with (obj) {}", "import x from 'foo';"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
              null,
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "with (obj) {}",
                languageOptions: { sourceType: "module" },
              },
              {
                code: "import x from 'foo';",
                languageOptions: { sourceType: "module" },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
              null,
            ]
          `);
        });

        it("set in individual test cases, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { sourceType: "script" },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "with (obj) {}",
                languageOptions: { sourceType: "module" },
              },
              {
                code: "import x from 'foo';",
                languageOptions: { sourceType: "module" },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
              null,
            ]
          `);
        });

        it("set in individual test cases, overriding `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { sourceType: "script" },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "with (obj) {}",
                languageOptions: { sourceType: "module" },
              },
              {
                code: "import x from 'foo';",
                languageOptions: { sourceType: "module" },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
              null,
            ]
          `);
        });
      });

      describe("script", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { sourceType: "script" },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: ["with (obj) {}", "import x from 'foo';"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { sourceType: "script" },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["with (obj) {}", "import x from 'foo';"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "with (obj) {}",
                languageOptions: { sourceType: "script" },
              },
              {
                code: "import x from 'foo';",
                languageOptions: { sourceType: "script" },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("commonjs", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { sourceType: "commonjs" },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: ["with (obj) {}", "return 123;", "import x from 'foo';"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { sourceType: "commonjs" },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["with (obj) {}", "return 123;", "import x from 'foo';"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "with (obj) {}",
                languageOptions: { sourceType: "commonjs" },
              },
              {
                code: "return 123;",
                languageOptions: { sourceType: "commonjs" },
              },
              {
                code: "import x from 'foo';",
                languageOptions: { sourceType: "commonjs" },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("unambiguous", () => {
        describe("with `eslintCompat` option", () => {
          it("set globally", () => {
            RuleTester.setDefaultConfig({
              languageOptions: { sourceType: "unambiguous" },
              eslintCompat: true,
            });

            const tester = new RuleTester();
            tester.run("no-foo", simpleRule, {
              valid: ["with (obj) {}", "import x from 'foo';"],
              invalid: [],
            });

            expect(runCases()).toMatchInlineSnapshot(`
              [
                [Error: 'unambiguous' source type is not supported in ESLint compatibility mode.
              Disable ESLint compatibility mode by setting \`eslintCompat\` to \`false\` in the config / test case.],
                [Error: 'unambiguous' source type is not supported in ESLint compatibility mode.
              Disable ESLint compatibility mode by setting \`eslintCompat\` to \`false\` in the config / test case.],
              ]
            `);
          });

          it("set in `RuleTester` options", () => {
            const tester = new RuleTester({
              languageOptions: { sourceType: "unambiguous" },
              eslintCompat: true,
            });
            tester.run("no-foo", simpleRule, {
              valid: ["with (obj) {}", "import x from 'foo';"],
              invalid: [],
            });

            expect(runCases()).toMatchInlineSnapshot(`
              [
                [Error: 'unambiguous' source type is not supported in ESLint compatibility mode.
              Disable ESLint compatibility mode by setting \`eslintCompat\` to \`false\` in the config / test case.],
                [Error: 'unambiguous' source type is not supported in ESLint compatibility mode.
              Disable ESLint compatibility mode by setting \`eslintCompat\` to \`false\` in the config / test case.],
              ]
            `);
          });

          it("set in individual test cases", () => {
            const tester = new RuleTester();
            tester.run("no-foo", simpleRule, {
              valid: [
                {
                  code: "with (obj) {}",
                  languageOptions: { sourceType: "unambiguous" },
                  eslintCompat: true,
                },
                {
                  code: "import x from 'foo';",
                  languageOptions: { sourceType: "unambiguous" },
                  eslintCompat: true,
                },
              ],
              invalid: [],
            });

            expect(runCases()).toMatchInlineSnapshot(`
              [
                [Error: 'unambiguous' source type is not supported in ESLint compatibility mode.
              Disable ESLint compatibility mode by setting \`eslintCompat\` to \`false\` in the config / test case.],
                [Error: 'unambiguous' source type is not supported in ESLint compatibility mode.
              Disable ESLint compatibility mode by setting \`eslintCompat\` to \`false\` in the config / test case.],
              ]
            `);
          });
        });

        describe("without `eslintCompat` option", () => {
          it("set globally", () => {
            RuleTester.setDefaultConfig({
              languageOptions: { sourceType: "unambiguous" },
            });

            const tester = new RuleTester();
            tester.run("no-foo", simpleRule, {
              valid: ["with (obj) {}", "import x from 'foo';"],
              invalid: [],
            });

            expect(runCases()).toEqual([null, null]);
          });

          it("set in `RuleTester` options", () => {
            const tester = new RuleTester({
              languageOptions: { sourceType: "unambiguous" },
            });
            tester.run("no-foo", simpleRule, {
              valid: ["with (obj) {}", "import x from 'foo';"],
              invalid: [],
            });

            expect(runCases()).toEqual([null, null]);
          });

          it("set in individual test cases", () => {
            const tester = new RuleTester();
            tester.run("no-foo", simpleRule, {
              valid: [
                {
                  code: "with (obj) {}",
                  languageOptions: { sourceType: "unambiguous" },
                },
                {
                  code: "import x from 'foo';",
                  languageOptions: { sourceType: "unambiguous" },
                },
              ],
              invalid: [],
            });

            expect(runCases()).toEqual([null, null]);
          });
        });
      });

      it("mixed across test cases", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [
            // Default = unambiguous
            "with (obj) {}",
            "import x from 'foo';",
            {
              code: "with (obj) {}",
              languageOptions: { sourceType: "script" },
            },
            {
              code: "import x from 'foo';",
              languageOptions: { sourceType: "module" },
            },
            {
              code: "with (obj) {}",
              languageOptions: { sourceType: "module" },
            },
            {
              code: "import x from 'foo';",
              languageOptions: { sourceType: "script" },
            },
            {
              code: "with (obj) {}",
              languageOptions: { sourceType: "unambiguous" },
            },
            {
              code: "import x from 'foo';",
              languageOptions: { sourceType: "unambiguous" },
            },
          ],
          invalid: [],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            null,
            null,
            null,
            null,
            [Error: Parsing failed],
            [Error: Parsing failed],
            null,
            null,
          ]
        `);
      });
    });

    describe("lang", () => {
      it("default (js)", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [
            "let x;",
            "<div/>",
            "let x: number;",
            "let x: T = <div/>;",
            "class C { f(): void }",
          ],
          invalid: [],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            null,
            [Error: Parsing failed],
            [Error: Parsing failed],
            [Error: Parsing failed],
            [Error: Parsing failed],
          ]
        `);
      });

      describe("js", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "js" } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "js" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "jsx" } },
          });

          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "js" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "let x: number;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "let x: T = <div/>;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "class C { f(): void }",
                languageOptions: { parserOptions: { lang: "js" } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "jsx" } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "let x: number;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "let x: T = <div/>;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "class C { f(): void }",
                languageOptions: { parserOptions: { lang: "js" } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases, overriding `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "jsx" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "let x: number;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "let x: T = <div/>;",
                languageOptions: { parserOptions: { lang: "js" } },
              },
              {
                code: "class C { f(): void }",
                languageOptions: { parserOptions: { lang: "js" } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("jsx", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "jsx" } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "jsx" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "jsx" } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "jsx" } },
              },
              {
                code: "let x: number;",
                languageOptions: { parserOptions: { lang: "jsx" } },
              },
              {
                code: "let x: T = <div/>;",
                languageOptions: { parserOptions: { lang: "jsx" } },
              },
              {
                code: "class C { f(): void }",
                languageOptions: { parserOptions: { lang: "jsx" } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("ts", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "ts" } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "ts" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "ts" } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "ts" } },
              },
              {
                code: "let x: number;",
                languageOptions: { parserOptions: { lang: "ts" } },
              },
              {
                code: "let x: T = <div/>;",
                languageOptions: { parserOptions: { lang: "ts" } },
              },
              {
                code: "class C { f(): void }",
                languageOptions: { parserOptions: { lang: "ts" } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              null,
              [Error: Parsing failed],
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("tsx", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "tsx" } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              null,
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "tsx" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              null,
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "tsx" } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "tsx" } },
              },
              {
                code: "let x: number;",
                languageOptions: { parserOptions: { lang: "tsx" } },
              },
              {
                code: "let x: T = <div/>;",
                languageOptions: { parserOptions: { lang: "tsx" } },
              },
              {
                code: "class C { f(): void }",
                languageOptions: { parserOptions: { lang: "tsx" } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              null,
              null,
              null,
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("dts", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "dts" } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              null,
              [Error: Parsing failed],
              null,
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "dts" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              "let x;",
              "<div/>",
              "let x: number;",
              "let x: T = <div/>;",
              "class C { f(): void }",
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              null,
              [Error: Parsing failed],
              null,
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "dts" } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "dts" } },
              },
              {
                code: "let x: number;",
                languageOptions: { parserOptions: { lang: "dts" } },
              },
              {
                code: "let x: T = <div/>;",
                languageOptions: { parserOptions: { lang: "dts" } },
              },
              {
                code: "class C { f(): void }",
                languageOptions: { parserOptions: { lang: "dts" } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
              null,
              [Error: Parsing failed],
              null,
            ]
          `);
        });
      });

      it("mixed across test cases", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [
            // Default (js)
            "let x;",
            "<div/>",
            "let x: number;",
            "let x: T = <div/>;",
            "class C { f(): void }",
            // JS
            {
              code: "let x;",
              languageOptions: { parserOptions: { lang: "js" } },
            },
            {
              code: "<div/>",
              languageOptions: { parserOptions: { lang: "js" } },
            },
            {
              code: "let x: number;",
              languageOptions: { parserOptions: { lang: "js" } },
            },
            {
              code: "let x: T = <div/>;",
              languageOptions: { parserOptions: { lang: "js" } },
            },
            {
              code: "class C { f(): void }",
              languageOptions: { parserOptions: { lang: "js" } },
            },
            // JSX
            {
              code: "let x;",
              languageOptions: { parserOptions: { lang: "jsx" } },
            },
            {
              code: "<div/>",
              languageOptions: { parserOptions: { lang: "jsx" } },
            },
            {
              code: "let x: number;",
              languageOptions: { parserOptions: { lang: "jsx" } },
            },
            {
              code: "let x: T = <div/>;",
              languageOptions: { parserOptions: { lang: "jsx" } },
            },
            {
              code: "class C { f(): void }",
              languageOptions: { parserOptions: { lang: "jsx" } },
            },
            // TS
            {
              code: "let x;",
              languageOptions: { parserOptions: { lang: "ts" } },
            },
            {
              code: "<div/>",
              languageOptions: { parserOptions: { lang: "ts" } },
            },
            {
              code: "let x: number;",
              languageOptions: { parserOptions: { lang: "ts" } },
            },
            {
              code: "let x: T = <div/>;",
              languageOptions: { parserOptions: { lang: "ts" } },
            },
            {
              code: "class C { f(): void }",
              languageOptions: { parserOptions: { lang: "ts" } },
            },
            // TSX
            {
              code: "let x;",
              languageOptions: { parserOptions: { lang: "tsx" } },
            },
            {
              code: "<div/>",
              languageOptions: { parserOptions: { lang: "tsx" } },
            },
            {
              code: "let x: number;",
              languageOptions: { parserOptions: { lang: "tsx" } },
            },
            {
              code: "let x: T = <div/>;",
              languageOptions: { parserOptions: { lang: "tsx" } },
            },
            {
              code: "class C { f(): void }",
              languageOptions: { parserOptions: { lang: "tsx" } },
            },
            // DTS
            {
              code: "let x;",
              languageOptions: { parserOptions: { lang: "dts" } },
            },
            {
              code: "<div/>",
              languageOptions: { parserOptions: { lang: "dts" } },
            },
            {
              code: "let x: number;",
              languageOptions: { parserOptions: { lang: "dts" } },
            },
            {
              code: "let x: T = <div/>;",
              languageOptions: { parserOptions: { lang: "dts" } },
            },
            {
              code: "class C { f(): void }",
              languageOptions: { parserOptions: { lang: "dts" } },
            },
          ],
          invalid: [],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            null,
            [Error: Parsing failed],
            [Error: Parsing failed],
            [Error: Parsing failed],
            [Error: Parsing failed],
            null,
            [Error: Parsing failed],
            [Error: Parsing failed],
            [Error: Parsing failed],
            [Error: Parsing failed],
            null,
            null,
            [Error: Parsing failed],
            [Error: Parsing failed],
            [Error: Parsing failed],
            null,
            [Error: Parsing failed],
            null,
            [Error: Parsing failed],
            [Error: Parsing failed],
            null,
            null,
            null,
            null,
            [Error: Parsing failed],
            null,
            [Error: Parsing failed],
            null,
            [Error: Parsing failed],
            null,
          ]
        `);
      });

      it("is overridden by `filename`", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [
            {
              code: "let x: number;",
              filename: "foo.ts",
              languageOptions: { parserOptions: { lang: "js" } },
            },
            {
              code: "<div />",
              filename: "foo.jsx",
              languageOptions: { parserOptions: { lang: "ts" } },
            },
          ],
          invalid: [
            {
              code: "let x: number;",
              filename: "foo.jsx",
              languageOptions: { parserOptions: { lang: "ts" } },
              errors: 1,
            },
            {
              code: "<div />",
              filename: "foo.ts",
              languageOptions: { parserOptions: { lang: "jsx" } },
              errors: 1,
            },
          ],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            null,
            null,
            [Error: Parsing failed],
            [Error: Parsing failed],
          ]
        `);
      });
    });

    describe("ecmaFeatures.jsx", () => {
      it("default (false)", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: ["let x;", "<div/>"],
          invalid: [],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            null,
            [Error: Parsing failed],
          ]
        `);
      });

      describe("false", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: ["let x;", "<div/>"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["let x;", "<div/>"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
          });

          const tester = new RuleTester({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["let x;", "<div/>"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases, overriding `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("true", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: ["let x;", "<div/>"],
            invalid: [],
          });

          expect(runCases()).toEqual([null, null]);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["let x;", "<div/>"],
            invalid: [],
          });

          expect(runCases()).toEqual([null, null]);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toEqual([null, null]);
        });
      });

      it("mixed across test cases", () => {
        const tester = new RuleTester();
        tester.run("no-foo", simpleRule, {
          valid: [
            "let x;",
            "<div/>",
            {
              code: "let x;",
              languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
            },
            {
              code: "<div/>",
              languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
            },
            {
              code: "let x;",
              languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
            },
            {
              code: "<div/>",
              languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
            },
          ],
          invalid: [],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            null,
            [Error: Parsing failed],
            null,
            [Error: Parsing failed],
            null,
            null,
          ]
        `);
      });

      describe("does not take priority over `lang` option", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { lang: "js", ecmaFeatures: { jsx: true } } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: ["let x;", "<div/>"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "js", ecmaFeatures: { jsx: true } } },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["let x;", "<div/>"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { lang: "js", ecmaFeatures: { jsx: true } } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { lang: "js", ecmaFeatures: { jsx: true } } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases with `lang` defined in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { lang: "js" } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "let x;",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
              },
              {
                code: "<div/>",
                languageOptions: { parserOptions: { ecmaFeatures: { jsx: true } } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              null,
              [Error: Parsing failed],
            ]
          `);
        });
      });
    });

    describe("ignoreNonFatalErrors", () => {
      it("default (off)", () => {
        const tester = new RuleTester({
          languageOptions: { sourceType: "module" },
        });
        tester.run("no-foo", simpleRule, {
          valid: ["function f(x, x) {}"],
          invalid: [],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            [Error: Parsing failed],
          ]
        `);
      });

      describe("disabled", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: {
              parserOptions: { ignoreNonFatalErrors: false },
            },
          });

          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: {
              sourceType: "module",
              parserOptions: { ignoreNonFatalErrors: false },
            },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in `RuleTester` options, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: {
              parserOptions: { ignoreNonFatalErrors: true },
            },
          });

          const tester = new RuleTester({
            languageOptions: {
              sourceType: "module",
              parserOptions: { ignoreNonFatalErrors: false },
            },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: {
                  parserOptions: { ignoreNonFatalErrors: false },
                },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: {
              parserOptions: { ignoreNonFatalErrors: true },
            },
          });

          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: {
                  parserOptions: { ignoreNonFatalErrors: false },
                },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
            ]
          `);
        });

        it("set in individual test cases, overriding `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: {
              sourceType: "module",
              parserOptions: { ignoreNonFatalErrors: true },
            },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: {
                  parserOptions: { ignoreNonFatalErrors: false },
                },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toMatchInlineSnapshot(`
            [
              [Error: Parsing failed],
            ]
          `);
        });
      });

      describe("enabled", () => {
        it("set globally", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
          });

          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: {
              sourceType: "module",
              parserOptions: { ignoreNonFatalErrors: true },
            },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });

        it("set in `RuleTester` options, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
          });

          const tester = new RuleTester({
            languageOptions: {
              sourceType: "module",
              parserOptions: { ignoreNonFatalErrors: true },
            },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });

        it("set in individual test cases, overriding global setting", () => {
          RuleTester.setDefaultConfig({
            languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
          });

          const tester = new RuleTester({
            languageOptions: { sourceType: "module" },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });

        it("set in individual test cases, overriding `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: {
              sourceType: "module",
              parserOptions: { ignoreNonFatalErrors: false },
            },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
              },
            ],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });
      });

      it("mixed across test cases", () => {
        const tester = new RuleTester({
          languageOptions: { sourceType: "module" },
        });
        tester.run("no-foo", simpleRule, {
          valid: [
            "function f(x, x) {}",
            {
              code: "function f(x, x) {}",
              languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
            },
            {
              code: "function f(x, x) {}",
              languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
            },
          ],
          invalid: [],
        });

        expect(runCases()).toMatchInlineSnapshot(`
          [
            [Error: Parsing failed],
            [Error: Parsing failed],
            null,
          ]
        `);
      });
    });
  });

  describe("globals", () => {
    const globalReporterRule: Rule = {
      create(context) {
        return {
          Program(node) {
            context.report({
              message: `globals: ${JSON.stringify(context.languageOptions.globals)}`,
              node,
            });
          },
        };
      },
    };

    it("is empty object if no globals defined", () => {
      const tester = new RuleTester();
      tester.run("no-foo", globalReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            errors: [
              {
                message: "globals: {}",
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    describe("set", () => {
      it("globally", () => {
        RuleTester.setDefaultConfig({
          languageOptions: {
            globals: {
              read: "readonly",
              write: "writable",
              disabled: "off",
            },
          },
        });

        const tester = new RuleTester();
        tester.run("no-foo", globalReporterRule, {
          valid: [],
          invalid: [
            {
              code: "",
              errors: [
                {
                  message: 'globals: {"read":"readonly","write":"writable","disabled":"off"}',
                },
              ],
            },
          ],
        });
        expect(runCases()).toEqual([null]);
      });

      it("in `RuleTester` options", () => {
        const tester = new RuleTester({
          languageOptions: {
            globals: {
              read: "readonly",
              write: "writable",
              disabled: "off",
            },
          },
        });

        tester.run("no-foo", globalReporterRule, {
          valid: [],
          invalid: [
            {
              code: "",
              errors: [
                {
                  message: 'globals: {"read":"readonly","write":"writable","disabled":"off"}',
                },
              ],
            },
          ],
        });
        expect(runCases()).toEqual([null]);
      });

      it("in test case", () => {
        const tester = new RuleTester();
        tester.run("no-foo", globalReporterRule, {
          valid: [],
          invalid: [
            {
              code: "",
              languageOptions: {
                globals: {},
              },
              errors: [
                {
                  message: "globals: {}",
                },
              ],
            },
            {
              code: "",
              languageOptions: {
                globals: {
                  read: "readonly",
                  write: "writable",
                  disabled: "off",
                },
              },
              errors: [
                {
                  message: 'globals: {"read":"readonly","write":"writable","disabled":"off"}',
                },
              ],
            },
          ],
        });
        expect(runCases()).toEqual([null, null]);
      });
    });

    it("merged between global config, config, and test case", () => {
      RuleTester.setDefaultConfig({
        languageOptions: {
          globals: {
            globalConfig: "readonly",
            globalConfigOverriddenByConfig: "readonly",
            globalConfigOverriddenByTestCase: "readonly",
            globalConfigOverriddenByBoth: "readonly",
          },
        },
      });

      const tester = new RuleTester({
        languageOptions: {
          globals: {
            config: "writable",
            globalConfigOverriddenByConfig: "writable",
            globalConfigOverriddenByBoth: "writable",
            configOverriddenByTestCase: "writable",
          },
        },
      });

      tester.run("no-foo", globalReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            languageOptions: {
              globals: {
                testCase: "off",
                globalConfigOverriddenByTestCase: "off",
                globalConfigOverriddenByBoth: "off",
                configOverriddenByTestCase: "off",
              },
            },
            errors: [
              {
                message: `globals: ${JSON.stringify({
                  globalConfig: "readonly",
                  globalConfigOverriddenByConfig: "writable",
                  globalConfigOverriddenByTestCase: "off",
                  globalConfigOverriddenByBoth: "off",
                  config: "writable",
                  configOverriddenByTestCase: "off",
                  testCase: "off",
                })}`,
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    it("normalizes values", () => {
      const tester = new RuleTester();
      tester.run("no-foo", globalReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            eslintCompat: true,
            languageOptions: {
              globals: {
                writable: "writable",
                writeable: "writeable",
                true: true,
                trueStr: "true",
                readonly: "readonly",
                readable: "readable",
                false: false,
                falseStr: "false",
                null: null,
                off: "off",
              },
            },
            errors: [
              {
                message: `globals: ${JSON.stringify({
                  writable: "writable",
                  writeable: "writable",
                  true: "writable",
                  trueStr: "writable",
                  readonly: "readonly",
                  readable: "readonly",
                  false: "readonly",
                  falseStr: "readonly",
                  null: "readonly",
                  off: "off",
                })}`,
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    describe("throws on invalid values", () => {
      it("other string", () => {
        const tester = new RuleTester();
        tester.run("no-foo", globalReporterRule, {
          valid: [
            {
              code: "",
              languageOptions: {
                globals: {
                  // @ts-expect-error - intentionally invalid value
                  invalid: "invalid",
                },
              },
            },
          ],
          invalid: [],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            [Error: 'invalid' is not a valid configuration for a global (use 'readonly', 'writable', or 'off')],
          ]
        `);
      });

      it("`null` when not in ESLint compatibility mode", () => {
        // Note: `null` being accepted in ESLint compatibility mode is tested above
        const tester = new RuleTester();
        tester.run("no-foo", globalReporterRule, {
          valid: [
            {
              code: "",
              languageOptions: {
                globals: {
                  null: null,
                },
              },
            },
          ],
          invalid: [],
        });
        expect(runCases()).toMatchInlineSnapshot(`
          [
            [Error: 'null' is not a valid configuration for a global (use 'readonly', 'writable', or 'off')],
          ]
        `);
      });
    });
  });

  describe("settings", () => {
    const settingsReporterRule: Rule = {
      create(context) {
        return {
          Program(node) {
            context.report({
              message: `settings: ${JSON.stringify(context.settings)}`,
              node,
            });
          },
        };
      },
    };

    it("is empty object if no settings defined", () => {
      const tester = new RuleTester();
      tester.run("no-foo", settingsReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            errors: [
              {
                message: "settings: {}",
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    it("reflects defined settings", () => {
      const tester = new RuleTester();
      tester.run("no-foo", settingsReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            settings: { foo: 123, nested: { bar: true, qux: null } },
            errors: [
              {
                message: 'settings: {"foo":123,"nested":{"bar":true,"qux":null}}',
              },
            ],
          },
          {
            code: "",
            settings: { x: "y" },
            errors: [
              {
                message: 'settings: {"x":"y"}',
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null, null]);
    });
  });

  describe("`cwd` option", () => {
    const cwdReporterRule: Rule = {
      create(context) {
        return {
          Program(node) {
            context.report({
              message: `cwd: ${context.cwd}`,
              node,
            });
          },
        };
      },
    };

    it("set globally", () => {
      RuleTester.setDefaultConfig({ cwd: "/a/b/c" });

      const tester = new RuleTester();
      tester.run("cwd", cwdReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            errors: [
              {
                message: "cwd: /a/b/c",
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    it("set in `RuleTester` options", () => {
      const tester = new RuleTester({ cwd: "/a/b/c" });
      tester.run("cwd", cwdReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            errors: [
              {
                message: "cwd: /a/b/c",
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });

    it("set in test case", () => {
      const tester = new RuleTester();
      tester.run("cwd", cwdReporterRule, {
        valid: [],
        invalid: [
          {
            code: "",
            cwd: "/a/b/c",
            errors: [
              {
                message: "cwd: /a/b/c",
              },
            ],
          },
        ],
      });
      expect(runCases()).toEqual([null]);
    });
  });
});
