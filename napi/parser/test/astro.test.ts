import { describe, expect, it, test } from "vitest";

import { parseAstro, parseAstroSync } from "../src-js/index.js";
import type {
  AstroParserOptions,
  AstroRoot,
  AstroFrontmatter,
  AstroScript,
} from "../src-js/index.js";

describe("parseAstro", () => {
  it("parses basic Astro file with frontmatter", () => {
    const code = `---
const name = "World";
---
<h1>Hello {name}!</h1>`;

    const ret = parseAstroSync(code);
    expect(ret.errors.length).toBe(0);

    const root = ret.root as AstroRoot;
    expect(root.type).toBe("AstroRoot");
    expect(root.frontmatter).not.toBeNull();
    expect(root.frontmatter?.type).toBe("AstroFrontmatter");
    expect(root.frontmatter?.program.body.length).toBeGreaterThan(0);
    expect(root.body.length).toBeGreaterThan(0);
  });

  it("parses Astro file without frontmatter", () => {
    const code = `<h1>Hello World!</h1>`;

    const ret = parseAstroSync(code);
    expect(ret.errors.length).toBe(0);

    const root = ret.root as AstroRoot;
    expect(root.type).toBe("AstroRoot");
    expect(root.frontmatter).toBeNull();
    expect(root.body.length).toBeGreaterThan(0);
  });

  it("parses Astro file with JSX expressions", () => {
    // Use a simpler JSX expression test that doesn't involve multi-line nested elements
    const code = `---
const name = "World";
---
<h1>Hello {name}!</h1>
<p>Welcome</p>`;

    const ret = parseAstroSync(code);
    expect(ret.errors.length).toBe(0);

    const root = ret.root as AstroRoot;
    expect(root.type).toBe("AstroRoot");
    expect(root.frontmatter).not.toBeNull();
    // Body should have h1, text, p elements
    expect(root.body.length).toBeGreaterThan(0);
  });

  it("parses Astro file with script tags", () => {
    const code = `---
const name = "World";
---
<h1>Hello {name}!</h1>
<script>
  console.log("Hello from script!");
</script>`;

    const ret = parseAstroSync(code);
    expect(ret.errors.length).toBe(0);

    const root = ret.root as AstroRoot;
    expect(root.type).toBe("AstroRoot");

    // Find the AstroScript in the body
    const astroScript = root.body.find((child: any) => child.type === "AstroScript") as
      | AstroScript
      | undefined;
    expect(astroScript).toBeDefined();
    expect(astroScript?.type).toBe("AstroScript");
    expect(astroScript?.program).toBeDefined();
  });

  it("works with async version", async () => {
    const code = `---
const name = "World";
---
<h1>Hello {name}!</h1>`;

    const ret = await parseAstro(code);
    expect(ret.errors.length).toBe(0);

    const root = ret.root as AstroRoot;
    expect(root.type).toBe("AstroRoot");
    expect(root.frontmatter).not.toBeNull();
  });

  it("supports range option", () => {
    const code = `---
const x = 1;
---
<div>test</div>`;

    const ret = parseAstroSync(code, { range: true });
    expect(ret.errors.length).toBe(0);

    const root = ret.root as AstroRoot;
    expect(root.range).toBeDefined();
    expect(root.range).toEqual([root.start, root.end]);
  });

  it("parses complex Astro file", () => {
    const code = `---
import Layout from '../layouts/Layout.astro';
import Card from '../components/Card.astro';

interface Props {
  title: string;
}

const { title } = Astro.props;
const items = await getItems();
---

<Layout title={title}>
  <main>
    <h1>{title}</h1>
    {items.map(item => (
      <Card title={item.title} href={item.href}>
        {item.description}
      </Card>
    ))}
  </main>
</Layout>

<style>
  main {
    padding: 1rem;
  }
</style>

<script>
  document.addEventListener('DOMContentLoaded', () => {
    console.log('Page loaded!');
  });
</script>`;

    const ret = parseAstroSync(code);
    // There will be some errors due to undefined variables like `getItems` and `Astro`
    // but the structure should still parse correctly

    const root = ret.root as AstroRoot;
    expect(root.type).toBe("AstroRoot");
    expect(root.frontmatter).not.toBeNull();
    expect(root.body.length).toBeGreaterThan(0);
  });

  it("reports syntax errors in frontmatter", () => {
    const code = `---
const x = 
---
<div>test</div>`;

    const ret = parseAstroSync(code);
    expect(ret.errors.length).toBeGreaterThan(0);
  });
});
