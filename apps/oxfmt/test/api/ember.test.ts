import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Ember Template Tag support", () => {
  describe("Basic", () => {
    it("should format `.gjs` with `ember: {}` (defaults)", async () => {
      const input = `import Component from '@glimmer/component';
export default class Foo extends Component {
<template>
      <button {{on "click" this.onClick}}>{{yield}}</button>
</template>
}
`;
      const result = await format("Foo.gjs", input, { ember: {} });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });

    it("should format `.gjs` with `ember: true` (defaults, equivalent to `{}`)", async () => {
      const input = `import Component from '@glimmer/component';
export default class Foo extends Component {
<template>
      <button {{on "click" this.onClick}}>{{yield}}</button>
</template>
}
`;
      const trueResult = await format("Foo.gjs", input, { ember: true });
      const objectResult = await format("Foo.gjs", input, { ember: {} });
      expect(trueResult.errors).toStrictEqual([]);
      expect(objectResult.errors).toStrictEqual([]);
      // `ember: true` should produce the same output as `ember: {}`
      expect(trueResult.code).toBe(objectResult.code);
    });

    it("should format `.gts` including TypeScript syntax", async () => {
      const input = `import Component from '@glimmer/component';
interface Signature { Args: { count: number } }
export default class Bar extends Component<Signature> {
<template>
      <span>{{@count}}</span>
</template>
}
`;
      const result = await format("Bar.gts", input, { ember: {} });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });

    it("should respect `ember.templateExportDefault`", async () => {
      const input = `<template>hello</template>\n`;
      const off = await format("Foo.gjs", input, { ember: {} });
      const on = await format("Foo.gjs", input, { ember: { templateExportDefault: true } });
      expect(off.errors).toStrictEqual([]);
      expect(on.errors).toStrictEqual([]);
      expect(off.code).not.toContain("export default");
      expect(on.code).toContain("export default");
      expect(on.code).toMatchSnapshot();
    });

    it("should respect `ember.templateSingleQuote`", async () => {
      const input = `<template><div title="a"></div></template>\n`;
      const result = await format("Foo.gjs", input, {
        ember: { templateSingleQuote: true },
      });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toContain("title='a'");
      expect(result.code).toMatchSnapshot();
    });
  });

  describe("Gating", () => {
    it("should error on `.gjs` without `ember` option", async () => {
      const input = `<template>hello</template>\n`;
      const result = await format("Foo.gjs", input);
      expect(result.code).toBe(input); // unchanged
      expect(result.errors.length).toBe(1);
      expect(result.errors[0].message).toMatch(/Cannot format `.+\.gjs`/);
    });

    it("should error on `.gts` with `ember: false` (explicitly disabled)", async () => {
      const input = `<template>hello</template>\n`;
      const result = await format("Bar.gts", input, { ember: false });
      expect(result.code).toBe(input); // unchanged
      expect(result.errors.length).toBe(1);
      expect(result.errors[0].message).toMatch(/Cannot format `.+\.gts`/);
    });
  });

  describe("Template section", () => {
    it("should sort Tailwind classes in template attributes", async () => {
      const input = `<template>
<div class="p-4 flex bg-red-500 text-white"></div>
</template>
`;
      const result = await format("Foo.gjs", input, {
        ember: {},
        sortTailwindcss: {},
      });
      expect(result.errors).toStrictEqual([]);
      // Display utilities (`flex`) should sort before spacing (`p-4`).
      expect(result.code).toContain('class="flex');
      expect(result.code).not.toContain('class="p-4 flex');
      expect(result.code).toMatchSnapshot();
    });

    it("should respect `prettier-ignore` on a template tag", async () => {
      const input = `<template>
      {{! prettier-ignore }}
      <div    >keep</div>
</template>
`;
      const result = await format("Foo.gjs", input, { ember: {} });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });
  });

  describe("Script section (JS)", () => {
    // NOTE: The plugin parses with Prettier's own `babel-ts` and prints with Prettier's
    // `estree` printer, so `oxc_formatter` (and with it `oxfmt-ignore`) never runs here.
    it("should format surrounding JS with Prettier, not `oxc_formatter`", async () => {
      const input = `const y={c:3,d:4};
<template>x</template>
`;
      const result = await format("Foo.gjs", input, { ember: {} });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toContain("const y = { c: 3, d: 4 };");
      expect(result.code).toMatchSnapshot();
    });
  });
});
