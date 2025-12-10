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
      it("default (module)", () => {
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
        describe("with `eslintCompat` option", () => {
          it("set globally", () => {
            RuleTester.setDefaultConfig({
              languageOptions: { sourceType: "commonjs" },
              eslintCompat: true,
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
              languageOptions: { sourceType: "commonjs" },
              eslintCompat: true,
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
                  languageOptions: { sourceType: "commonjs" },
                  eslintCompat: true,
                },
                {
                  code: "import x from 'foo';",
                  languageOptions: { sourceType: "commonjs" },
                  eslintCompat: true,
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

        describe("without `eslintCompat` option", () => {
          it("set globally", () => {
            RuleTester.setDefaultConfig({
              languageOptions: { sourceType: "commonjs" },
            });

            const tester = new RuleTester();
            tester.run("no-foo", simpleRule, {
              valid: ["with (obj) {}", "import x from 'foo';"],
              invalid: [],
            });

            expect(runCases()).toMatchInlineSnapshot(`
              [
                [Error: 'commonjs' source type is only supported in ESLint compatibility mode.
              Enable ESLint compatibility mode by setting \`eslintCompat\` to \`true\` in the config / test case.],
                [Error: 'commonjs' source type is only supported in ESLint compatibility mode.
              Enable ESLint compatibility mode by setting \`eslintCompat\` to \`true\` in the config / test case.],
              ]
            `);
          });

          it("set in `RuleTester` options", () => {
            const tester = new RuleTester({
              languageOptions: { sourceType: "commonjs" },
            });
            tester.run("no-foo", simpleRule, {
              valid: ["with (obj) {}", "import x from 'foo';"],
              invalid: [],
            });

            expect(runCases()).toMatchInlineSnapshot(`
              [
                [Error: 'commonjs' source type is only supported in ESLint compatibility mode.
              Enable ESLint compatibility mode by setting \`eslintCompat\` to \`true\` in the config / test case.],
                [Error: 'commonjs' source type is only supported in ESLint compatibility mode.
              Enable ESLint compatibility mode by setting \`eslintCompat\` to \`true\` in the config / test case.],
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
                  code: "import x from 'foo';",
                  languageOptions: { sourceType: "commonjs" },
                },
              ],
              invalid: [],
            });

            expect(runCases()).toMatchInlineSnapshot(`
              [
                [Error: 'commonjs' source type is only supported in ESLint compatibility mode.
              Enable ESLint compatibility mode by setting \`eslintCompat\` to \`true\` in the config / test case.],
                [Error: 'commonjs' source type is only supported in ESLint compatibility mode.
              Enable ESLint compatibility mode by setting \`eslintCompat\` to \`true\` in the config / test case.],
              ]
            `);
          });
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
            "with (obj) {}",
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
            [Error: Parsing failed],
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
        const tester = new RuleTester();
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
            languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
          });

          const tester = new RuleTester();
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
            languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
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
            languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
          });

          const tester = new RuleTester({
            languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
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
          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
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
            languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
          });

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
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
            languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
          });
          tester.run("no-foo", simpleRule, {
            valid: [
              {
                code: "function f(x, x) {}",
                languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
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

          const tester = new RuleTester();
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });

        it("set in `RuleTester` options", () => {
          const tester = new RuleTester({
            languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
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
            languageOptions: { parserOptions: { ignoreNonFatalErrors: true } },
          });
          tester.run("no-foo", simpleRule, {
            valid: ["function f(x, x) {}"],
            invalid: [],
          });

          expect(runCases()).toEqual([null]);
        });

        it("set in individual test cases", () => {
          const tester = new RuleTester();
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

          const tester = new RuleTester();
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
            languageOptions: { parserOptions: { ignoreNonFatalErrors: false } },
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
        const tester = new RuleTester();
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
});
