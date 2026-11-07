import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "vue-actual-text-plugin",
  },
  rules: {
    "actual-text": {
      create(context) {
        return {
          Program(node) {
            const { text, actualText, actualOffset } = context.sourceCode;
            const sectionMatches =
              actualText.slice(actualOffset, actualOffset + text.length) === text;

            context.report({
              message:
                `actualOffset: ${actualOffset}, isWholeFile: ${actualText === text}, `
                + `sectionMatches: ${sectionMatches}`,
              node,
            });

            const start = actualText.indexOf("template-bad");
            if (start === -1) return;

            context.report({
              message: "Unexpected template marker",
              actualRange: [start, start + "template-bad".length],
            });
          },
        };
      },
    },
  },
};

export default plugin;
