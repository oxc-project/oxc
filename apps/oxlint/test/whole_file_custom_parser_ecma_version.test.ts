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
    start: 0,
    end: SOURCE_TEXT.length,
    range: [0, SOURCE_TEXT.length],
    loc: {
      start: { line: 1, column: 0 },
      end: { line: 1, column: SOURCE_TEXT.length },
    },
    comments: [],
    tokens: [],
  };
}

afterEach(() => {
  registeredRules.length = 0;
  if (allOptions !== null) allOptions.length = 1;
  resetStateAfterError();
});

describe("whole-file custom parser ecmaVersion", () => {
  it("passes top-level languageOptions.ecmaVersion to the external parser", () => {
    let receivedOptions: Record<string, unknown> | undefined;
    const parser = {
      parseForESLint(_code: string, options?: Record<string, unknown>) {
        receivedOptions = options;
        return { ast: createProgram() };
      },
    };

    const languageOptionsId = registerLanguageOptions({
      parser,
      ecmaVersion: 2022,
    });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          ok: {
            create(context) {
              return {
                Program(node) {
                  context.report({
                    node,
                    message: "ok",
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

    expect(receivedOptions).toMatchObject({
      ecmaVersion: 2022,
      filePath: FILE_PATH,
    });
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0].message).toBe("ok");
  });


  it("exposes top-level languageOptions.ecmaVersion through context.languageOptions", () => {
    const parser = {
      parseForESLint() {
        return { ast: createProgram() };
      },
    };

    const languageOptionsId = registerLanguageOptions({
      parser,
      ecmaVersion: 2022,
    });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          ok: {
            create(context) {
              const seenEcmaVersion = context.languageOptions.ecmaVersion;
              return {
                Program(node) {
                  context.report({
                    node,
                    message: String(seenEcmaVersion),
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

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0].message).toBe("2022");
  });


  it("resets runtime ecmaVersion between files", () => {
    const parser = {
      parseForESLint() {
        return { ast: createProgram() };
      },
    };

    const configuredLanguageOptionsId = registerLanguageOptions({
      parser,
      ecmaVersion: 2022,
    });
    const defaultLanguageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          ok: {
            create(context) {
              const seenEcmaVersion = context.languageOptions.ecmaVersion;
              return {
                Program(node) {
                  context.report({
                    node,
                    message: String(seenEcmaVersion),
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

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [configuredLanguageOptionsId],
      null,
      SOURCE_TEXT,
    );
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0].message).toBe("2022");

    resetStateAfterError();

    lintFileImpl(
      FILE_PATH,
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [defaultLanguageOptionsId],
      null,
      SOURCE_TEXT,
    );
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0].message).toBe("2026");
  });
});
