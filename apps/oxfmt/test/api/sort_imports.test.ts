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
  
  it("should merge duplicate imports from same source", async () => {
    const input = `
import { join } from "node:path";
import { styleText } from "node:util";
import { join, resolve } from "node:path";
    `;

    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toStrictEqual(
      `
import { join, resolve } from "node:path";
import { styleText } from "node:util";
`.trimStart(),
    );

    expect(result.errors).toStrictEqual([]);
  });

  it("should merge two imports with different specifiers", async () => {
    const input = `
import { a } from "x";
import { b } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import { a, b } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should deduplicate overlapping specifiers", async () => {
    const input = `
import { a } from "x";
import { a, b } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import { a, b } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should merge three imports from same source", async () => {
    const input = `
import { a } from "x";
import { b } from "x";
import { c } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import { a, b, c } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve aliases when merging", async () => {
    const input = `
import { a as b } from "x";
import { c } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import { a as b, c } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve per-specifier type annotations when merging", async () => {
    const input = `
import { type A } from "x";
import { b } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import { type A, b } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should merge type imports with same source", async () => {
    const input = `
import type { A } from "x";
import type { B } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import type { A, B } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should not merge type and value imports from same source", async () => {
    const input = `
import type { A } from "x";
import { b } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import type { A } from "x";
import { b } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should not merge default import with named import", async () => {
    const input = `
import foo from "x";
import { bar } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import foo from "x";
import { bar } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });

  it("should not merge namespace import with named import", async () => {
    const input = `
import * as ns from "x";
import { bar } from "x";
`;
    const result = await format("a.ts", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toBe(
      `
import * as ns from "x";
import { bar } from "x";
`.trimStart(),
    );
    expect(result.errors).toStrictEqual([]);
  });
});
