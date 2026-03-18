import { readFileSync } from "node:fs";

import type { Plugin } from "#oxlint/plugins";

function findPhysicalRange(filename: string, needle: string): [number, number] | null {
  const sourceText = readFileSync(filename, "utf8");
  const start = sourceText.indexOf(needle);
  if (start === -1) return null;
  return [start, start + needle.length];
}

const plugin: Plugin = {
  meta: {
    name: "vue-physical-plugin",
  },
  rules: {
    "report-template": {
      create(context) {
        return {
          Program() {
            const range = findPhysicalRange(context.physicalFilename, "template-bad");
            if (range === null) return;

            context.report({
              message: "Unexpected template marker",
              physicalRange: range,
            });
          },
        };
      },
    },
  },
};

export default plugin;
