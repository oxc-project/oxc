import { defineConfig } from "#oxlint";

function createLoc(start: number, end: number) {
  return {
    start: { line: 1, column: start },
    end: { line: 1, column: end },
  };
}

const svelteStubParser = {
  parseForESLint(code: string) {
    const commentText = "<!--A-->";
    const markupText = "<h1>Hello</h1>";
    const commentStart = code.indexOf(commentText);
    const commentEnd = commentStart + commentText.length;
    const markupStart = code.indexOf(markupText);
    const markupEnd = markupStart + markupText.length;

    return {
      ast: {
        type: "Program",
        sourceType: "module",
        body: [
          {
            type: "SvelteElement",
            name: "h1",
            children: [],
            range: [markupStart, markupEnd],
            loc: createLoc(markupStart, markupEnd),
          },
        ],
        range: [0, code.length],
        loc: {
          start: { line: 1, column: 0 },
          end: { line: 2, column: 0 },
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
      },
      services: {
        isSvelte: true,
      },
      visitorKeys: {
        Program: ["body"],
        SvelteElement: ["children"],
      },
    };
  },
};

export default defineConfig({
  categories: {
    correctness: "off",
  },
  jsPlugins: ["./plugin.ts"],
  overrides: [
    {
      files: ["**/*.svelte"],
      languageOptions: {
        parser: svelteStubParser,
      },
      rules: {
        "whole-file-svelte-source/tokens-and-comments": "error",
      },
    },
  ],
});
