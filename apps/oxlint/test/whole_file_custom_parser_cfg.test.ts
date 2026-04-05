import { afterEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { lintFileImpl, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";
import { diagnostics } from "../src-js/plugins/report.ts";

const FILE_PATH = "/workspace/App.svelte";
const SOURCE_TEXT = "while (true) {}";

function createProgram() {
  return {
    type: "Program",
    body: [] as unknown[],
    sourceType: "module",
    start: 0,
    end: SOURCE_TEXT.length,
    range: [0, SOURCE_TEXT.length],
    loc: {
      start: { line: 1, column: 0 },
      end: { line: 1, column: SOURCE_TEXT.length },
    },
    comments: [] as unknown[],
    tokens: [] as unknown[],
    templateBody: {
      type: "SvelteScriptBlock",
      body: [
        {
          type: "WhileStatement",
          test: {
            type: "Literal",
            value: true,
            raw: "true",
            start: 7,
            end: 11,
            range: [7, 11],
            loc: {
              start: { line: 1, column: 7 },
              end: { line: 1, column: 11 },
            },
          },
          body: {
            type: "BlockStatement",
            body: [] as unknown[],
            start: 13,
            end: 15,
            range: [13, 15],
            loc: {
              start: { line: 1, column: 13 },
              end: { line: 1, column: 15 },
            },
          },
          start: 0,
          end: 15,
          range: [0, 15],
          loc: {
            start: { line: 1, column: 0 },
            end: { line: 1, column: 15 },
          },
        },
      ],
      start: 0,
      end: SOURCE_TEXT.length,
      range: [0, SOURCE_TEXT.length],
      loc: {
        start: { line: 1, column: 0 },
        end: { line: 1, column: SOURCE_TEXT.length },
      },
    },
  };
}

afterEach(() => {
  registeredRules.length = 0;
  diagnostics.length = 0;
  if (allOptions !== null) allOptions.length = 1;
  resetStateAfterError();
});

describe("whole-file custom parser CFG support", () => {
  it("runs CFG listeners through the whole-file custom parser lane", () => {
    const parser = {
      parseForESLint() {
        return {
          ast: createProgram(),
          visitorKeys: {
            Program: ["body", "templateBody"],
            SvelteScriptBlock: ["body"],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });
    let loopArgsCount = 0;

    const plugin = registerPlugin(
      {
        meta: { name: "cfg-plugin" },
        rules: {
          cfg: {
            create(context) {
              const events: [string, string][] = [];

              return {
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onCodePathStart(_codePath, node) {
                  events.push(["onCodePathStart", node.type]);
                },
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onCodePathEnd(_codePath, node) {
                  events.push(["onCodePathEnd", node.type]);

                  if (node.type === "Program") {
                    context.report({
                      node,
                      message: events.map(([eventName, type]) => `${eventName} ${type}`).join("\n"),
                    });
                  }
                },
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onCodePathSegmentStart(_segment, node) {
                  events.push(["onCodePathSegmentStart", node.type]);
                },
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onCodePathSegmentEnd(_segment, node) {
                  events.push(["onCodePathSegmentEnd", node.type]);
                },
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onUnreachableCodePathSegmentStart(_segment, node) {
                  events.push(["onUnreachableCodePathSegmentStart", node.type]);
                },
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onUnreachableCodePathSegmentEnd(_segment, node) {
                  events.push(["onUnreachableCodePathSegmentEnd", node.type]);
                },
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onCodePathSegmentLoop(_fromSegment, _toSegment, node) {
                  events.push(["onCodePathSegmentLoop", node.type]);
                },
              };
            },
          },
          noop: {
            create() {
              return {
                // @ts-expect-error - TODO: Make the types for CFG events work in test fixtures
                onCodePathSegmentLoop(...args) {
                  loopArgsCount = args.length;
                  expect((args[2] as { type: string }).type).toBe("WhileStatement");
                },
              };
            },
          },
        },
      },
      null,
      false,
      null,
    );

    const ruleIds = [plugin.offset, plugin.offset + 1];
    setOptions(
      JSON.stringify({
        options: [[], []],
        ruleIds,
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      ruleIds,
      [0, 1],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      SOURCE_TEXT,
    );

    expect(loopArgsCount).toBe(3);
    expect(diagnostics.map(({ message }) => message)).toEqual([
      [
        "onCodePathStart Program",
        "onCodePathSegmentStart Program",
        "onCodePathSegmentEnd Literal",
        "onCodePathSegmentStart Literal",
        "onCodePathSegmentEnd BlockStatement",
        "onCodePathSegmentStart BlockStatement",
        "onCodePathSegmentLoop WhileStatement",
        "onCodePathSegmentEnd WhileStatement",
        "onUnreachableCodePathSegmentStart WhileStatement",
        "onUnreachableCodePathSegmentEnd Program",
        "onCodePathEnd Program",
      ].join("\n"),
    ]);
  });
});
