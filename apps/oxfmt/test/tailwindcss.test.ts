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

  test("should preserve whitespace when tailwindPreserveWhitespace is true", async () => {
    // Input with leading/trailing whitespace in class string
    const input = `const A = <div className="  p-4 flex  ">Hello</div>;`;

    // Without tailwindPreserveWhitespace, whitespace should be trimmed
    const resultWithoutOption = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });
    expect(resultWithoutOption.code).toContain('className="flex p-4"');

    // With tailwindPreserveWhitespace: true, whitespace should be preserved
    const resultWithOption = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindPreserveWhitespace: true,
      },
    });
    // Whitespace should be preserved around the sorted classes
    expect(resultWithOption.code).toContain('className="  flex p-4  "');
    expect(resultWithOption.errors).toStrictEqual([]);
  });

  test("should remove duplicates by default", async () => {
    // Input with duplicate class names
    const input = `const A = <div className="flex p-4 flex p-4">Hello</div>;`;

    // By default, duplicates should be removed
    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Should only have one instance of each class
    expect(result.code).toContain('className="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  test("should preserve duplicates when tailwindPreserveDuplicates is true", async () => {
    // Input with duplicate class names
    const input = `const A = <div className="flex p-4 flex p-4">Hello</div>;`;

    // With tailwindPreserveDuplicates: true, duplicates should be preserved
    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindPreserveDuplicates: true,
      },
    });

    // Duplicates should be preserved (sorted but kept)
    expect(result.code).toContain('className="flex flex p-4 p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  // Template literal tests for ignoreFirst/ignoreLast behavior
  test("should handle template literal with expressions - no spacing around expression", async () => {
    // No space between classes and expressions - adjacent classes should NOT be sorted
    // "p-4" is last in first quasi (no trailing space) -> ignored
    // "flex" is first in second quasi (no leading space) -> ignored
    const input = `const A = <div className={\`p-4\${x}flex\`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Both "p-4" and "flex" are adjacent to expressions - should remain untouched
    expect(result.code).toContain("`p-4${x}flex`");
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle template literal with expressions - with spacing", async () => {
    // Spaces around classes - classes should be sorted
    const input = `const A = <div className={\`p-4 flex \${x} m-2 inline\`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // First quasi "p-4 flex " has trailing space, so all classes get sorted: "flex p-4 "
    // Second quasi " m-2 inline" - sorted per Tailwind order
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 inline");
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle template literal with partial spacing", async () => {
    // First quasi has trailing space, second doesn't start with space
    // "p-4 flex " -> all sortable
    // "m-2 inline" -> "m-2" is first without leading space, should be ignored
    const input = `const A = <div className={\`p-4 flex \${x}m-2 inline\`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // First quasi should be sorted: "flex p-4 "
    expect(result.code).toContain("flex p-4");
    // "m-2" is adjacent to ${x} (no space) - should stay first, "inline" can be sorted relative to rest
    // The expected result is: `flex p-4 ${x}m-2 inline`
    expect(result.code).toContain("${x}m-2");
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle simple template literal without expressions", async () => {
    // Template literal without expressions should be treated like string literal
    const input = `const A = <div className={\`p-4 flex\`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain("`flex p-4`");
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle template literal with tailwindFunctions", async () => {
    const input = `const A = <div className={clsx(\`p-4 flex \${x} m-2 inline\`)}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindFunctions: ["clsx"],
      },
    });

    // Classes should be sorted within each quasi
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 inline");
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle template literal with line breaks", async () => {
    // Template literal with line breaks between classes
    const input = `const A = <div className={\`
      p-4 flex
      \${x}
      m-2 inline
    \`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Classes should be sorted within each quasi, line breaks preserved
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 inline");
    expect(result.errors).toStrictEqual([]);
  });

  test("should preserve spaces between expressions", async () => {
    // Template literal with only spaces between expressions should not lose them
    const input = "const A = <div className={`${a} ${b} ${c}`}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Spaces between expressions should be preserved
    expect(result.code).toContain("${a} ${b} ${c}");
    expect(result.errors).toStrictEqual([]);
  });

  test("should normalize multiple spaces around expressions to single space", async () => {
    // Template literal with multiple spaces around expression
    const input =
      "const A = <div className={`flex items-center   ${variant}   bg-blue-500 p-4`}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Multiple spaces should be collapsed to single space
    expect(result.code).toContain(
      "flex items-center ${variant} bg-blue-500 p-4"
    );
    expect(result.errors).toStrictEqual([]);
  });

  test("should collapse whitespace in template literal with line breaks when tailwindPreserveWhitespace is false", async () => {
    // Template literal with line breaks - whitespace should be collapsed
    const input = `const A = <div className={\`
      p-4 flex
      \${x}
      m-2 inline
    \`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindPreserveWhitespace: false,
      },
    });

    // Leading/trailing whitespace in first/last quasi should be collapsed
    // First quasi starts with newline+spaces, last quasi ends with newline+spaces
    expect(result.code).toContain("`flex p-4");
    expect(result.code).toContain("m-2 inline`");
    expect(result.errors).toStrictEqual([]);
  });

  test("should preserve whitespace in template literal when tailwindPreserveWhitespace is true", async () => {
    // Template literal with leading/trailing whitespace in first and last quasis
    const input = `const A = <div className={\`  p-4 flex \${x} m-2 inline  \`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        tailwindPreserveWhitespace: true,
      },
    });

    // Whitespace should be preserved - first quasi keeps leading spaces, last keeps trailing
    expect(result.code).toContain("`  flex p-4");
    expect(result.code).toContain("m-2 inline  `");
    expect(result.errors).toStrictEqual([]);
  });
});
