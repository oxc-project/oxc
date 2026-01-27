import { describe, expect, test } from "vitest";
import { format } from "../../../dist/index.js";

describe("Vue <script> tag formatting via API", () => {
  test("should sort imports", async () => {
    const input = `<script>
import z from "z";
import a from "a";
const x = 1;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toContain('import a from "a";\nimport z from "z";');
    expect(result.errors).toStrictEqual([]);
  });

  test("should sort Tailwind classes in JSX", async () => {
    const input = `<script>
const x = <div className="p-4 flex">Hello</div>;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain('className="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  test("should sort both imports and Tailwind classes", async () => {
    const input = `<script>
import z from "z";
import a from "a";
const x = <div className="p-4 flex">Hello</div>;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      experimentalSortImports: {},
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain('import a from "a";\nimport z from "z";');
    expect(result.code).toContain('className="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });

  test('should sort imports in <script lang="ts">', async () => {
    const input = `<script lang="ts">
import z from "z";
import a from "a";
const x: number = 1;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toContain('import a from "a";\nimport z from "z";');
    expect(result.errors).toStrictEqual([]);
  });

  test("should sort imports in <script setup>", async () => {
    const input = `<script setup>
import z from "z";
import a from "a";
const x = 1;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toContain('import a from "a";\nimport z from "z";');
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle multiple <script> tags", async () => {
    const input = `<script>
import z from "z";
import a from "a";
</script>
<script setup>
import y from "y";
import b from "b";
const x = 1;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toContain('import a from "a";\nimport z from "z";');
    expect(result.code).toContain('import b from "b";\nimport y from "y";');
    expect(result.errors).toStrictEqual([]);
  });

  test("should format whitespace (basic formatting)", async () => {
    const input = `<script>
const   x   =   1;
const y=2;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {});

    expect(result.code).toContain("const x = 1;");
    expect(result.code).toContain("const y = 2;");
    expect(result.errors).toStrictEqual([]);
  });

  test("should respect semi option", async () => {
    const input = `<script>
const x = 1;
const y = 2;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      semi: false,
    });

    expect(result.code).toContain("const x = 1\n");
    expect(result.code).toContain("const y = 2\n");
    expect(result.errors).toStrictEqual([]);
  });

  test("should respect vueIndentScriptAndStyle option", async () => {
    const input = `<script>
const x = 1;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {
      vueIndentScriptAndStyle: true,
    });

    expect(result.code).toContain("<script>\n  const x = 1;\n</script>");
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle CSS-in-JS (styled-components)", async () => {
    const input = `<script>
import styled from "styled-components";
const Button = styled.button\`
  color:red;
  background:   blue;
\`;
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {});

    expect(result.code).toContain("styled.button`");
    expect(result.errors).toStrictEqual([]);
  });

  test("should handle syntax error gracefully", async () => {
    const input = `<script>
const x = {
  // missing closing brace
</script>
<template><div>Hello</div></template>`;

    const result = await format("test.vue", input, {});

    expect(result.code).toContain("const x = {");
    expect(result.code).toContain("// missing closing brace");
    expect(result.errors).toStrictEqual([]);
  });
});

describe("HTML <script> tag formatting via API", () => {
  test("should sort imports", async () => {
    const input = `<!DOCTYPE html>
<html>
<head>
  <script>
import z from "z";
import a from "a";
const x = 1;
  </script>
</head>
<body></body>
</html>`;

    const result = await format("test.html", input, {
      experimentalSortImports: {},
    });

    expect(result.code).toContain('import a from "a";');
    expect(result.code).toContain('import z from "z";');
    const aIndex = result.code.indexOf('import a from "a";');
    const zIndex = result.code.indexOf('import z from "z";');
    expect(aIndex).toBeLessThan(zIndex);
    expect(result.errors).toStrictEqual([]);
  });

  test("should sort Tailwind classes in JSX", async () => {
    const input = `<!DOCTYPE html>
<html>
<head>
  <script>
const x = <div className="p-4 flex">Hello</div>;
  </script>
</head>
<body></body>
</html>`;

    const result = await format("test.html", input, {
      experimentalTailwindcss: {},
    });

    expect(result.code).toContain('className="flex p-4"');
    expect(result.errors).toStrictEqual([]);
  });
});
