import { createRequire } from "node:module";
import { describe, expect, it } from "vitest";
import { RuleTester } from "../src-js/package/rule_tester.ts";

import type { Rule as ESLintRule, RuleTester as ESLintRuleTester } from "eslint";
import type { Rule } from "../src-js/plugins.ts";

const runESLint: ESLintRuleTester["run"] = createRequire(import.meta.url)(
  "./rule_tester_assertions.cjs",
);

const rule: Rule = {
  meta: {
    hasSuggestions: true,
    schema: [{ enum: ["plain", "data", "suggestion", "point", "empty-range", "start-only"] }],
    messages: {
      plain: "Unexpected identifier",
      data: "Unexpected {{ name }}",
      fix: "Replace {{ name }}",
    },
  },
  create(context) {
    return {
      Identifier(node) {
        const mode = context.options[0];
        context.report({
          node,
          ...(mode === "point" ? { loc: { line: 1, column: 0 } } : {}),
          ...(mode === "start-only" ? { loc: { start: { line: 1, column: 0 } } } : {}),
          ...(mode === "empty-range"
            ? { loc: { start: { line: 1, column: 0 }, end: { line: 1, column: 0 } } }
            : {}),
          messageId: mode === "data" ? "data" : "plain",
          data: { name: node.name },
          ...(mode === "suggestion"
            ? {
                suggest: [
                  {
                    messageId: "fix",
                    data: { name: node.name },
                    fix: (fixer) => fixer.replaceText(node, "bar"),
                  },
                ],
              }
            : {}),
        });
      },
    };
  },
};

const literalRule: Rule = {
  create(context) {
    return {
      Identifier(node) {
        context.report({ node, message: "Unexpected identifier" });
      },
    };
  },
};

interface Case {
  name: string;
  assertionOptions?: RuleTester.AssertionOptions;
  errors: RuleTester.InvalidTestCase["errors"];
  mode?: "plain" | "data" | "suggestion" | "point" | "empty-range" | "start-only";
  rule?: Rule;
  failure?: RegExp;
}

const location = { line: 1, column: 1, endLine: 1, endColumn: 4 };
const cases: Case[] = [
  {
    name: "explicit empty ranges reject undefined end properties",
    assertionOptions: { requireLocation: true },
    mode: "empty-range",
    errors: [{ messageId: "plain", line: 1, column: 1, endLine: undefined, endColumn: undefined }],
    failure: /location/i,
  },
  {
    name: "omitted end locations reject numeric end expectations",
    assertionOptions: { requireLocation: true },
    mode: "point",
    errors: [{ messageId: "plain", line: 1, column: 1, endLine: 1, endColumn: 1 }],
    failure: /location/i,
  },
  {
    name: "inherited data does not satisfy the error data requirement",
    assertionOptions: { requireData: true },
    mode: "data",
    errors: [Object.assign(Object.create({ data: { name: "foo" } }), { messageId: "data" })],
    failure: /data/,
  },
  {
    name: "true requires a message in error objects",
    assertionOptions: { requireMessage: true },
    // @ts-expect-error - intentionally omit the required message assertion
    errors: [{}],
    failure: /message/,
  },
  {
    name: "message mode rejects both message forms",
    assertionOptions: { requireMessage: "message" },
    errors: [{ message: "Unexpected identifier", messageId: "plain" }],
    failure: /messageId/,
  },
  {
    name: "messageId mode rejects both message forms",
    assertionOptions: { requireMessage: "messageId" },
    errors: [{ message: "Unexpected identifier", messageId: "plain" }],
    failure: /message/,
  },
  {
    name: "data requirements accept string messages",
    assertionOptions: { requireData: true },
    mode: "data",
    errors: ["Unexpected foo"],
  },
  {
    name: "empty data does not satisfy placeholders",
    assertionOptions: { requireData: true },
    mode: "data",
    errors: [{ messageId: "data", data: {} }],
    failure: /message/i,
  },
  {
    name: "explicit empty ranges still require end properties",
    assertionOptions: { requireLocation: true },
    mode: "empty-range",
    errors: [{ messageId: "plain", line: 1, column: 1 }],
    failure: /endLine, endColumn/,
  },
  {
    name: "explicit empty ranges accept complete locations",
    assertionOptions: { requireLocation: true },
    mode: "empty-range",
    errors: [{ messageId: "plain", line: 1, column: 1, endLine: 1, endColumn: 1 }],
  },
  {
    name: "start-only location can omit end",
    assertionOptions: { requireLocation: true },
    mode: "start-only",
    errors: [{ messageId: "plain", line: 1, column: 1 }],
  },
  { name: "defaults accept counts", errors: 1 },
  {
    name: "false options accept counts",
    assertionOptions: { requireMessage: false, requireLocation: false, requireData: false },
    errors: 1,
  },
  { name: "requireData alone accepts counts", assertionOptions: { requireData: true }, errors: 1 },
  ...([true, "message", "messageId"] as const).map((requireMessage): Case => ({
    name: `requireMessage ${requireMessage} rejects counts`,
    assertionOptions: { requireMessage },
    errors: 1,
    failure: /array/,
  })),
  ...([true, "message"] as const).flatMap((requireMessage): Case[] => [
    {
      name: `requireMessage ${requireMessage} accepts strings`,
      assertionOptions: { requireMessage },
      errors: ["Unexpected identifier"],
    },
    {
      name: `requireMessage ${requireMessage} accepts regexps`,
      assertionOptions: { requireMessage },
      errors: [/Unexpected/],
    },
    {
      name: `requireMessage ${requireMessage} accepts message objects`,
      assertionOptions: { requireMessage },
      errors: [{ message: "Unexpected identifier" }],
    },
  ]),
  {
    name: "true accepts messageId",
    assertionOptions: { requireMessage: true },
    errors: [{ messageId: "plain" }],
  },
  {
    name: "messageId accepts messageId",
    assertionOptions: { requireMessage: "messageId" },
    errors: [{ messageId: "plain" }],
  },
  {
    name: "message rejects messageId",
    assertionOptions: { requireMessage: "message" },
    errors: [{ messageId: "plain" }],
    failure: /message/,
  },
  {
    name: "messageId rejects message",
    assertionOptions: { requireMessage: "messageId" },
    errors: [{ message: "Unexpected identifier" }],
    failure: /messageId/,
  },
  {
    name: "messageId rejects strings",
    assertionOptions: { requireMessage: "messageId" },
    errors: ["Unexpected identifier"],
    failure: /object/,
  },
  {
    name: "messageId rejects regexps",
    assertionOptions: { requireMessage: "messageId" },
    errors: [/Unexpected/],
    failure: /object/,
  },
  {
    name: "messageId requires meta.messages",
    assertionOptions: { requireMessage: "messageId" },
    rule: literalRule,
    errors: [{ messageId: "plain" }],
    failure: /meta.messages/,
  },
  {
    name: "message accepts rules without meta.messages",
    assertionOptions: { requireMessage: "message" },
    rule: literalRule,
    errors: ["Unexpected identifier"],
  },
  {
    name: "location rejects counts",
    assertionOptions: { requireLocation: true },
    errors: 1,
    failure: /array/,
  },
  {
    name: "location rejects strings",
    assertionOptions: { requireLocation: true },
    errors: ["Unexpected identifier"],
    failure: /object/,
  },
  {
    name: "location rejects regexps",
    assertionOptions: { requireLocation: true },
    errors: [/Unexpected/],
    failure: /object/,
  },
  {
    name: "location accepts complete location",
    assertionOptions: { requireLocation: true },
    errors: [{ messageId: "plain", ...location }],
  },
  ...(["line", "column", "endLine", "endColumn"] as const).map((key): Case => {
    const incomplete: Partial<typeof location> = { ...location };
    delete incomplete[key];
    return {
      name: `location requires ${key}`,
      assertionOptions: { requireLocation: true },
      errors: [{ messageId: "plain", ...incomplete }],
      failure: new RegExp(key),
    };
  }),
  {
    name: "location still checks values",
    assertionOptions: { requireLocation: true },
    errors: [{ messageId: "plain", ...location, column: 2 }],
    failure: /location/i,
  },
  {
    name: "point location can omit end in compatibility mode",
    assertionOptions: { requireLocation: true },
    mode: "point",
    errors: [{ messageId: "plain", line: 1, column: 1 }],
  },
  {
    name: "point location accepts explicit undefined end",
    assertionOptions: { requireLocation: true },
    mode: "point",
    errors: [{ messageId: "plain", line: 1, column: 1, endLine: undefined, endColumn: undefined }],
  },
  {
    name: "data defaults allow omitted placeholder data",
    mode: "data",
    errors: [{ messageId: "data" }],
  },
  ...([true, "error"] as const).flatMap((requireData): Case[] => [
    {
      name: `data ${requireData} requires error data`,
      assertionOptions: { requireData },
      mode: "data",
      errors: [{ messageId: "data" }],
      failure: /data/,
    },
    {
      name: `data ${requireData} accepts correct error data`,
      assertionOptions: { requireData },
      mode: "data",
      errors: [{ messageId: "data", data: { name: "foo" } }],
    },
    {
      name: `data ${requireData} rejects incorrect error data`,
      assertionOptions: { requireData },
      mode: "data",
      errors: [{ messageId: "data", data: { name: "bar" } }],
      failure: /message/i,
    },
    {
      name: `data ${requireData} allows messages without placeholders`,
      assertionOptions: { requireData },
      errors: [{ messageId: "plain" }],
    },
    {
      name: `data ${requireData} allows literal messages`,
      assertionOptions: { requireData },
      mode: "data",
      errors: [{ message: "Unexpected foo" }],
    },
  ]),
  {
    name: "suggestion mode does not require error data",
    assertionOptions: { requireData: "suggestion" },
    mode: "data",
    errors: [{ messageId: "data" }],
  },
  ...([true, "suggestion"] as const).flatMap((requireData): Case[] => [
    {
      name: `data ${requireData} requires suggestion data`,
      assertionOptions: { requireData },
      mode: "suggestion",
      errors: [{ messageId: "plain", suggestions: [{ messageId: "fix", output: "bar" }] }],
      failure: /data/,
    },
    {
      name: `data ${requireData} accepts correct suggestion data`,
      assertionOptions: { requireData },
      mode: "suggestion",
      errors: [
        {
          messageId: "plain",
          suggestions: [{ messageId: "fix", data: { name: "foo" }, output: "bar" }],
        },
      ],
    },
    {
      name: `data ${requireData} allows literal suggestion descriptions`,
      assertionOptions: { requireData },
      mode: "suggestion",
      errors: [{ messageId: "plain", suggestions: [{ desc: "Replace foo", output: "bar" }] }],
    },
  ]),
  {
    name: "error mode does not require suggestion data",
    assertionOptions: { requireData: "error" },
    mode: "suggestion",
    errors: [{ messageId: "plain", suggestions: [{ messageId: "fix", output: "bar" }] }],
  },
  {
    name: "all options work together",
    assertionOptions: { requireMessage: "messageId", requireLocation: true, requireData: true },
    mode: "data",
    errors: [{ messageId: "data", data: { name: "foo" }, ...location }],
  },
];

// Run the same cases through ESLint to guard against differences in option semantics.
describe.each(["Oxlint", "ESLint"] as const)("%s assertion options", (engine) => {
  function run(tests: RuleTester.TestCases, testedRule: Rule = rule): void {
    if (engine === "Oxlint") {
      RuleTester.describe = (_name, fn) => fn();
      RuleTester.it = (_name, fn) => fn();
      new RuleTester({ eslintCompat: true }).run("assertions", testedRule, tests);
    } else {
      runESLint(
        "assertions",
        testedRule as unknown as ESLintRule.RuleModule,
        tests as Parameters<ESLintRuleTester["run"]>[2],
      );
    }
  }

  it.each(cases.filter((test) => test.failure === undefined))(
    "$name",
    ({ assertionOptions, errors, mode = "plain", rule: testedRule = rule }) => {
      const execute = () =>
        run(
          {
            assertionOptions,
            valid: [],
            invalid: [{ code: "foo", ...(testedRule === rule ? { options: [mode] } : {}), errors }],
          },
          testedRule,
        );
      expect(execute).not.toThrow();
    },
  );

  it.each(cases.filter((test) => test.failure !== undefined))(
    "$name",
    ({ assertionOptions, errors, mode = "plain", rule: testedRule = rule, failure }) => {
      expect(() =>
        run(
          {
            assertionOptions,
            valid: [],
            invalid: [{ code: "foo", ...(testedRule === rule ? { options: [mode] } : {}), errors }],
          },
          testedRule,
        ),
      ).toThrow(failure);
    },
  );

  it("does not leak assertion options between runs", () => {
    const options = Object.freeze({
      requireMessage: true,
      requireLocation: true,
    } satisfies RuleTester.AssertionOptions);
    run({
      assertionOptions: options,
      valid: [],
      invalid: [{ code: "foo", errors: [{ messageId: "plain", ...location }] }],
    });
    expect(() => run({ valid: [], invalid: [{ code: "foo", errors: 1 }] })).not.toThrow();
  });

  it("does not apply invalid-case requirements to valid cases", () => {
    expect(() =>
      run(
        {
          assertionOptions: {
            requireMessage: "messageId",
            requireLocation: true,
            requireData: true,
          },
          valid: ["42"],
          invalid: [],
        },
        literalRule,
      ),
    ).not.toThrow();
  });
});

describe("Oxlint location conventions", () => {
  it("requires and accepts zero-based columns outside compatibility mode", () => {
    RuleTester.describe = (_name, fn) => fn();
    RuleTester.it = (_name, fn) => fn();
    const tester = new RuleTester();
    expect(() =>
      tester.run("assertions", rule, {
        assertionOptions: { requireLocation: true },
        valid: [],
        invalid: [
          {
            code: "foo",
            errors: [{ messageId: "plain", line: 1, column: 0, endLine: 1, endColumn: 3 }],
          },
        ],
      }),
    ).not.toThrow();
    expect(() =>
      tester.run("assertions", rule, {
        assertionOptions: { requireLocation: true },
        valid: [],
        invalid: [
          { code: "foo", options: ["point"], errors: [{ messageId: "plain", line: 1, column: 0 }] },
        ],
      }),
    ).toThrow(/endLine, endColumn/);
  });
});
