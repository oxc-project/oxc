import { definePlugin } from "oxlint";
import type { Node } from "#oxlint";

const SPAN: Node = {
  start: 0,
  end: 0,
  range: [0, 0],
  loc: {
    start: { line: 0, column: 0 },
    end: { line: 0, column: 0 },
  },
};

export default definePlugin({
  meta: {
    name: "test-plugin-options",
  },
  rules: {
    "check-options": {
      create(context) {
        context.report({
          message: JSON.stringify(context.options, null, 2),
          node: SPAN,
        });
        return {};
      },
    },
  },
});
