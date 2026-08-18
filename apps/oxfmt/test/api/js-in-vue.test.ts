import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// NOTE: For now, Vue files are partially handled by Prettier

describe("Format js-in-vue with prettier-plugin-oxfmt", () => {
  it("should format .vue w/ sort-imports", async () => {
    const input = `
<script lang="ts">
import z from "z";
  import a from "a";
    import m from "m";

</script>
<script lang="ts" setup>
import z from "z";
  import a from "a";
    import m from "m";

</script>
<template> <div>{{a+m+z}}</div> </template>
`;
    const result = await format("a.vue", input, {
      vueIndentScriptAndStyle: true,
      experimentalSortImports: {},
    });

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .vue w/ sort-tailwindcss", async () => {
    const input = `
<script setup>
import { ref } from "vue";
import clsx from "clsx";

const count = ref(0);
const cls = clsx("p-4 flex");
</script>
<template>
  <div class="flex p-4">{{count}}</div>
  <div class="p-4 flex">{{count}}</div>
</template>
`;
    const result = await format("a.vue", input, {
      vueIndentScriptAndStyle: true,
      experimentalSortImports: {},
      experimentalTailwindcss: { functions: ["clsx"] },
    });

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  // https://github.com/oxc-project/oxc/issues/20084
  it("should format .vue w/ template literal idempotently (vueIndentScriptAndStyle)", async () => {
    const input = `
<script setup>
const a = \`
  hello
  world
\`;
</script>
<template>
  <div>{{ a }}</div>
</template>
`;
    const result = await format("a.vue", input, {
      vueIndentScriptAndStyle: true,
    });

    // Format again to verify idempotency
    const result2 = await format("a.vue", result.code, {
      vueIndentScriptAndStyle: true,
    });

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
    expect(result2.code).toBe(result.code);
    expect(result2.errors).toStrictEqual([]);
  });

  it("should format .vue w/ template literal (no vueIndentScriptAndStyle)", async () => {
    const input = `
<script setup>
const a = \`
  hello
  world
\`;
</script>
<template>
  <div>{{ a }}</div>
</template>
`;
    const result = await format("a.vue", input);

    // Format again to verify idempotency
    const result2 = await format("a.vue", result.code);

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
    expect(result2.code).toBe(result.code);
    expect(result2.errors).toStrictEqual([]);
  });

  // gql-in-js-in-vue: the `oxc_formatter_graphql` IR's blank runs
  // (`exact_line_breaks`, part of the block string's VALUE) must survive the IR→Doc conversion
  // back to the Prettier host (encoded as that many hardlines, which Prettier never collapses).
  it("should preserve gql block-string blank lines through a .vue script", async () => {
    const input = `
<script setup>
const q = graphql\`
  """
  First paragraph.


  Second paragraph after two blanks.
  """
  type Query {
    hello: String
  }
\`;
</script>
`;
    const result = await format("a.vue", input);

    // Format again to verify idempotency
    const result2 = await format("a.vue", result.code);

    expect(result.code).toContain("First paragraph.\n\n\n  Second paragraph after two blanks.");
    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
    expect(result2.code).toBe(result.code);
    expect(result2.errors).toStrictEqual([]);
  });

  it('should format <script lang="tsx"> blocks', async () => {
    const input = `
<script lang="tsx">
export default {
  render( h ): VNode {return <div>{ this.foo   }</div>    },
}
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toContain("return <div>{this.foo}</div>;");
    expect(result.errors).toStrictEqual([]);
  });

  it('should format generic arrows in <script lang="ts"> blocks', async () => {
    const input = `
<script lang="ts">
export const identity=<T>(x:T):T=>x;
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toContain("export const identity = <T>(x: T): T => x;");
    expect(result.errors).toStrictEqual([]);
  });

  // NOTE: Trailing comma of a lone generic arrow param is grammar-keyed, not path-keyed:
  // plain-TS blocks behave like plain .ts files (comma removable).
  // Unlike Prettier, which keeps the comma for any non-.ts `opts.filepath`. (ts-in-vue)
  it('should drop the lone generic param comma in <script lang="ts"> blocks', async () => {
    const input = `
<script setup lang="ts">
const getOptions = <T = any,>(list: T[]) => list;
const identity = <T,>(x: T) => x;
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toContain("const getOptions = <T = any>(list: T[]) => list;");
    expect(result.code).toContain("const identity = <T>(x: T) => x;");
    expect(result.errors).toStrictEqual([]);
  });

  it('should keep the lone generic param comma in <script lang="tsx"> blocks', async () => {
    const input = `
<script lang="tsx">
const identity = <T,>(x: T) => x;
const el = <div>hi</div>;
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toContain("const identity = <T,>(x: T) => x;");
    expect(result.errors).toStrictEqual([]);
  });

  it('should keep the comma in a JSX-free lang="tsx" block', async () => {
    const input = `
<script setup lang="tsx">
const getOptions = <T = any,>(list: T[]) => list;
const identity = <T,>(x: T) => x;
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toContain("const getOptions = <T = any,>(list: T[]) => list;");
    expect(result.code).toContain("const identity = <T,>(x: T) => x;");
    expect(result.errors).toStrictEqual([]);
  });

  it('should detect lang="tsx" after a generic attribute containing `>`', async () => {
    const input = `
<script setup generic="T extends Record<string, string>" lang="tsx">
const pick = <U = T,>(x: U) => x;
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toContain("const pick = <U = T,>(x: U) => x;");
    expect(result.errors).toStrictEqual([]);
  });

  // https://github.com/oxc-project/oxc/issues/25568
  it("should not add a blank line after a dangling comment in an empty object", async () => {
    // The IR's hardline (comment terminator) + softline (before `}`) must print as a single break,
    // like the Rust printer's newline suppression at a line start.
    const input = `
<script setup>
const a = {
  // x
}
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toBe(`<script setup>
const a = {
  // x
};
</script>
`);
    expect(result.errors).toStrictEqual([]);
  });

  // https://github.com/oxc-project/oxc/issues/25569
  it("should dedent template literal interpolation to root inside a function", async () => {
    // The IR's dedent-to-root must survive the Doc conversion
    // (JSON cannot represent the `-Infinity` Prettier expects; it is restored JS-side).
    const input = `
<script setup>
const f = () => {
  s.value = \`
\${items ? Object.entries(items).map(([k, v]) => k + v).join("") : ""}
\`
}
</script>
`;
    const result = await format("a.vue", input, { printWidth: 80 });

    // Format again to verify idempotency
    const result2 = await format("a.vue", result.code, { printWidth: 80 });

    expect(result.code).toBe(`<script setup>
const f = () => {
  s.value = \`
\${
  items
    ? Object.entries(items)
        .map(([k, v]) => k + v)
        .join("")
    : ""
}
\`;
};
</script>
`);
    expect(result.errors).toStrictEqual([]);
    expect(result2.code).toBe(result.code);
    expect(result2.errors).toStrictEqual([]);
  });

  it("should not indent a comment-only script block", async () => {
    // The comment's leading IR `Space` must be dropped at the line start,
    // like the Rust printer does, or `/**` gains a spurious leading space.
    const input = `
<script lang="ts">
/**
 * Docs.
 */
</script>
`;
    const result = await format("a.vue", input);

    expect(result.code).toBe(`<script lang="ts">
/**
 * Docs.
 */
</script>
`);
    expect(result.errors).toStrictEqual([]);
  });
});
