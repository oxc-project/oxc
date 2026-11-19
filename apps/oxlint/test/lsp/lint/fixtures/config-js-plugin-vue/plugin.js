const MARKER = "template-bad";

const plugin = {
  meta: {
    name: "js-plugin",
  },
  rules: {
    "template-marker": {
      create(context) {
        return {
          Program() {
            const start = context.sourceCode.actualText.indexOf(MARKER);
            if (start === -1) return;

            context.report({
              message: "Unexpected template marker",
              actualRange: [start, start + MARKER.length],
            });
          },
        };
      },
    },
  },
};

export default plugin;
