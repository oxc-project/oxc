import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Tailwind CSS Sorting", () => {
  it("should sort Tailwind classes when experimentalTailwindcss is enabled", async () => {
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

  it("should resolve relative tailwindConfig paths", async () => {
    const cwd = process.cwd();
    const dir = await mkdtemp(join(tmpdir(), "oxfmt-tailwind-"));

    try {
      await writeFile(
        join(dir, "tailwind.config.js"),
        "module.exports = { content: [], theme: { extend: {} }, plugins: [] };",
      );

      const input = `const A = <div className="p-4 flex">Hello</div>;`;
      const result = await format("src/test.tsx", input, {
        experimentalTailwindcss: {
          config: join(relative(cwd, dir), "tailwind.config.js"),
        },
      });

      expect(result.code).toContain('className="flex p-4"');
      expect(result.errors).toStrictEqual([]);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  it("should NOT sort Tailwind classes when experimentalTailwindcss is disabled (default)", async () => {
    const input = `const A = <div className="p-4 flex bg-red-500 text-white">Hello</div>;`;

    const result = await format("test.tsx", input);

    // Original order should be preserved
    expect(result.code).toContain('className="p-4 flex bg-red-500 text-white"');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort multiple className attributes", async () => {
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

  it("should handle class attribute (not just className)", async () => {
    const input = `const A = <div class="p-4 flex">Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain('class="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  it("should accept experimentalTailwindcss as object with options", async () => {
    const input = `const A = <div className="p-4 flex">Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        preserveWhitespace: false,
        preserveDuplicates: false,
      },
    });

    // Should still sort when options object is provided
    expect(result.code).toContain('className="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  it("should respect attributes option for custom attributes", async () => {
    // By default, only 'class' and 'className' are sorted
    const input = `const A = <div myClassProp="p-4 flex">Hello</div>;`;

    // Without attributes, custom attribute should NOT be sorted
    const resultWithoutOption = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });
    expect(resultWithoutOption.code).toContain('myClassProp="p-4 flex"');

    // With attributes including 'myClassProp', it SHOULD be sorted
    const resultWithOption = await format("test.tsx", input, {
      experimentalTailwindcss: {
        attributes: ["myClassProp"],
      },
    });
    expect(resultWithOption.code).toContain('myClassProp="flex p-4"');
    expect(resultWithOption.errors).toStrictEqual([]);
  });

  it("should respect functions option for custom functions", async () => {
    // Test with clsx function call
    const input = `const A = <div className={clsx("p-4 flex")}>Hello</div>;`;

    // With functions including 'clsx', the string argument should be sorted
    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // The clsx argument should be sorted
    expect(result.code).toContain('clsx("flex p-4")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle multiple functions", async () => {
    const input = `
const A = (
  <div className={clsx("p-4 flex")}>
    <span className={cva("p-2 inline")}>Title</span>
  </div>
);`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx", "cva"],
      },
    });

    expect(result.code).toContain('clsx("flex p-4")');
    expect(result.code).toContain('cva("inline p-2")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort classes in member expression on tailwind function (clsx.foo(...))", async () => {
    const input = `const A = <div className={clsx.foo("p-4 flex")}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    expect(result.code).toContain('clsx.foo("flex p-4")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort classes in object member tailwind function (obj.clsx(...))", async () => {
    const input = `const A = <div className={obj.clsx("p-4 flex")}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["obj"],
      },
    });

    expect(result.code).toContain('obj.clsx("flex p-4")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort classes in chained call (foo().clsx(...))", async () => {
    const input = `const A = <div className={foo().clsx("p-4 flex")}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["foo"],
      },
    });

    expect(result.code).toContain('foo().clsx("flex p-4")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort classes in deeply nested member expression (a.b.c.clsx(...))", async () => {
    const input = `const A = <div className={a.b.c.clsx("p-4 flex")}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["a"],
      },
    });

    expect(result.code).toContain('a.b.c.clsx("flex p-4")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort classes in computed member expression (obj[key](...))", async () => {
    const input = `const A = <div className={obj[key]("p-4 flex")}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["obj"],
      },
    });

    expect(result.code).toContain('obj[key]("flex p-4")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve whitespace when preserveWhitespace is true", async () => {
    // Input with leading/trailing whitespace in class string
    const input = `const A = <div className="  p-4 flex  ">Hello</div>;`;

    // Without preserveWhitespace, whitespace should be trimmed
    const resultWithoutOption = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });
    expect(resultWithoutOption.code).toContain('className="flex p-4"');

    // With preserveWhitespace: true, whitespace should be preserved
    const resultWithOption = await format("test.tsx", input, {
      experimentalTailwindcss: {
        preserveWhitespace: true,
      },
    });
    // Whitespace should be preserved around the sorted classes
    expect(resultWithOption.code).toContain('className="  flex p-4  "');
    expect(resultWithOption.errors).toStrictEqual([]);
  });

  it("should remove duplicates by default", async () => {
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

  it("should preserve duplicates when preserveDuplicates is true", async () => {
    // Input with duplicate class names
    const input = `const A = <div className="flex p-4 flex p-4">Hello</div>;`;

    // With preserveDuplicates: true, duplicates should be preserved
    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        preserveDuplicates: true,
      },
    });

    // Duplicates should be preserved (sorted but kept)
    expect(result.code).toContain('className="flex flex p-4 p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  // Template literal tests for ignoreFirst/ignoreLast behavior
  it("should handle template literal with expressions - no spacing around expression", async () => {
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

  it("should handle template literal with expressions - with spacing", async () => {
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

  it("should handle template literal with partial spacing", async () => {
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

  it("should sort classes after ignored first class", async () => {
    // When expression is directly followed by a class (no space), that class is ignored
    // But subsequent classes should still be sorted with proper spacing
    const input = "const A = <div className={`flex ${variant}items-center p-4`}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // "items-center" is ignored (touching expression), "p-4" is sorted
    // Space should be preserved between ignored class and sorted content
    expect(result.code).toContain("${variant}items-center p-4");
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort classes before ignored last class", async () => {
    // When a class is directly followed by expression (no space), that class is ignored
    const input = "const A = <div className={`flex p-4${variant} items-center`}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // "p-4" is ignored (touching expression), "flex" is sorted
    // Space should be preserved between sorted content and ignored class
    expect(result.code).toContain("flex p-4${variant}");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle simple template literal without expressions", async () => {
    // Template literal without expressions should be treated like string literal
    const input = `const A = <div className={\`p-4 flex\`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain("`flex p-4`");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle template literal with functions", async () => {
    const input = `const A = <div className={clsx(\`p-4 flex \${x} m-2 inline\`)}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Classes should be sorted within each quasi
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 inline");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle template literal with line breaks", async () => {
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

  it("should preserve spaces between expressions", async () => {
    // Template literal with only spaces between expressions should not lose them
    const input = "const A = <div className={`${a} ${b} ${c}`}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Spaces between expressions should be preserved
    expect(result.code).toContain("${a} ${b} ${c}");
    expect(result.errors).toStrictEqual([]);
  });

  it("should normalize multiple spaces between expressions to single space", async () => {
    // Template literal with only multiple spaces between expressions
    const input = "const A = <div className={`${a}   ${b}   ${c}`}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Multiple spaces should be normalized to single space
    expect(result.code).toContain("${a} ${b} ${c}");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle template literal with only spaces (no classes)", async () => {
    // Template literal containing only spaces - normalized to single space
    const input = "const A = <div className={`   `}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Multiple spaces normalized to single space (like Prettier)
    expect(result.code).toContain("className={` `}");
    expect(result.errors).toStrictEqual([]);
  });

  it("should normalize multiple spaces around expressions to single space", async () => {
    // Template literal with multiple spaces around expression
    const input =
      "const A = <div className={`flex items-center   ${variant}   bg-blue-500 p-4`}>Hello</div>;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Multiple spaces should be collapsed to single space
    expect(result.code).toContain("flex items-center ${variant} bg-blue-500 p-4");
    expect(result.errors).toStrictEqual([]);
  });

  it("should collapse whitespace in template literal with line breaks when preserveWhitespace is false", async () => {
    // Template literal with line breaks - whitespace should be collapsed
    const input = `const A = <div className={\`
      p-4 flex
      \${x}
      m-2 inline
    \`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        preserveWhitespace: false,
      },
    });

    // Leading/trailing whitespace in first/last quasi should be collapsed
    // First quasi starts with newline+spaces, last quasi ends with newline+spaces
    expect(result.code).toContain("`flex p-4");
    expect(result.code).toContain("m-2 inline`");
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve whitespace in template literal when preserveWhitespace is true", async () => {
    // Template literal with leading/trailing whitespace in first and last quasis
    const input = `const A = <div className={\`  p-4 flex \${x} m-2 inline  \`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        preserveWhitespace: true,
      },
    });

    // Whitespace should be preserved - first quasi keeps leading spaces, last keeps trailing
    expect(result.code).toContain("`  flex p-4");
    expect(result.code).toContain("m-2 inline  `");
    expect(result.errors).toStrictEqual([]);
  });

  it("should collapse newlines to single space when preserveWhitespace is false (default)", async () => {
    const input = `<div className={\`flex
items-center
bg-blue-500
p-4
text-white
\${isActive ? \`font-bold
shadow-lg\` : "font-normal"}\`} />;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Newlines should be collapsed to single space, classes sorted
    expect(result.code).toMatchInlineSnapshot(`
      "<div
        className={\`flex items-center bg-blue-500 p-4 text-white \${
          isActive ? \`font-bold shadow-lg\` : "font-normal"
        }\`}
      />;
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve newlines when preserveWhitespace is true", async () => {
    const input = `<div className={\`flex
items-center
bg-blue-500
p-4
text-white
\${isActive ? \`font-bold
shadow-lg\` : "font-normal"}\`} />;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        preserveWhitespace: true,
      },
    });

    // Newlines should be preserved
    expect(result.code).toMatchInlineSnapshot(`
      "<div className={\`flex
      items-center
      bg-blue-500
      p-4
      text-white
      \${isActive ? \`font-bold
      shadow-lg\` : "font-normal"}\`} />;
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  // Tests for can_collapse_whitespace in template literal expressions
  // These test the whitespace preservation logic based on adjacent quasi whitespace

  it("should collapse leading space when quasi before ends with whitespace", async () => {
    // Quasi "header " ends with space, so leading space in string can be collapsed
    const input = "const A = <div className={`header ${isExtendable ? ' active' : ''}`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Leading space should be trimmed (quasi already provides separation)
    expect(result.code).toContain('`header ${isExtendable ? "active" : ""}`');
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve leading space when quasi before has no trailing whitespace", async () => {
    // Quasi "header" has NO trailing space, so leading space in string must be preserved
    const input = "const A = <div className={`header${isExtendable ? ' active' : ''}`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Leading space must be preserved (no separation from quasi)
    expect(result.code).toContain('`header${isExtendable ? " active" : ""}`');
    expect(result.errors).toStrictEqual([]);
  });

  it("should collapse trailing space when quasi after starts with whitespace", async () => {
    // Quasi " suffix" starts with space, so trailing space in string can be collapsed
    const input = "const A = <div className={`${condition ? 'active ' : ''} suffix`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Trailing space should be trimmed (quasi already provides separation)
    expect(result.code).toContain('${condition ? "active" : ""} suffix');
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve trailing space when quasi after has no leading whitespace", async () => {
    // Quasi "suffix" has NO leading space, so trailing space in string must be preserved
    const input = "const A = <div className={`${condition ? 'active ' : ''}suffix`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Trailing space must be preserved (no separation to quasi)
    expect(result.code).toContain('${condition ? "active " : ""}suffix');
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve both leading and trailing space when surrounded by non-whitespace quasis", async () => {
    // String " middle " is between quasis without whitespace on either side
    const input = "const A = <div className={`prefix${condition ? ' middle ' : ''}suffix`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Both spaces must be preserved
    expect(result.code).toContain('${condition ? " middle " : ""}');
    expect(result.errors).toStrictEqual([]);
  });

  // Tests for nested expressions inside template literals (PR #396 fixes)
  it("should not trim whitespace inside nested ternary string literals", async () => {
    // The leading space in ' header-extendable' should NOT be trimmed
    // because the quasi before the expression doesn't end with whitespace
    const input =
      "const A = <div className={`header${isExtendable ? ' header-extendable' : ''}`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Space should be preserved in the ternary string (formatter uses double quotes)
    expect(result.code).toContain('`header${isExtendable ? " header-extendable" : ""}`');
    expect(result.errors).toStrictEqual([]);
  });

  it("should not trim whitespace inside concat expressions", async () => {
    // Spaces inside concat expressions should be preserved
    const input = "const A = <div className={a + ' p-4 ' + b} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Spaces in the middle of concat should be preserved (formatter uses double quotes)
    expect(result.code).toContain('a + " p-4 " + b');
    expect(result.errors).toStrictEqual([]);
  });

  // Tests for template literals in binary expressions
  it("should sort template literal on right side of binary expression", async () => {
    const input = "const A = <div className={a + ` p-4 flex `} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Template literal should be sorted, spaces preserved at boundaries
    expect(result.code).toContain("a + ` flex p-4`");
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort template literal on left side of binary expression", async () => {
    const input = "const A = <div className={` p-4 flex ` + b} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Template literal should be sorted, spaces preserved at boundaries
    expect(result.code).toContain("`flex p-4 ` + b");
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort template literal in middle of binary expression", async () => {
    const input = "const A = <div className={a + ` p-4 flex ` + b} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Template literal should be sorted, spaces preserved at boundaries
    expect(result.code).toContain("a + ` flex p-4 ` + b");
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort template literal with expressions in binary expression", async () => {
    const input = "const A = <div className={a + ` p-4 ${x} flex ` + b} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Classes should be sorted, expression preserved
    expect(result.code).toContain("a + ` p-4 ${x} flex ` + b");
    expect(result.errors).toStrictEqual([]);
  });

  it("should not trim whitespace in nested ternary with leading space only", async () => {
    // Issue #337: leading space removed incorrectly
    const input =
      "const A = <div className={`MuiApi-item-root${isExtendable ? ' MuiApi-item-header-extendable' : ''}`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Leading space must be preserved (formatter uses double quotes)
    expect(result.code).toContain(
      '`MuiApi-item-root${isExtendable ? " MuiApi-item-header-extendable" : ""}`',
    );
    expect(result.errors).toStrictEqual([]);
  });

  // Tests for nested template literals (inside ternary, concat, etc.)
  it("should sort template literal inside ternary expression", async () => {
    // Template literal nested inside a ternary should be sorted
    const input = "const A = <div className={condition ? `p-4 flex` : `m-2 grid`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Both template literals should be sorted
    expect(result.code).toContain("`flex p-4`");
    expect(result.code).toContain("`m-2 grid`"); // m-2 grid is already in correct Tailwind order
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort template literal with expressions inside ternary", async () => {
    // Template literal with expressions, nested inside ternary
    const input = "const A = <div className={condition ? `p-4 flex ${x} m-2 inline` : `grid`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Classes should be sorted within quasis
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 inline");
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort template literal inside logical expression", async () => {
    // Template literal nested inside logical OR
    const input = "const A = <div className={variant || `p-4 flex`} />;";

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain("`flex p-4`");
    expect(result.errors).toStrictEqual([]);
  });

  // Tests for nested call expressions - strings inside non-Tailwind calls should NOT be sorted
  // Issue: https://github.com/tailwindlabs/prettier-plugin-tailwindcss/issues/426
  it("should NOT sort strings inside nested non-Tailwind call expressions", async () => {
    // The "\n" inside value.includes() should NOT be treated as a Tailwind class
    const input = `const A = <div className={classNames(
  "bg-red-500",
  value.includes("\\n") ? "pt-2" : "pt-1"
)} />;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["classNames"],
      },
    });

    // The "\n" should remain as "\n", not be corrupted
    expect(result.code).toContain('value.includes("\\n")');
    // Direct Tailwind class strings should still be sorted (single classes, no sorting needed here)
    expect(result.code).toContain('"bg-red-500"');
    expect(result.code).toContain('"pt-2"');
    expect(result.code).toContain('"pt-1"');
    expect(result.errors).toStrictEqual([]);
  });

  it("should NOT sort strings inside deeply nested call expressions", async () => {
    // Multiple levels of nested calls - strings in inner calls should be preserved
    const input = `const A = <div className={clsx(
  "flex p-4",
  arr.filter(x => x.startsWith("prefix")).join(" ")
)} />;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Direct Tailwind classes should be sorted
    expect(result.code).toContain('"flex p-4"');
    // Strings inside nested calls should be preserved
    expect(result.code).toContain('startsWith("prefix")');
    expect(result.code).toContain('join(" ")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort Tailwind classes but preserve strings in member expression calls", async () => {
    // str.split(" ") - the " " should not be sorted as Tailwind
    const input = `const A = <div className={clsx("p-4 flex", input.split(" ").map(x => x))} />;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Direct Tailwind classes should be sorted
    expect(result.code).toContain('"flex p-4"');
    // The " " in split should be preserved
    expect(result.code).toContain('split(" ")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve escape sequences in nested call expressions", async () => {
    // Various escape sequences should be preserved in nested calls
    const input = `const A = <div className={clsx(
  "flex",
  str.includes("\\t") && "p-4",
  str.match(/\\s/) && "m-2"
)} />;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Escape sequences should be preserved
    expect(result.code).toContain('includes("\\t")');
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle empty class string", async () => {
    const input = `const A = <div className="">Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Empty string should remain empty
    expect(result.code).toContain('className=""');
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle whitespace-only class string", async () => {
    const input = `const A = <div className="   ">Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // Whitespace-only should be normalized to single space
    expect(result.code).toContain('className=" "');
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle deeply nested template literals (3 levels)", async () => {
    const input = `const A = <div className={\`p-4 flex \${a ? \`m-2 grid \${b ? \`inline-block\` : ""}\` : ""}\`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
    });

    // All levels should be sorted
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 grid");
    expect(result.code).toContain("inline-block");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle template literal inside function inside template literal", async () => {
    const input = `const A = <div className={\`p-4 flex \${clsx(\`m-2 grid\`)}\`}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Both template literals should be sorted
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 grid");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle multiple function calls with template literals", async () => {
    const input = `const A = <div className={clsx(\`p-4 flex\`, cva(\`m-2 grid\`))}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx", "cva"],
      },
    });

    // Both template literals should be sorted
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 grid");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle array of template literals in function call", async () => {
    const input = `const A = <div className={clsx([\`p-4 flex\`, \`m-2 grid\`])}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Both template literals should be sorted
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 grid");
    expect(result.errors).toStrictEqual([]);
  });

  it("should handle object with template literal values", async () => {
    const input = `const A = <div className={clsx({[\`p-4 flex\`]: true, [\`m-2 grid\`]: false})}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Template literals should be sorted
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 grid");
    expect(result.errors).toStrictEqual([]);
  });

  // Issue: https://github.com/oxc-project/oxc/issues/18712
  it("should sort classes in object property keys (string literals)", async () => {
    // Object keys with class names should be sorted like other strings
    const input = `const A = <div className={cn({ 'p-[2px] elevation-elevated-selected': true })}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["cn"],
      },
    });

    // Classes in object keys should be sorted
    expect(result.code).toContain("elevation-elevated-selected p-[2px]");
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort classes in multiple object property keys", async () => {
    const input = `const A = <div className={clsx({
      'p-4 flex': condition1,
      'm-2 grid': condition2,
      'text-white bg-red-500': condition3
    })}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // All object key class strings should be sorted
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain("m-2 grid");
    expect(result.code).toContain("bg-red-500 text-white");
    expect(result.errors).toStrictEqual([]);
  });

  it("should NOT sort strings inside non-Tailwind function calls in object keys", async () => {
    // Strings in nested function calls should not be sorted
    const input = `const A = <div className={clsx({
      'p-4 flex': value.includes(" "),
      'error-class': true
    })}>Hello</div>;`;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    // Object key classes should be sorted, but the "\n" in includes() should be preserved
    expect(result.code).toContain("flex p-4");
    expect(result.code).toContain('includes(" ")');
    expect(result.errors).toStrictEqual([]);
  });
});

describe("Tailwind CSS Sorting (Non-JS Files)", () => {
  it("should sort Tailwind classes in HTML files", async () => {
    const input = `<div class="p-4 flex bg-red-500 text-white">Hello</div>`;

    const result = await format("test.html", input, {
      experimentalTailwindcss: {},
    });

    // After sorting, flex should come before p-4 (display before spacing)
    expect(result.code).toContain('class="flex');
    expect(result.code).not.toContain('class="p-4 flex');
    expect(result.errors).toStrictEqual([]);
  });

  it("should NOT sort Tailwind classes in HTML when experimentalTailwindcss is disabled", async () => {
    const input = `<div class="p-4 flex bg-red-500 text-white">Hello</div>`;

    const result = await format("test.html", input);

    // Original order should be preserved
    expect(result.code).toContain('class="p-4 flex bg-red-500 text-white"');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort Tailwind classes in Vue SFC files", async () => {
    const input = `<template>
  <div class="p-4 flex bg-red-500 text-white">Hello</div>
</template>`;

    const result = await format("test.vue", input, {
      experimentalTailwindcss: {},
    });

    // After sorting, flex should come before p-4
    expect(result.code).toContain('class="flex');
    expect(result.code).not.toContain('class="p-4 flex');
    expect(result.errors).toStrictEqual([]);
  });

  it("should sort multiple class attributes in HTML", async () => {
    const input = `<div class="p-4 flex">
  <span class="text-white bg-red-500">Title</span>
</div>`;

    const result = await format("test.html", input, {
      experimentalTailwindcss: {},
    });

    // Both class attributes should be sorted
    expect(result.code).toContain('class="flex p-4"');
    expect(result.code).toContain('class="bg-red-500 text-white"');
    expect(result.errors).toStrictEqual([]);
  });

  it("should respect attributes option in HTML", async () => {
    const input = `<div class="p-4 flex" data-classes="text-white bg-red-500">Hello</div>`;

    // Without attributes option, only class should be sorted
    const resultWithoutOption = await format("test.html", input, {
      experimentalTailwindcss: {},
    });
    expect(resultWithoutOption.code).toContain('class="flex p-4"');
    expect(resultWithoutOption.code).toContain('data-classes="text-white bg-red-500"'); // Not sorted

    // With attributes option, data-classes should also be sorted
    const resultWithOption = await format("test.html", input, {
      experimentalTailwindcss: {
        attributes: ["data-classes"],
      },
    });
    expect(resultWithOption.code).toContain('class="flex p-4"');
    expect(resultWithOption.code).toContain('data-classes="bg-red-500 text-white"'); // Sorted
    expect(resultWithOption.errors).toStrictEqual([]);
  });

  it("should respect functions option in Vue SFC", async () => {
    const input = `<template>
  <div :class="clsx('p-4 flex')">Hello</div>
</template>`;

    // With functions option, clsx argument should be sorted
    const result = await format("test.vue", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    expect(result.code).toContain("clsx('flex p-4')");
    expect(result.errors).toStrictEqual([]);
  });

  it("should remove duplicates in HTML by default", async () => {
    const input = `<div class="flex p-4 flex p-4">Hello</div>`;

    const result = await format("test.html", input, {
      experimentalTailwindcss: {},
    });

    // Duplicates should be removed
    expect(result.code).toContain('class="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  it("should preserve duplicates in HTML when preserveDuplicates is true", async () => {
    const input = `<div class="flex p-4 flex p-4">Hello</div>`;

    const result = await format("test.html", input, {
      experimentalTailwindcss: {
        preserveDuplicates: true,
      },
    });

    // Duplicates should be preserved
    expect(result.code).toContain('class="flex flex p-4 p-4"');
    expect(result.errors).toStrictEqual([]);
  });
});

describe("Tailwind CSS Sorting with `experimentalSortImports` enabled", () => {
  it("should sort Tailwind classes in default options", async () => {
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

  it("should preserve whitespace when preserveWhitespace is true", async () => {
    // Input with leading/trailing whitespace in class string
    const input = `const A = <div className="  p-4 flex  ">Hello</div>;`;

    // Without preserveWhitespace, whitespace should be trimmed
    const resultWithoutOption = await format("test.tsx", input, {
      experimentalTailwindcss: {},
      experimentalSortImports: {},
    });
    expect(resultWithoutOption.code).toContain('className="flex p-4"');

    // With preserveWhitespace: true, whitespace should be preserved
    const resultWithOption = await format("test.tsx", input, {
      experimentalTailwindcss: {
        preserveWhitespace: true,
      },
      experimentalSortImports: {},
    });
    // Whitespace should be preserved around the sorted classes
    expect(resultWithOption.code).toContain('className="  flex p-4  "');
    expect(resultWithOption.errors).toStrictEqual([]);
  });
});

describe("Tailwind CSS Sorting works with other options", () => {
  it("should keep quotes with `singleQuote: true`", async () => {
    const input = `
      <div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
      <div className={clsx("text-md before:content-['hello']")}>Hello</div>;
      <div className={showLandingPage ? "container pb-6" : 'hidden'}>title</div>
    `;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
      singleQuote: true,
    });

    expect(result.code).toMatchInlineSnapshot(`
      "<div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
      <div className={clsx("text-md before:content-['hello']")}>Hello</div>;
      <div className={showLandingPage ? 'container pb-6' : 'hidden'}>title</div>;
      "
    `);
  });

  it("should keep quotes with default `singleQuote`", async () => {
    const input = `
      <div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
      <div className={clsx("text-md before:content-['hello']")}>Hello</div>;
      <div className={showLandingPage ? "container pb-6" : 'hidden'}>title</div>
    `;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {
        functions: ["clsx"],
      },
    });

    expect(result.code).toMatchInlineSnapshot(`
      "<div className={clsx('text-md before:content-["hello"]')}>Hello</div>;
      <div className={clsx("text-md before:content-['hello']")}>Hello</div>;
      <div className={showLandingPage ? "container pb-6" : "hidden"}>title</div>;
      "
    `);
  });

  it("should handle quotes with `jsxSingleQuote: true` correctly", async () => {
    const input = `
        <div className="text-md before:content-['hello']">Hello</div>;
        <div className='text-md before:content-["hello"]'>Hello</div>;
    `;

    const result = await format("test.tsx", input, {
      experimentalTailwindcss: {},
      jsxSingleQuote: true,
    });

    expect(result.code).toMatchInlineSnapshot(`
      "<div className="text-md before:content-['hello']">Hello</div>;
      <div className='text-md before:content-["hello"]'>Hello</div>;
      "
    `);
  });
});

describe("Tailwind CSS Sorting in Embedded HTML (Tagged Template Literals)", () => {
  it("should sort Tailwind classes in html tagged template literal", async () => {
    const input = `const view = html\`<div class="p-4 flex bg-red-500">Hello</div>\`;`;

    const result = await format("test.ts", input, {
      experimentalTailwindcss: {},
    });

    // After sorting, flex should come before p-4 (display before spacing)
    expect(result.code).toContain('class="flex');
    expect(result.code).not.toContain('class="p-4 flex');
    expect(result.errors).toStrictEqual([]);
  });
});
