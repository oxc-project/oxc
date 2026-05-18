import type { Plugin } from "#oxlint/plugins";

const plugin: Plugin = {
  meta: {
    name: "empty-lines-plugin",
  },
  rules: {
    "no-multiple-empty-lines": {
      meta: {
        schema: false,
      },
      create(context) {
        const options = (context.options[0] ?? {}) as Record<string, number>;
        const max = options.max ?? 2;
        const maxBOF = options.maxBOF ?? max;

        return {
          Program() {
            const { lines } = context.sourceCode;

            // Check BOF blank lines
            let bofBlankLines = 0;
            for (const line of lines) {
              if (line.trim() === "") {
                bofBlankLines++;
              } else {
                break;
              }
            }
            if (bofBlankLines > maxBOF) {
              context.report({
                message: `Too many blank lines at the beginning of file. Max of ${maxBOF} allowed.`,
                loc: { line: 1, column: 0 },
              });
            }

            // Check consecutive blank lines
            let consecutiveBlank = 0;
            for (let i = 0; i < lines.length; i++) {
              if (lines[i].trim() === "") {
                consecutiveBlank++;
              } else {
                if (consecutiveBlank > max) {
                  context.report({
                    message: `More than ${max} blank line not allowed.`,
                    loc: { line: i + 1, column: 0 },
                  });
                }
                consecutiveBlank = 0;
              }
            }
          },
        };
      },
    },
  },
};

export default plugin;
