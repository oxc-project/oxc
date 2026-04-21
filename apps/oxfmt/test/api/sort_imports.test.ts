import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Sort imports", () => {
  it("should sort with customGroups", async () => {
    const input = `import { foo } from "./foo";
import { util } from "~/utils/util";
import { store } from "~/stores/store";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {
        newlinesBetween: false,
        customGroups: [
          { elementNamePattern: ["~/stores/**"], groupName: "stores" },
          { elementNamePattern: ["~/utils/**"], groupName: "utils" },
        ],
        groups: ["stores", "utils", "sibling"],
      },
    });

    expect(result.code).toBe(
      `
import { store } from "~/stores/store";
import { util } from "~/utils/util";
import { foo } from "./foo";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort with customGroups using selector and modifiers", async () => {
    const input = `import { bar } from "@scope/bar";
import type { FooType } from "@scope/foo";
import { foo } from "@scope/foo";
import type { BarType } from "@scope/bar";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {
        customGroups: [
          {
            groupName: "scope-types",
            elementNamePattern: ["@scope/**"],
            modifiers: ["type"],
          },
          {
            groupName: "scope-values",
            elementNamePattern: ["@scope/**"],
            modifiers: ["value"],
          },
        ],
        groups: ["scope-types", "scope-values", "unknown"],
      },
    });

    expect(result.code).toBe(
      `
import type { BarType } from "@scope/bar";
import type { FooType } from "@scope/foo";

import { bar } from "@scope/bar";
import { foo } from "@scope/foo";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });
});
