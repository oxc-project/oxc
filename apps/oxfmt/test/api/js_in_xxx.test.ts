import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

const mdSource = `\
Header
======

_Look,_ code blocks are formatted *too!*

\`\`\` js
function identity(x) { return x }
\`\`\`

\`\`\`jsx
<Button>Click</Button>
\`\`\`

\`\`\`ts
const a:string ="x";
\`\`\`

\`\`\`\`tsx
const X=()=>(
<>
  <P> ... </P>
</>)
\`\`\`\`

Pilot|Airport|Hours
--|:--:|--:
John Doe|SKG|1338
Jane Roe|JFK|314

- - - - - - - - - - - - - - -

+ List
 + with a [link] (/to/somewhere)
+ and [another one]


  [another one]:  http://example.com 'Example title'

Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Curabitur consectetur maximus risus, sed maximus tellus tincidunt et.
`;

const mdxSource = `\
import     {     Baz } from     './Fixture'
import { Buz  }   from './Fixture'

export  const   foo    = {
  hi:     \`Fudge \${Baz.displayName || 'Baz'}\`,
  authors: [
     'fred',
           'sally'
    ]
}

# Hello,    world!


 I'm an awesome   paragraph.

<!-- I'm a comment -->

<Foo bg='red'>
      <Bar    >hi    </Bar>
       {  hello       }
       {     /* another comment */}
</Foo>

\`\`\`
test codeblock
\`\`\`

\`\`\`js
module.exports =         'test'
\`\`\`

\`\`\`tsx
<>
  <div>
    Hello,   world!
  </div>
</>
\`\`\`

\`\`\`sh
npm i -g foo
\`\`\`

| Test  | Table   |
|    :---     | :----  |
|   Col1  | Col2    |

export   default     ({children   }) => < div>{    children}</div>
`;

const vueSource = `\
<script setup lang="ts">
import z from "z";
  import a from "a";
    import m from "m";

const count = ref(0);
</script>
<template>
  <div class="p-4 flex m-0">{{ count }}</div>
  <button class="m-2 p-1 flex">Click</button>
  <p>Templates are formatted as well...
    </p>
</template>

<style>
.and { css: too !important }
</style>
`;

const multiScriptVueSource = `\
<script lang="ts">
import z from "z";
import a from "a";

export default { name: "App" };
</script>
<script setup lang="ts">
import m from "m";
import b from "b";

const count = ref(0);
</script>
<template>
  <div>{{ count }}</div>
</template>
`;

const noImportsVueSource = `\
<!-- https://github.com/oxc-project/oxc/issues/20428 -->
<script lang="ts">
let foo = 'foo';

let bar = 'bar';
</script>
`;

const files: [string, string][] = [
  ["test.md", mdSource],
  ["test.mdx", mdxSource],
  ["app.vue", vueSource],
  ["multi-script.vue", multiScriptVueSource],
  ["no-imports.vue", noImportsVueSource],
];

describe("js-in-xxx", () => {
  it.each(files)("should format %s with full-config", async (filename, source) => {
    const result = await format(filename, source, {
      vueIndentScriptAndStyle: true,
      sortImports: {},
      sortTailwindcss: {},
    });
    expect(result.errors).toStrictEqual([]);
    expect(result.code).toMatchSnapshot();
  });
});
