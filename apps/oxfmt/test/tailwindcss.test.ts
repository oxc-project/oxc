import { format } from "../dist/index.js";
import { describe, expect, test } from "vitest";

describe("Tailwind CSS Sorting", () => {
  test("should sort Tailwind classes when experimentalTailwindcss is enabled", async () => {
    // Unsorted: p-4 comes before flex
    const input = `const A = <div className="p-4 flex bg-red-500 text-white">Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // After sorting, flex should come before p-4 (display before spacing)
    // The exact order of bg-red-500 and text-white may vary by Tailwind version
    expect(result.code).toContain('className="flex');
    expect(result.code).not.toContain('className="p-4 flex'); // p-4 should not be before flex
    expect(result.errors).toStrictEqual([]);
  });

  test("should NOT sort Tailwind classes when experimentalTailwindcss is disabled (default)", async () => {
    const input = `const A = <div className="p-4 flex bg-red-500 text-white">Hello</div>;`;

    const result = await format("test.tsx", input);

    // Original order should be preserved
    expect(result.code).toContain('className="p-4 flex bg-red-500 text-white"');
    expect(result.errors).toStrictEqual([]);
  });

  test("should sort multiple className attributes", async () => {
    // Use classes that will definitely be reordered
    const input = `
const A = (
  <div className="p-4 flex">
    <span className="p-2 inline">Title</span>
  </div>
);`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Both className attributes should be sorted (display utilities before spacing)
    expect(result.code).toContain('className="flex p-4"');
    expect(result.code).toContain('className="inline p-2"');
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle class attribute (not just className)", async () => {
    const input = `const A = <div class="p-4 flex">Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain('class="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  test("should accept experimentalTailwindcss as object with options", async () => {
    const input = `const A = <div className="p-4 flex">Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindPreserveWhitespace: false,
        tailwindPreserveDuplicates: false,
      },
    });

    // Should still sort when options object is provided
    expect(result.code).toContain('className="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  test("should respect tailwindAttributes option for custom attributes", async () => {
    // By default, only 'class' and 'className' are sorted
    const input = `const A = <div myClassProp="p-4 flex">Hello</div>;`;

    // Without tailwindAttributes, custom attribute should NOT be sorted
    const resultWithoutOption = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });
    expect(resultWithoutOption.code).toContain('myClassProp="p-4 flex"');

    // With tailwindAttributes including 'myClassProp', it SHOULD be sorted
    const resultWithOption = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindAttributes: ["myClassProp"],
      },
    });
    expect(resultWithOption.code).toContain('myClassProp="flex p-4"');
    expect(resultWithOption.errors).toStrictEqual([]);
  });

  test("should respect tailwindFunctions option for custom functions", async () => {
    // Test with clsx function call
    const input = `const A = <div className={clsx("p-4 flex")}>Hello</div>;`;

    // With tailwindFunctions including 'clsx', the string argument should be sorted
    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindFunctions: ["clsx"],
      },
    });

    // The clsx argument should be sorted
    expect(result.code).toContain('clsx("flex p-4")');
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle multiple tailwindFunctions", async () => {
    const input = `
const A = (
  <div className={clsx("p-4 flex")}>
    <span className={cva("p-2 inline")}>Title</span>
  </div>
);`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindFunctions: ["clsx", "cva"],
      },
    });

    expect(result.code).toContain('clsx("flex p-4")');
    expect(result.code).toContain('cva("inline p-2")');
    expect(result.errors).toStrictEqual([]);
  });
});
