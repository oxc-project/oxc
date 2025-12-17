import { format } from "../dist/index.js";
import { describe, expect, test } from "vitest";
import type { FormatOptions } from "../dist/index.js";

describe("API Tests", () => {
  test("`format()` function exists", () => {
    expect(typeof format).toBe("function");
  });

  test("should `format()` multiple times", async () => {
    const result1 = await format("a.ts", "const x:number=42");
    expect(result1.code).toBe("const x: number = 42;\n");
    expect(result1.errors).toStrictEqual([]);

    const result2 = await format("a.json", '{"key":           "value"}');
    expect(result2.code).toBe('{ "key": "value" }\n');
    expect(result2.errors).toStrictEqual([]);
  });

  test("should TS types and options work", async () => {
    const options: FormatOptions = {
      quoteProps: "as-needed", // Can be string literal
      printWidth: 120,
      semi: false,
      experimentalSortPackageJson: false,
      experimentalSortImports: {
        // Can be optional object
        partitionByComment: false,
      },
    };

    const result = await format("a.ts", "const x={'y':1}", options);
    expect(result.code).toBe("const x = { y: 1 }\n");
    expect(result.errors).toStrictEqual([]);

    const { errors } = await format("a.ts", "const x={'y':1}", {
      // @ts-expect-error: Test invalid options is validated
      semi: "invalid",
    });
    expect(errors.length).toBe(1);
  });
});
