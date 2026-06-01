import { describe, expect, it, vi } from "vitest";
import { format } from "../../dist/index.js";

describe("Svelte support", () => {
  describe("Basic", () => {
    it("should format `.svelte` with `svelte: {}` (defaults)", async () => {
      const input = `<script>let   count=$state(0);</script>
<button onclick={()=>count++}>clicks: {count}</button>
<style>button{color:red;}</style>
`;
      const result = await format("App.svelte", input, { svelte: {} });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });

    it("should format `.svelte` with `svelte: true` (defaults, equivalent to `{}`)", async () => {
      const input = `<script>let   count=$state(0);</script>
<button onclick={()=>count++}>clicks: {count}</button>
<style>button{color:red;}</style>
`;
      const trueResult = await format("App.svelte", input, { svelte: true });
      const objectResult = await format("App.svelte", input, { svelte: {} });
      expect(trueResult.errors).toStrictEqual([]);
      expect(objectResult.errors).toStrictEqual([]);
      // `svelte: true` should produce the same output as `svelte: {}`
      expect(trueResult.code).toBe(objectResult.code);
    });

    it("should respect `svelte.sortOrder`", async () => {
      const input = `<style>button{color:red;}</style>
<script>let count=$state(0);</script>
<button onclick={()=>count++}>{count}</button>
`;
      const defaultOrder = await format("App.svelte", input, { svelte: {} });
      const customOrder = await format("App.svelte", input, {
        svelte: { sortOrder: "styles-options-markup-scripts" },
      });
      expect(defaultOrder.errors).toStrictEqual([]);
      expect(customOrder.errors).toStrictEqual([]);
      // Order should differ between the two configurations
      expect(defaultOrder.code).not.toBe(customOrder.code);
      expect(customOrder.code).toMatchSnapshot();
    });

    it("should accept all `svelte.*` options together", async () => {
      const input = `<script>
let value = $state("x");
</script>
<input bind:value={value} />
<style>
input { color: red; }
</style>
`;
      const result = await format("App.svelte", input, {
        svelte: {
          sortOrder: "styles-options-markup-scripts",
          allowShorthand: false,
          indentScriptAndStyle: false,
        },
      });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });
  });

  describe("Gating", () => {
    it("should error on `.svelte` without `svelte` option", async () => {
      const input = `<script>let count = $state(0);</script>
<p>{count}</p>
`;
      const result = await format("App.svelte", input);
      expect(result.code).toBe(input); // unchanged
      expect(result.errors.length).toBe(1);
      expect(result.errors[0].message).toMatch(/Cannot format `\.svelte`/);
    });

    it("should error on `.svelte` with `svelte: false` (explicitly disabled)", async () => {
      const input = `<script>let count = $state(0);</script>
<p>{count}</p>
`;
      const result = await format("App.svelte", input, { svelte: false });
      expect(result.code).toBe(input); // unchanged
      expect(result.errors.length).toBe(1);
      expect(result.errors[0].message).toMatch(/Cannot format `\.svelte`/);
    });
  });

  describe("Script section (JS)", () => {
    it("should respect `oxfmt-ignore` inside `<script>`", async () => {
      // Note: `oxfmt-ignore` inside <script> applies to the next statement.
      const input = `<script>
// oxfmt-ignore
const x   =   { a:1,b:2 };
const y={c:3,d:4};
</script>
<p>{x.a + y.c}</p>
`;
      const result = await format("App.svelte", input, { svelte: {} });
      expect(result.errors).toStrictEqual([]);
      // Ignored line keeps the messy spacing; the next line is normalized.
      expect(result.code).toContain("const x   =   { a:1,b:2 };");
      expect(result.code).toContain("const y = { c: 3, d: 4 };");
      expect(result.code).toMatchSnapshot();
    });

    // `prettier-plugin-svelte` does not handle this...
    // https://github.com/sveltejs/prettier-plugin-svelte/issues/456
    it("should currently error with `embeddedLanguageFormatting: 'off'` (known limitation)", async () => {
      // The plugin dumps the offending AST node via `console.error` before throwing
      // https://github.com/sveltejs/prettier-plugin-svelte/blob/c4b7844961d0b937ab1eab2c8323d40a9422ac30/src/print/index.ts#L725
      const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      try {
        const input = `<script>
const x={a:1,b:2};
</script>
<p>{x.a}</p>
`;
        const result = await format("App.svelte", input, {
          svelte: {},
          embeddedLanguageFormatting: "off",
        });
        expect(result.errors.length).toBeGreaterThan(0);
        expect(result.errors[0].message).toMatch(/unknown node type: Script/);
      } finally {
        errSpy.mockRestore();
      }
    });

    it('should sort imports inside `<script lang="ts">`', async () => {
      const input = `<script lang="ts">
import type { Z } from "zoo";
import { a } from "ant";
import type { A } from "ant";
let n: number = $state(0);
</script>
<p>{n}</p>
`;
      const result = await format("App.svelte", input, {
        svelte: {},
        sortImports: { order: "asc" },
      });
      expect(result.errors).toStrictEqual([]);
      // `ant` should appear before `zoo`, type annotation preserved.
      const antIdx = result.code.indexOf('from "ant"');
      const zooIdx = result.code.indexOf('from "zoo"');
      expect(antIdx).toBeGreaterThan(-1);
      expect(zooIdx).toBeGreaterThan(-1);
      expect(antIdx).toBeLessThan(zooIdx);
      expect(result.code).toContain("let n: number = $state(0);");
      expect(result.code).toMatchSnapshot();
    });
  });

  describe("Template section", () => {
    it("should sort Tailwind classes in template attributes (within `{#each}` block)", async () => {
      // Embedding the attribute inside an `{#each}` block also incidentally
      // exercises Svelte block markup formatting via `prettier-plugin-svelte`.
      const input = `<script>let items = $state([1,2,3]);</script>
{#each items as n}
<div class="p-4 flex bg-red-500 text-white">{n}</div>
{/each}
`;
      const result = await format("App.svelte", input, {
        svelte: {},
        sortTailwindcss: {},
      });
      expect(result.errors).toStrictEqual([]);
      // Display utilities (`flex`) should sort before spacing (`p-4`).
      expect(result.code).toContain('class="flex');
      expect(result.code).not.toContain('class="p-4 flex');
      expect(result.code).toMatchSnapshot();
    });
  });
});
