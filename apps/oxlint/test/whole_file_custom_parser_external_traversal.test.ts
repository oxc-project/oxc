import { afterEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { lintFileImpl, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";
import { diagnostics } from "../src-js/plugins/report.ts";

const FILE_PATH = "/workspace/App.svelte";
const SOURCE_TEXT = "<h1>Hello</h1>";

function createProgram() {
  return {
    type: "Program",
    body: [],
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
      type: "SvelteElement",
      name: "h1",
      start: 0,
      end: SOURCE_TEXT.length,
      range: [0, SOURCE_TEXT.length],
      loc: {
        start: { line: 1, column: 0 },
        end: { line: 1, column: SOURCE_TEXT.length },
      },
      children: [
        {
          type: "SvelteText",
          value: "Hello",
          start: 4,
          end: 9,
          range: [4, 9],
          loc: {
            start: { line: 1, column: 4 },
            end: { line: 1, column: 9 },
          },
        },
      ],
    },
  };
}

afterEach(() => {
  registeredRules.length = 0;
  diagnostics.length = 0;
  if (allOptions !== null) allOptions.length = 1;
  resetStateAfterError();
});

describe("whole-file custom parser external traversal", () => {
  it("walks external-only nodes with parser visitor keys and inferred child keys", () => {
    const parser = {
      parseForESLint() {
        return {
          ast: createProgram(),
          visitorKeys: {
            Program: ["body", "templateBody"],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-svelte": {
            create(context) {
              return {
                SvelteElement(node) {
                  context.report({ node, message: `element:${(node as { type: string }).type}` });
                },
                "SvelteElement > SvelteText"(node) {
                  context.report({ node, message: `selector:${(node as { value: string }).value}` });
                },
                "*"(node) {
                  if ((node as { type: string }).type === "SvelteText") {
                    context.report({ node, message: `wildcard:${(node as { type: string }).type}` });
                  }
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

    setOptions(
      JSON.stringify({
        options: [[]],
        ruleIds: [0],
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      SOURCE_TEXT,
    );

    expect(diagnostics.map(({ message }) => message).sort()).toEqual([
      "element:SvelteElement",
      "selector:Hello",
      "wildcard:SvelteText",
    ]);
  });

  it("respects parser visitor key order while still appending inferred external subtrees", () => {
    const parser = {
      parseForESLint() {
        return {
          ast: {
            type: "Program",
            body: [
              {
                type: "ExpressionStatement",
                expression: {
                  type: "Literal",
                  value: "script",
                  raw: '"script"',
                  start: 0,
                  end: 8,
                  range: [0, 8],
                  loc: {
                    start: { line: 1, column: 0 },
                    end: { line: 1, column: 8 },
                  },
                },
                start: 0,
                end: 8,
                range: [0, 8],
                loc: {
                  start: { line: 1, column: 0 },
                  end: { line: 1, column: 8 },
                },
              },
            ],
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
              type: "SvelteElement",
              name: "h1",
              start: 0,
              end: SOURCE_TEXT.length,
              range: [0, SOURCE_TEXT.length],
              loc: {
                start: { line: 1, column: 0 },
                end: { line: 1, column: SOURCE_TEXT.length },
              },
              children: [],
            },
          },
          visitorKeys: {
            Program: ["templateBody", "body"],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-order": {
            create(context) {
              return {
                SvelteElement(node) {
                  context.report({ node, message: "template" });
                },
                ExpressionStatement(node) {
                  context.report({ node, message: "script" });
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

    setOptions(
      JSON.stringify({
        options: [[]],
        ruleIds: [0],
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      SOURCE_TEXT,
    );

    expect(diagnostics.map(({ message }) => message)).toEqual(["template", "script"]);
  });

  it("reaches custom subtrees on known container nodes even when parser visitor keys omit them", () => {
    const parser = {
      parseForESLint() {
        return {
          ast: createProgram(),
          visitorKeys: {
            Program: ["body"],
            SvelteElement: ["children"],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-svelte": {
            create(context) {
              return {
                SvelteText(node) {
                  context.report({ node, message: `text:${(node as { value: string }).value}` });
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

    setOptions(
      JSON.stringify({
        options: [[]],
        ruleIds: [0],
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      SOURCE_TEXT,
    );

    expect(diagnostics.map(({ message }) => message)).toEqual(["text:Hello"]);
  });

  it("ignores non-AST parser metadata while normalizing external nodes", () => {
    const parser = {
      parseForESLint() {
        const program = createProgram() as ReturnType<typeof createProgram> & {
          metadata?: { owner: unknown };
        };
        program.metadata = { owner: program };
        return {
          ast: program,
          visitorKeys: {
            Program: ["body", "templateBody"],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-svelte": {
            create(context) {
              return {
                SvelteText(node) {
                  context.report({ node, message: `text:${(node as { value: string }).value}` });
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

    setOptions(
      JSON.stringify({
        options: [[]],
        ruleIds: [0],
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    expect(() => {
      lintFileImpl(
        FILE_PATH,
        0,
        null,
        [0],
        [0],
        "{}",
        '{"globals":{},"envs":{}}',
        [languageOptionsId],
        null,
        SOURCE_TEXT,
      );
    }).not.toThrow();

    expect(diagnostics.map(({ message }) => message)).toEqual(["text:Hello"]);
  });


  it("does not traverse parser comments, tokens, or attached comment arrays as AST nodes", () => {
    const parser = {
      parseForESLint() {
        const program = createProgram() as ReturnType<typeof createProgram> & {
          templateBody: ReturnType<typeof createProgram>["templateBody"] & {
            leadingComments?: unknown[];
          };
        };
        const comment = {
          type: "Line",
          value: "ignored",
          start: 0,
          end: 0,
          range: [0, 0],
          loc: {
            start: { line: 1, column: 0 },
            end: { line: 1, column: 0 },
          },
        };
        const token = {
          type: "Identifier",
          value: "ignored",
          start: 0,
          end: 1,
          range: [0, 1],
          loc: {
            start: { line: 1, column: 0 },
            end: { line: 1, column: 1 },
          },
        };
        program.comments = [comment];
        program.tokens = [token];
        program.templateBody.leadingComments = [comment];
        return {
          ast: program,
          visitorKeys: {
            Program: ["body", "templateBody", "comments", "tokens"],
            SvelteElement: ["children", "leadingComments"],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-svelte": {
            create(context) {
              return {
                Line(node) {
                  context.report({ node, message: "comment" });
                },
                Identifier(node) {
                  context.report({ node, message: "token" });
                },
                SvelteText(node) {
                  context.report({ node, message: `text:${(node as { value: string }).value}` });
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

    setOptions(
      JSON.stringify({
        options: [[]],
        ruleIds: [0],
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      SOURCE_TEXT,
    );

    expect(diagnostics.map(({ message }) => message)).toEqual(["text:Hello"]);
  });

  it("sets the external Program parent to null so SourceCode.getAncestors works", () => {
    const parser = {
      parseForESLint() {
        return {
          ast: createProgram(),
          visitorKeys: {
            Program: ["body", "templateBody"],
            SvelteElement: ["children"],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-svelte": {
            create(context) {
              return {
                SvelteText(node) {
                  context.report({
                    node,
                    message: context.sourceCode
                      .getAncestors(node as any)
                      .map((ancestor) => (ancestor as { type: string }).type)
                      .join(","),
                  });
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

    setOptions(
      JSON.stringify({
        options: [[]],
        ruleIds: [0],
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    expect(() => {
      lintFileImpl(
        FILE_PATH,
        0,
        null,
        [0],
        [0],
        "{}",
        '{"globals":{},"envs":{}}',
        [languageOptionsId],
        null,
        SOURCE_TEXT,
      );
    }).not.toThrow();

    expect(diagnostics.map(({ message }) => message)).toEqual(["Program,SvelteElement"]);
  });

  it("respects explicit empty parser visitor keys for custom nodes", () => {
    const parser = {
      parseForESLint() {
        return {
          ast: createProgram(),
          visitorKeys: {
            Program: ["templateBody"],
            SvelteElement: [],
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "walk-svelte": {
            create(context) {
              return {
                SvelteElement(node) {
                  context.report({ node, message: "element" });
                },
                SvelteText(node) {
                  context.report({ node, message: "text" });
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

    setOptions(
      JSON.stringify({
        options: [[]],
        ruleIds: [0],
        cwd: "/workspace",
        workspaceUri: null,
      }),
    );

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      SOURCE_TEXT,
    );

    expect(diagnostics.map(({ message }) => message)).toEqual(["element"]);
  });

});
