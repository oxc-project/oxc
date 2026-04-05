import { afterEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { lintFile, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";
import { diagnostics } from "../src-js/plugins/report.ts";

describe("whole-file custom-parser parse errors", () => {
  afterEach(() => {
    registeredRules.length = 0;
    diagnostics.length = 0;
    if (allOptions !== null) allOptions.length = 1;
    resetStateAfterError();
  });

  it("serializes parser-like whole-file errors as parse errors", () => {
    const sourceText = `{#if page.data.user && }\n{/if}`;
    const errorIndex = sourceText.indexOf("}");

    const parser = {
      parseForESLint() {
        const error = new SyntaxError("Expected an identifier") as SyntaxError & {
          index: number;
          lineNumber: number;
          column: number;
        };
        error.index = errorIndex;
        error.lineNumber = 1;
        error.column = errorIndex;
        throw error;
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "never-runs": {
            create() {
              return {
                Program() {
                  throw new Error("rule should not run when the parser fails");
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

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      sourceText,
    );

    expect(result).not.toBeNull();
    expect(JSON.parse(result!)).toEqual({
      Success: {
        diagnostics: [],
        comments: [],
        parseError: {
          message: "Expected an identifier",
          start: errorIndex,
          end: errorIndex + 1,
        },
      },
    });
  });

  it("falls back to line and column when the parser omits an index", () => {
    const sourceText = `line 1\n{#if page.data.user && }\n{/if}`;
    const errorIndex = sourceText.indexOf("}");

    const parser = {
      parseForESLint() {
        const error = new SyntaxError("Unexpected token }") as SyntaxError & {
          lineNumber: number;
          column: number;
        };
        error.lineNumber = 2;
        error.column = 23;
        throw error;
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "never-runs": {
            create() {
              return {};
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

    const result = lintFile(
      "/workspace/App.svelte",
      0,
      null,
      [0],
      [0],
      "{}",
      '{"globals":{},"envs":{}}',
      [languageOptionsId],
      null,
      sourceText,
    );

    expect(result).not.toBeNull();
    expect(JSON.parse(result!)).toEqual({
      Success: {
        diagnostics: [],
        comments: [],
        parseError: {
          message: "Unexpected token }",
          start: errorIndex,
          end: errorIndex + 1,
        },
      },
    });
  });
});
