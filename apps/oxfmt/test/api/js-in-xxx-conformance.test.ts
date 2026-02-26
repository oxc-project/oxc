import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, relative } from "node:path";
import type { Plugin } from "prettier";
import prettier from "prettier";
import * as astroPlugin from "prettier-plugin-astro";
import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

// NOTE: Fixtures can be downloaded by `pnpm download-prettier-fixtures`
const FIXTURES_DIR = join(import.meta.dirname, "../../prettier-fixtures");

describe("Prettier conformance for .vue files", () => {
  const vueFixtures = collectFixtures(".vue", [
    "vue/range/example.vue",
    "vue/multiparser/lang-tsx.vue",
  ]);
  vueFixtures.push(
    {
      name: "edge/vue-bindings-multiline-template-literal.vue",
      content: `<template>
  <Comp
    #default="{ a = \`line
\${foo}\` }"
  >{{ a }}</Comp>
</template>
`,
    },
    {
      name: "edge/vue-for-multiline-template-literal.vue",
      content: `<template>
  <div v-for="(item = \`line
\${foo}\`, index) in items">{{ item }} - {{ index }}</div>
</template>
`,
    },
    {
      name: "edge/vue-script-generic-template-literal-type.vue",
      content: `<script setup lang="ts" generic="T extends \`\${string}-\${number}\`, U">
const value = 1
</script>
`,
    },
  );

  describe.concurrent.each(vueFixtures)("$name", ({ name, content }) => {
    it.each([
      { printWidth: 80 },
      {
        printWidth: 100,
        vueIndentScriptAndStyle: true,
        singleQuote: true,
      },
    ])("%j", async (options) => {
      const [oxfmtRes, prettierRes] = await compareWithPrettier(name, content, "vue", options);
      expect(oxfmtRes).toBe(prettierRes);
    });
  });
});

describe("Prettier conformance for .astro files", () => {
  const astroFixtures = collectFixtures(".astro", []);
  astroFixtures.push(
    {
      name: "edge/astro-frontmatter-and-script.astro",
      content: `---
import { foo } from "bar";
const x = 1;
---
<html>
  <body>
    <h1>{x}</h1>
    <script>
      const y = 2;
    </script>
  </body>
</html>
`,
    },
    {
      name: "edge/astro-typescript-frontmatter.astro",
      content: `---
interface Props {
  title: string;
  count: number;
}

const { title, count } = Astro.props;
---
<h1>{title} ({count})</h1>
`,
    },
    {
      name: "edge/astro-no-frontmatter.astro",
      content: `<div>
  <p>Hello</p>
</div>
`,
    },
    {
      name: "edge/astro-empty-frontmatter.astro",
      content: `---
---
<div>hello</div>
`,
    },
    {
      name: "edge/astro-client-directives.astro",
      content: `---
import Counter from "../components/Counter";
import Toggle from "../components/Toggle";
---
<Counter client:load />
<Toggle client:visible />
`,
    },
  );

  describe.concurrent.each(astroFixtures)("$name", ({ name, content }) => {
    it.each([
      { printWidth: 80 },
      { printWidth: 100, singleQuote: true },
    ])("%j", async (options) => {
      const [oxfmtRes, prettierRes] = await compareWithPrettier(
        name,
        content,
        "astro",
        options,
        [astroPlugin as Plugin],
      );
      expect(oxfmtRes).toBe(prettierRes);
    });
  });
});

// ---

type TestCase = { name: string; content: string };

function collectFixtures(ext: string, excludes: string[] = []): TestCase[] {
  const dir = FIXTURES_DIR;
  // NOTE: In CI, the fixtures might not be present, just skip and only run edge cases.
  if (!existsSync(dir)) return [];

  const results: TestCase[] = [];
  for (const entry of readdirSync(dir, { withFileTypes: true, recursive: true })) {
    if (!entry.isFile() || !entry.name.endsWith(ext)) continue;

    const fullPath = join(entry.parentPath, entry.name);
    const relPath = relative(dir, fullPath);
    if (excludes.some((s) => relPath.includes(s))) continue;

    results.push({ name: relPath, content: readFileSync(fullPath, "utf8") });
  }

  return results.sort((a, b) => a.name.localeCompare(b.name));
}

async function compareWithPrettier(
  fileName: string,
  content: string,
  parser: string,
  options: Record<string, unknown> = {},
  plugins: Plugin[] = [],
) {
  let prettierResult;
  try {
    prettierResult = await prettier.format(content, {
      parser,
      filepath: fileName,
      plugins,
      ...options,
    });
  } catch {
    prettierResult = "ERROR";
  }

  let oxfmtResult;
  const res = await format(fileName, content, options);
  if (res.errors.length !== 0) {
    oxfmtResult = "ERROR";
  } else {
    oxfmtResult = res.code;
  }

  return [oxfmtResult, prettierResult];
}
