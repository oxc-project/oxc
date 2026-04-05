import type { Node, Plugin } from "#oxlint/plugins";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

const plugin: Plugin = {
  meta: {
    name: "whole-file-svelte",
  },
  rules: {
    "markup-visible": {
      create(context) {
        return {
          Program() {
            const parserServices = context.sourceCode.parserServices as {
              isSvelte?: unknown;
              filePath?: unknown;
              flavor?: unknown;
            };

            context.report({
              message: [
                `whole-file: ${context.sourceCode.text.includes("<h1>Hello</h1>")}`,
                `services: ${parserServices.isSvelte === true}`,
                `filePath: ${parserServices.filePath === context.filename}`,
                `parserOption: ${parserServices.flavor === "svelte-stub"}`,
              ].join("; "),
              node: SPAN,
            });
          },
        };
      },
    },
  },
};

export default plugin;
