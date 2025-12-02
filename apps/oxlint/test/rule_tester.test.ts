import { beforeEach, describe, expect, it, vi } from "vitest";
import { RuleTester } from "../src-js/index.ts";

import type { Rule } from "../src-js/index.ts";

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
  const errors = [];
  for (const testCase of cases) {
    let error = null;
    try {
      testCase.fn();
    } catch (err) {
      error = err;
    }
    errors.push(error);
  }
  return errors;
}

const simpleRule: Rule = {
  create(context) {
    return {
      Identifier(node) {
        if (node.name === "foo") context.report({ message: "No foo!", node });
      },
    };
  },
};

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
          options: [],
        },
        {
          name: "valid case name",
          code: "let x = 1;",
          options: [],
        },
      ],
      invalid: [
        {
          code: "invalid code from object",
          options: [],
          errors: 1,
        },
        {
          name: "invalid case name",
          code: "let x = 1;",
          options: [],
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
            [AssertionError: The reported message has an unsubstituted placeholder 'name'. Please provide the missing value via the \`data\` property on the error object.],
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
});
