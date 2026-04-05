import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { resetFileContext, setupFileContext } from "../src-js/plugins/context.ts";
import { lintFileImpl, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";
import { diagnostics, report } from "../src-js/plugins/report.ts";
import { resetSourceAndAst, setupExternalSourceForFile } from "../src-js/plugins/source_code.ts";

import type { Program } from "../src-js/generated/types.d.ts";

const SOURCE_TEXT = "<h1>Hello</h1>";

function createProgram(): Program {
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
    comments: [],
    tokens: [],
    parent: null,
  };
}

const RULE_DETAILS = {
  ruleIndex: 0,
  messages: null,
  isFixable: false,
  hasSuggestions: false,
} as any;

describe("whole-file custom-parser diagnostics", () => {
  beforeEach(() => {
    setupFileContext("/test/App.svelte");
    setupExternalSourceForFile(SOURCE_TEXT, createProgram(), false, null);
    diagnostics.length = 0;
  });

  afterEach(() => {
    diagnostics.length = 0;
    resetSourceAndAst();
    resetFileContext();
  });

  it("prefers parser-provided node.loc over node.range", () => {
    report(
      {
        message: "greeting",
        node: {
          range: [0, SOURCE_TEXT.length],
          loc: {
            start: { line: 1, column: 4 },
            end: { line: 1, column: 9 },
          },
        },
      } as any,
      [],
      RULE_DETAILS,
    );

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]).toMatchObject({ start: 4, end: 9 });
  });

  it("falls back to node.range when parser-provided node.loc is malformed", () => {
    report(
      {
        message: "greeting",
        node: {
          range: [4, 9],
          loc: { start: null },
        },
      } as any,
      [],
      RULE_DETAILS,
    );

    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]).toMatchObject({ start: 4, end: 9 });
  });

  it("rejects reversed explicit loc ranges", () => {
    expect(() => {
      report(
        {
          message: "greeting",
          loc: {
            start: { line: 1, column: 9 },
            end: { line: 1, column: 4 },
          },
        } as any,
        [],
        RULE_DETAILS,
      );
    }).toThrow(/greater than or equal/);
  });

  it("rejects reversed node ranges", () => {
    expect(() => {
      report(
        {
          message: "greeting",
          node: { range: [9, 4] },
        } as any,
        [],
        RULE_DETAILS,
      );
    }).toThrow(/greater than or equal/);
  });

  it("rejects out-of-bounds node ranges", () => {
    expect(() => {
      report(
        {
          message: "greeting",
          node: { range: [0, SOURCE_TEXT.length + 1] },
        } as any,
        [],
        RULE_DETAILS,
      );
    }).toThrow(/within source text bounds/);
  });

  it("strips a leading BOM before whole-file custom-parser setup while preserving hasBOM", () => {
    registeredRules.length = 0;
    diagnostics.length = 0;
    if (allOptions !== null) allOptions.length = 1;

    const bomSourceText = `﻿${SOURCE_TEXT}`;
    let receivedCode: string | undefined;

    const parser = {
      parseForESLint(code: string) {
        receivedCode = code;
        return {
          ast: {
            ...createProgram(),
            end: code.length,
            range: [0, code.length],
            loc: {
              start: { line: 1, column: 0 },
              end: { line: 1, column: code.length },
            },
          },
        };
      },
    };

    const languageOptionsId = registerLanguageOptions({ parser });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "report-bom-state": {
            create(context) {
              return {
                Program(node) {
                  context.report({
                    node,
                    message: `${context.sourceCode.hasBOM}:${context.sourceCode.text}`,
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

    try {
      lintFileImpl(
        "/workspace/App.svelte",
        0,
        null,
        [0],
        [0],
        "{}",
        '{"globals":{},"envs":{}}',
        [languageOptionsId],
        null,
        bomSourceText,
      );

      expect(receivedCode).toBe(SOURCE_TEXT);
      expect(diagnostics).toHaveLength(1);
      expect(diagnostics[0]?.message).toBe(`true:${SOURCE_TEXT}`);
    } finally {
      registeredRules.length = 0;
      diagnostics.length = 0;
      if (allOptions !== null) allOptions.length = 1;
      resetStateAfterError();
    }
  });
});
