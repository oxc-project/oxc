import type { Node, Plugin } from "#oxlint/plugins";

function getLoc(sourceText: string, offset: number) {
  const lines = sourceText.slice(0, offset).split("\n");
  const line = lines.length;
  const column = lines.at(-1)?.length ?? 0;
  return { line, column };
}

const plugin: Plugin = {
  meta: {
    name: "whole-file-svelte-disable",
  },
  rules: {
    "markup-visible": {
      create(context) {
        return {
          Program() {
            const start = context.sourceCode.text.indexOf("<h1>");
            const end = start + 4;
            const startLoc = getLoc(context.sourceCode.text, start);
            const endLoc = getLoc(context.sourceCode.text, end);
            const node: Node = {
              start,
              end,
              range: [start, end],
              loc: {
                start: startLoc,
                end: endLoc,
              },
            };

            context.report({
              message: "whole-file custom-parser diagnostics should respect disable directives",
              node,
            });
          },
        };
      },
    },
  },
};

export default plugin;
