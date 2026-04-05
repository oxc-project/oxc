import type { Plugin } from "#oxlint/plugins";

type SvelteLiteralNode = {
  type?: string;
  value?: string;
};

type SvelteAttributeNode = {
  key?: { name?: string };
  value?: SvelteLiteralNode[];
};

const plugin: Plugin = {
  meta: {
    name: "real-svelte-fixes",
  },
  rules: {
    "whole-file-edits": {
      meta: {
        fixable: "code",
        hasSuggestions: true,
      },
      create(context) {
        const hasParserServices = context.sourceCode.parserServices.isSvelte === true;

        return {
          SvelteAttribute(node) {
            const attribute = node as SvelteAttributeNode;
            const literal = attribute.value?.[0];

            if (
              attribute.key?.name !== "class" ||
              literal?.type !== "SvelteLiteral" ||
              literal.value !== "greeting"
            ) {
              return;
            }

            context.report({
              node: literal as any,
              message: `fix class attribute; parserServices: ${hasParserServices}`,
              fix(fixer) {
                return fixer.replaceText(literal as any, "welcome");
              },
            });
          },
          SvelteText(node) {
            if ((node as { value?: string }).value !== "Hello") return;

            context.report({
              node,
              message: `suggest greeting text; parserServices: ${hasParserServices}`,
              suggest: [
                {
                  desc: "Use Hi",
                  fix(fixer) {
                    return fixer.replaceText(node, "Hi");
                  },
                },
                {
                  desc: "Use Hey",
                  fix(fixer) {
                    return fixer.replaceText(node, "Hey");
                  },
                },
              ],
            });
          },
        };
      },
    },
  },
};

export default plugin;
