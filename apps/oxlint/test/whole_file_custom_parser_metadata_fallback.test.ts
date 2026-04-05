import { afterEach, describe, expect, it } from "vitest";
import { registerLanguageOptions } from "../src-js/js_language_options_registry.ts";
import { lintFileImpl, resetStateAfterError } from "../src-js/plugins/lint.ts";
import { registerPlugin, registeredRules } from "../src-js/plugins/load.ts";
import { allOptions, setOptions } from "../src-js/plugins/options.ts";
import { diagnostics } from "../src-js/plugins/report.ts";

const FILE_PATH = "/workspace/App.svelte";
const SOURCE_TEXT = "<!--A--><h1>Hello</h1>";

function createLoc(start: number, end: number) {
  return {
    start: { line: 1, column: start },
    end: { line: 1, column: end },
  };
}

afterEach(() => {
  registeredRules.length = 0;
  diagnostics.length = 0;
  if (allOptions !== null) allOptions.length = 1;
  resetStateAfterError();
});

describe("whole-file custom parser metadata fallback", () => {
  it("falls back to parseForESLint top-level comments and tokens when ast metadata arrays are empty", () => {
    const commentText = "<!--A-->";
    const markupText = "<h1>Hello</h1>";
    const commentStart = SOURCE_TEXT.indexOf(commentText);
    const commentEnd = commentStart + commentText.length;
    const markupStart = SOURCE_TEXT.indexOf(markupText);
    const markupEnd = markupStart + markupText.length;

    const parser = {
      parseForESLint() {
        return {
          ast: {
            type: "Program",
            body: [],
            sourceType: "module",
            range: [0, SOURCE_TEXT.length],
            loc: createLoc(0, SOURCE_TEXT.length),
            comments: [],
            tokens: [],
            templateBody: {
              type: "SvelteElement",
              name: "h1",
              children: [],
              range: [markupStart, markupEnd],
              loc: createLoc(markupStart, markupEnd),
            },
          },
          comments: [
            {
              type: "Block",
              value: "A",
              range: [commentStart, commentEnd],
              loc: createLoc(commentStart, commentEnd),
            },
          ],
          tokens: [
            { type: "Punctuator", value: "<", range: [markupStart, markupStart + 1] },
            { type: "Identifier", value: "h1", range: [markupStart + 1, markupStart + 3] },
            { type: "Punctuator", value: ">", range: [markupStart + 3, markupStart + 4] },
            { type: "Identifier", value: "Hello", range: [markupStart + 4, markupStart + 9] },
            { type: "Punctuator", value: "</", range: [markupStart + 9, markupStart + 11] },
            { type: "Identifier", value: "h1", range: [markupStart + 11, markupStart + 13] },
            { type: "Punctuator", value: ">", range: [markupStart + 13, markupStart + 14] },
          ],
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
          "read-metadata": {
            create(context) {
              return {
                SvelteElement(node) {
                  const comments = context.sourceCode.getAllComments() as Array<{
                    type: string;
                    value: string;
                  }>;
                  const tokens = context.sourceCode.getTokens(node) as Array<{
                    type: string;
                    value: string;
                  }>;
                  context.report({
                    node,
                    message: [
                      `comments=${comments.map(({ type, value }) => `${type}:${value}`).join(",")}`,
                      `tokens=${tokens.map(({ type, value }) => `${type}:${value}`).join(",")}`,
                      `astComments=${context.sourceCode.ast.comments === comments}`,
                      `astTokens=${context.sourceCode.ast.tokens?.[0] === tokens[0]}`,
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

    expect(diagnostics.map(({ message }) => message)).toEqual([
      "comments=Block:A; tokens=Punctuator:<,Identifier:h1,Punctuator:>,Identifier:Hello,Punctuator:</,Identifier:h1,Punctuator:>; astComments=true; astTokens=true",
    ]);
  });


  it("falls back to AST-attached visitor keys, parser services, and scope manager", () => {
    const parserServices = Object.freeze({ isAstAttached: true });
    const scopeManager = {
      scopes: [],
      globalScope: null,
      acquire() {
        return null;
      },
      getDeclaredVariables() {
        return [];
      },
    };

    const parser = {
      parseForESLint() {
        return {
          ast: {
            type: "Program",
            body: [],
            sourceType: "module",
            range: [0, SOURCE_TEXT.length],
            loc: createLoc(0, SOURCE_TEXT.length),
            comments: [],
            tokens: [],
            visitorKeys: {
              Program: ["body", "templateBody"],
              SvelteElement: ["children"],
            },
            services: parserServices,
            scopeManager,
            templateBody: {
              type: "SvelteElement",
              name: "h1",
              children: [],
              range: [0, SOURCE_TEXT.length],
              loc: createLoc(0, SOURCE_TEXT.length),
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
          "read-ast-attached-metadata": {
            create(context) {
              return {
                SvelteElement(node) {
                  context.report({
                    node,
                    message: [
                      `services=${context.sourceCode.parserServices === parserServices}`,
                      `visitorKeys=${context.sourceCode.visitorKeys.Program?.includes("templateBody") === true}`,
                      `scopeManager=${context.sourceCode.scopeManager === scopeManager}`,
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

    expect(diagnostics.map(({ message }) => message)).toEqual([
      "services=true; visitorKeys=true; scopeManager=true",
    ]);
  });
});
