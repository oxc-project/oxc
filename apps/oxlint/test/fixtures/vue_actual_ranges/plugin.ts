import { readFileSync } from "node:fs";

import type { Plugin } from "#oxlint/plugins";

function findActualRange(filename: string, needle: string): [number, number] | null {
  const sourceText = readFileSync(filename, "utf8");
  const start = sourceText.indexOf(needle);
  if (start === -1) return null;
  return [start, start + needle.length];
}

const plugin: Plugin = {
  meta: {
    name: "vue-actual-plugin",
  },
  rules: {
    "report-template": {
      create(context) {
        return {
          Program() {
            const range = findActualRange(context.physicalFilename, "template-bad");
            if (range === null) return;

            context.report({
              message: "Unexpected template marker",
              actualRange: range,
            });
          },
        };
      },
    },
  },
};

export default plugin;
