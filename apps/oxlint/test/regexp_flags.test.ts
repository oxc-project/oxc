import { expect, it } from "vitest";
import { RuleTester } from "../src-js/package/rule_tester.ts";

import type { Rule } from "../src-js/plugins.ts";

RuleTester.describe = (_name, fn) => fn();
RuleTester.it = (_name, fn) => fn();

it("preserves source flag order for plugin diagnostics and fixes", () => {
  const rule: Rule = {
    meta: { fixable: "code" },
    create(context) {
      return {
        Literal(node) {
          if (!("regex" in node)) return;
          const { flags } = node.regex;
          const sorted = flags.split("").toSorted().join("");
          expect(node.value).toBeInstanceOf(RegExp);
          expect((node.value as RegExp).flags).toBe(sorted);
          if (flags === sorted) return;
          context.report({
            node,
            message: `Unsorted flags: ${flags}`,
            fix: (fixer) => fixer.replaceTextRange([node.end - flags.length, node.end], sorted),
          });
        },
      };
    },
  };
  expect(() =>
    new RuleTester().run("sort-flags", rule, {
      valid: ["/\\w/gimsvy"],
      invalid: [
        {
          code: "/\\w/yvsimg",
          output: "/\\w/gimsvy",
          errors: [{ message: "Unsorted flags: yvsimg" }],
        },
      ],
    }),
  ).not.toThrow();
});
