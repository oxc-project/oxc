import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Svelte", () => {
  it("should format a basic .svelte file", async () => {
    const code = `
<script>
let   count=0;
</script>

<button on:click={()=>count++}>{count}</button>

<style>button{color:red;}</style>
`.trim();

    const result = await format("App.svelte", code);
    expect(result.code).toMatchInlineSnapshot(`
      "<script>
        let count = 0;
      </script>

      <button on:click={() => count++}>{count}</button>

      <style>
        button {
          color: red;
        }
      </style>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should respect svelte options", async () => {
    const code = `
<style>div{color:red;}</style>

<div>hello</div>

<script>let x=1;</script>
`.trim();

    const result = await format("App.svelte", code, {
      svelte: {
        sortOrder: "options-styles-scripts-markup",
        indentScriptAndStyle: false,
      },
    });
    expect(result.code).toMatchInlineSnapshot(`
      "<style>
      div {
        color: red;
      }
      </style>

      <script>
      let x = 1;
      </script>

      <div>hello</div>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should format script with lang=ts", async () => {
    const code = `
<script lang="ts">
let   count:number=0;
</script>

<div>{count}</div>
`.trim();

    const result = await format("App.svelte", code);
    expect(result.code).toMatchInlineSnapshot(`
      "<script lang="ts">
        let count: number = 0;
      </script>

      <div>{count}</div>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should format svelte:options", async () => {
    const code = `
<svelte:options   customElement="my-el" />

<div>hello</div>
`.trim();

    const result = await format("App.svelte", code);
    expect(result.code).toMatchInlineSnapshot(`
      "<svelte:options customElement="my-el" />

      <div>hello</div>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should respect svelte strictMode option", async () => {
    const code = `<input />`;

    const result = await format("App.svelte", code, {
      svelte: { strictMode: true },
    });
    expect(result.code).toMatchInlineSnapshot(`
      "<input />
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should respect svelte allowShorthand option", async () => {
    const code = `
<script>
let value = 1;
</script>

<div {value}>hello</div>
`.trim();

    const result = await format("App.svelte", code, {
      svelte: { allowShorthand: false },
    });
    expect(result.code).toMatchInlineSnapshot(`
      "<script>
        let value = 1;
      </script>

      <div value={value}>hello</div>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });

  it("should work with Tailwind class sorting in markup", async () => {
    const code = `<div class="py-2 px-4 flex mt-4 items-center">hello</div>`;

    const result = await format("App.svelte", code, {
      sortTailwindcss: {},
    });
    expect(result.code).toMatchInlineSnapshot(`
      "<div class="mt-4 flex items-center px-4 py-2">hello</div>
      "
    `);
    expect(result.errors).toStrictEqual([]);
  });
});
