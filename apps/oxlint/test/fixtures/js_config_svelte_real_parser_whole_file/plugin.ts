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
    name: "real-svelte",
  },
  rules: {
    canary: {
      create(context) {
        let sawElement = false;
        let elementName = "none";

        return {
          SvelteElement(node) {
            if (sawElement) return;
            sawElement = true;

            const name = (node as { name?: { name?: string } }).name;
            if (typeof name?.name === "string") {
              elementName = name.name;
            }
          },
          "Program:exit"() {
            const parserServices = context.sourceCode.parserServices as {
              isSvelte?: unknown;
              getStyleContext?: unknown;
            };

            context.report({
              message: [
                `parser:${context.languageOptions.parser?.name === "svelte-eslint-parser"}`,
                `services:${parserServices.isSvelte === true}`,
                `style:${typeof parserServices.getStyleContext === "function"}`,
                `element:${sawElement ? elementName : "none"}`,
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
