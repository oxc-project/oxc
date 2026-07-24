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
});
