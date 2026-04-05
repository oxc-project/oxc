import { afterEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { lintFileImpl, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";

const FILE_PATH = "/workspace/App.svelte";
const SOURCE_TEXT = `<script lang="ts">export let name: string = "world";</script>\n<h1>Hello {name}</h1>`;

function createProgram() {
  return {
    type: "Program",
    body: [],
    start: 0,
    end: SOURCE_TEXT.length,
    range: [0, SOURCE_TEXT.length],
    loc: {
      start: { line: 1, column: 0 },
      end: { line: 2, column: 17 },
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

describe("whole-file custom parser type-aware option isolation", () => {
  it("does not let parser mutations leak back into shared nested parser options", () => {
    const nestedTsParser = {
      parseForESLint() {
        return { ast: createProgram() };
      },
    };
    const preprocess = () => "preprocessed";
    const parserOptions = {
      parser: {
        ts: nestedTsParser,
      },
      projectService: true,
      extraFileExtensions: [".svelte"],
      svelteConfig: {
        compilerOptions: {
          runes: true,
        },
        preprocess,
      },
    };
    const parserCallSnapshots: string[] = [];
    const parserCallOptions: Record<string, unknown>[] = [];
    const parser = {
      name: "mutating-svelte-like-parser",
      parseForESLint(_code: string, options?: Record<string, unknown>) {
        if (options == null) {
          throw new Error("expected parser call options");
        }

        parserCallOptions.push(options);
        const extraFileExtensions = Array.isArray(options.extraFileExtensions)
          ? options.extraFileExtensions
          : [];
        const svelteConfig = options.svelteConfig as {
          compilerOptions?: { runes?: unknown };
          preprocess?: unknown;
        } | undefined;
        parserCallSnapshots.push([
          `exts:${extraFileExtensions.join(",")}`,
          `runes:${svelteConfig?.compilerOptions?.runes === true}`,
          `projectService:${options.projectService === true}`,
          `preprocess:${typeof svelteConfig?.preprocess === "function"}`,
        ].join("; "));

        extraFileExtensions.push(".mutated");
        if (svelteConfig?.compilerOptions != null) {
          svelteConfig.compilerOptions.runes = false;
        }

        return { ast: createProgram() };
      },
    };

    const languageOptionsId = registerLanguageOptions({
      parser,
      parserOptions,
    });

    registerPlugin(
      {
        meta: { name: "test-plugin" },
        rules: {
          "report-type-aware-options": {
            create(context) {
              return {
                Program(node) {
                  const currentParserOptions = context.languageOptions.parserOptions as {
                    parser?: { ts?: unknown };
                    projectService?: unknown;
                    extraFileExtensions?: unknown;
                    svelteConfig?: {
                      compilerOptions?: { runes?: unknown };
                      preprocess?: unknown;
                    };
                  };
                  const extraFileExtensions = Array.isArray(currentParserOptions.extraFileExtensions)
                    ? currentParserOptions.extraFileExtensions
                    : [];

                  context.report({
                    node,
                    message: [
                      `exts:${extraFileExtensions.join(",")}`,
                      `runes:${currentParserOptions.svelteConfig?.compilerOptions?.runes === true}`,
                      `projectService:${currentParserOptions.projectService === true}`,
                      `parserSame:${currentParserOptions.parser?.ts === nestedTsParser}`,
                      `preprocessSame:${currentParserOptions.svelteConfig?.preprocess === preprocess}`,
                    ].join("; "),
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

    const firstResult = lintFileImpl(
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
    const secondResult = lintFileImpl(
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

    expect(parserCallSnapshots).toEqual([
      "exts:.svelte; runes:true; projectService:true; preprocess:true",
      "exts:.svelte; runes:true; projectService:true; preprocess:true",
    ]);
    expect(parserCallOptions).toHaveLength(2);
    expect(parserCallOptions[0]).not.toBe(parserCallOptions[1]);
    expect(parserCallOptions[0]?.extraFileExtensions).not.toBe(
      parserCallOptions[1]?.extraFileExtensions,
    );
    expect(parserOptions.extraFileExtensions).toEqual([".svelte"]);
    expect(parserOptions.svelteConfig.compilerOptions.runes).toBe(true);

    const firstPayload = JSON.parse(firstResult ?? "null");
    const secondPayload = JSON.parse(secondResult ?? "null");
    expect(firstPayload.Success.diagnostics[0].message).toBe(
      "exts:.svelte; runes:true; projectService:true; parserSame:true; preprocessSame:true",
    );
    expect(secondPayload.Success.diagnostics[0].message).toBe(
      "exts:.svelte; runes:true; projectService:true; parserSame:true; preprocessSame:true",
    );
  });
});
