import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("arrayWrap", () => {
  describe("preserve (default)", () => {
    it("single-line stays single-line", async () => {
      const input = "const x = [1, 2, 3];\n";
      const result = await format("a.ts", input, { arrayWrap: "preserve" });
      expect(result.code).toBe("const x = [1, 2, 3];\n");
      expect(result.errors).toStrictEqual([]);
    });

    it("multiline with objects stays multiline", async () => {
      const input = `const x = [
  { a: 1, b: 2 },
  { a: 3, b: 4 },
];
`;
      const result = await format("a.ts", input, { arrayWrap: "preserve" });
      expect(result.code).toBe(`const x = [
  { a: 1, b: 2 },
  { a: 3, b: 4 },
];
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("default (no option) behaves like preserve", async () => {
      const input = "const x = [1, 2, 3];\n";
      const result = await format("a.ts", input);
      expect(result.code).toBe("const x = [1, 2, 3];\n");
      expect(result.errors).toStrictEqual([]);
    });
  });

  describe("collapse", () => {
    it("collapses multiline to single line when fits", async () => {
      const input = `const x = [
  1,
  2,
  3,
];
`;
      const result = await format("a.ts", input, { arrayWrap: "collapse" });
      expect(result.code).toBe("const x = [1, 2, 3];\n");
      expect(result.errors).toStrictEqual([]);
    });

    it("stays multiline when exceeds printWidth", async () => {
      const input = `const x = [
  "aaaaaaaaaa",
  "bbbbbbbbbb",
  "cccccccccc",
  "dddddddddd",
  "eeeeeeeeee",
];
`;
      const result = await format("a.ts", input, {
        arrayWrap: "collapse",
        printWidth: 40,
      });
      expect(result.code).toBe(`const x = [
  "aaaaaaaaaa",
  "bbbbbbbbbb",
  "cccccccccc",
  "dddddddddd",
  "eeeeeeeeee",
];
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("empty array stays flat", async () => {
      const input = "const x = [];\n";
      const result = await format("a.ts", input, { arrayWrap: "collapse" });
      expect(result.code).toBe("const x = [];\n");
      expect(result.errors).toStrictEqual([]);
    });
  });

  describe("minElementsToWrap", () => {
    it("below threshold stays flat", async () => {
      const input = "const x = [1];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      expect(result.code).toBe("const x = [1];\n");
      expect(result.errors).toStrictEqual([]);
    });

    it("at threshold forces multiline", async () => {
      const input = "const x = [1, 2];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      expect(result.code).toBe(`const x = [
  1,
  2,
];
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("above threshold forces multiline", async () => {
      const input = "const x = [1, 2, 3];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      expect(result.code).toBe(`const x = [
  1,
  2,
  3,
];
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("empty array stays flat", async () => {
      const input = "const x = [];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      expect(result.code).toBe("const x = [];\n");
      expect(result.errors).toStrictEqual([]);
    });

    it("below threshold preserves multiline formatting", async () => {
      const input = `const x = [
  z.string(),
  z.array(ContentPartSchema),
];
`;
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 3 },
      });
      expect(result.code).toBe(`const x = [
  z.string(),
  z.array(ContentPartSchema),
];
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("below threshold preserves multiline numeric arrays", async () => {
      const input = `const RETRYABLE_STATUS_CODES = new Set([
  429,
  503,
]);
`;
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 3 },
      });
      expect(result.code).toBe(`const RETRYABLE_STATUS_CODES = new Set([
  429,
  503,
]);
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("below threshold collapses single-line formatting", async () => {
      const input = "const x = [1, 2];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 3 },
      });
      expect(result.code).toBe("const x = [1, 2];\n");
      expect(result.errors).toStrictEqual([]);
    });

    it("nested arrays are evaluated independently", async () => {
      const input = "const x = [[1, 2], [3]];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      // Outer has 2 elements (>= threshold) -> expands
      // Inner [1, 2] has 2 elements (>= threshold) -> also expands
      // Inner [3] has 1 element (< threshold) -> stays flat
      expect(result.code).toBe(`const x = [
  [
    1,
    2,
  ],
  [3],
];
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("inner arrays above threshold also expand", async () => {
      const input = "const x = [[1, 2, 3]];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      expect(result.code).toBe(`const x = [
  [
    1,
    2,
    3,
  ],
];
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("destructuring patterns also forced multiline", async () => {
      const input = "const [a, b] = values;\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      expect(result.code).toBe(`const [
  a,
  b,
] = values;
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("below threshold preserves multiline destructuring", async () => {
      const input = `const [
  first,
  second,
] = values;
`;
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 3 },
      });
      expect(result.code).toBe(`const [
  first,
  second,
] = values;
`);
      expect(result.errors).toStrictEqual([]);
    });

    it("formatting is idempotent", async () => {
      const input = "const x = [1, 2, 3];\n";
      const opts = { arrayWrap: { minElementsToWrap: 2 } } as const;
      const first = await format("a.ts", input, opts);
      const second = await format("a.ts", first.code, opts);
      expect(second.code).toBe(first.code);
      expect(second.errors).toStrictEqual([]);
    });

    it("spread elements count as elements", async () => {
      const input = "const x = [1, ...rest];\n";
      const result = await format("a.ts", input, {
        arrayWrap: { minElementsToWrap: 2 },
      });
      expect(result.code).toBe(`const x = [
  1,
  ...rest,
];
`);
      expect(result.errors).toStrictEqual([]);
    });
  });
});
