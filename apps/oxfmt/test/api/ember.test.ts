import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Ember Template Tag support", () => {
  describe("Basic", () => {
    it("should format `.gjs` with `ember: true`", async () => {
      const input = `import Component from '@glimmer/component';
export default class Foo extends Component {
<template>
      <button {{on "click" this.onClick}}>{{yield}}</button>
</template>
}
`;
      const result = await format("Foo.gjs", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });

    it("should be idempotent", async () => {
      const input = `import Component from '@glimmer/component';
export default class Foo extends Component {
<template>
      <button {{on "click" this.onClick}}>{{yield}}</button>
</template>
}
`;
      const once = await format("Foo.gjs", input, { ember: true });
      const twice = await format("Foo.gjs", once.code, { ember: true });
      expect(once.errors).toStrictEqual([]);
      expect(twice.errors).toStrictEqual([]);
      expect(twice.code).toBe(once.code);
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
      const result = await format("Bar.gts", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });
  });

  describe("Statement position", () => {
    it("should print a top-level tag as a declaration", async () => {
      // The two spellings are one declaration: a bare top-level tag already compiles to the
      // module's default export, so the keyword and the terminator both come off.
      const explicit = await format("Foo.gjs", `export default <template>x</template>;\n`, {
        ember: true,
      });
      const bare = await format("Foo.gjs", `<template>x</template>\n`, { ember: true });
      expect(explicit.errors).toStrictEqual([]);
      expect(explicit.code).toBe(`<template>x</template>\n`);
      expect(bare.code).toBe(explicit.code);
    });

    it("should leave a real `export default` alone", async () => {
      const input = `export default function f() {}\n<template>x</template>\n`;
      const result = await format("Foo.gjs", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toContain("export default function f() {}");
    });

    it("should break a group holding a Handlebars comment", async () => {
      const input = `<template><A @a={{1}} {{! note }} @b={{2}} /></template>\n`;
      const result = await format("Foo.gjs", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      // The comment annotates an attribute, so it must not share a line with them.
      expect(result.code).toContain("{{! note }}\n");
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
        ember: true,
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
      const result = await format("Foo.gjs", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toMatchSnapshot();
    });
  });

  describe("Script section (JS)", () => {
    it("should format the surrounding JS with `oxc_formatter`", async () => {
      const input = `const y={c:3,d:4};
<template>x</template>
`;
      const result = await format("Foo.gjs", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toContain("const y = { c: 3, d: 4 };");
      expect(result.code).toMatchSnapshot();
    });

    it("should apply `sortImports` around a template tag", async () => {
      const input = `import { z } from "zoo";
import { a } from "ant";
<template>x</template>
`;
      const result = await format("Foo.gjs", input, {
        ember: true,
        sortImports: { order: "asc" },
      });
      expect(result.errors).toStrictEqual([]);
      expect(result.code.indexOf('from "ant"')).toBeLessThan(result.code.indexOf('from "zoo"'));
    });

    it("should respect `oxfmt-ignore` beside a template tag", async () => {
      const input = `class A {
  // oxfmt-ignore
  x   =   { a:1,b:2 };
  y={c:3,d:4};
  <template>x</template>
}
`;
      const result = await format("Foo.gjs", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toContain("x   =   { a:1,b:2 };");
      expect(result.code).toContain("y = { c: 3, d: 4 };");
    });

    it("should leave a tag inside a string alone", async () => {
      const input = `const s   =   "<template>x</template>";
`;
      const result = await format("Foo.gjs", input, { ember: true });
      expect(result.errors).toStrictEqual([]);
      expect(result.code).toBe(`const s = "<template>x</template>";
`);
    });
  });
});
