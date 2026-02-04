import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Format js-in-xxx with prettier-plugin-oxfmt", () => {
  it("should oxfmt-ignore work", async () => {
    const input = `
<!doctype html>
<html><head>
    <script>
      // oxfmt-ignore
      const    a=  1;

window
.foo(   a  )
    </script>
  </head>
  <body>
</body>
</html>
`;
    const result = await format("a.html", input);

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

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
      experimentalTailwindcss: { functions: ["clsx"] },
    });

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format .md w/o duplicating TS lines", async () => {
    // https://prettier.io/playground/#N4Igxg9gdgLgprEAuEADdMDOAdKBDAdzwEsYACABQCcIBbYzOAOirkwgBsA3OACjLLYQXPBwCucIQICUAblyES5YLgEwAFgl7SyKqAIC+AGlwH5UAPQWyAFXUMykWrQTk8UACZkARnA4QCMgJiDg4fODIPMQAHDmIwPHgPBSJSMgBzOBhqOgY+aVx0VBAjEAhomGJoTGRQPCoaAgp6hBqUUSIATxrS7yo8MABrLIBlPBcAGWIoOGQAM1FGUohvACs4MBgAdX7o5BBo1kYqHhKQPoHhmBHogen05BgqCVLGekfnuFK4AA9ouCoxBcsFEAHl-v0YBAqBQIJhSFUoPsEB4zr8IUDXKIbAD8IC2PNFl8QPCoOkOHAAIpiCDwQkcJYgVaYH4je4U6m02ZIBYM4kARxp8By0TaIDwmAAtDM4B5ZWcniQ4mSAMJ0Wh4faiDhnUnkuAAQRgT2I3jEwoBUxm9MZ6hgtA4W3s8EwtzAcBGrQRXFInX2YEwPWEEgAkp5XCMwICKgbPCMYJ0KTbiYc4XAdng9ihDmwAadvp5QXMrdyQBw5mdpsdsv10hrk6VblRjvsNVRBh4AkjG4DYFtiB4NMgABwABlKrEFxFY1Dwdc1PKJpRgeG8-cH6mQACZSmJGDZV21eYy4LRfB45R4Ju50mI53AAGLQjXG+5a80QEAGAxAA
    const input = `
# Markdown

In Prettier, using \`typescript\` parser inside \`markdown\` parser duplicates lines after \`await { then }\`.

\`\`\`ts
await Promise.resolve(  "value"  );
await {
  then() {
  },
};
// This comment and below will be duplicated
await getPromise()
\`\`\`

But we are not!
`;
    const result = await format("a.md", input);

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });
});

describe("Disable js-in-xxx formatting", () => {
  it("should be disabled by prettier-ignore", async () => {
    const input = `
# Markdown

## Disabled

<!-- prettier-ignore -->
\`\`\`ts
const x=1;
const y=2;
console.log(x+y);
\`\`\`

## Enabled

\`\`\`jsx
const X = ()=>
 <><div>JSX</div><hr/></>
\`\`\`
`;
    const result = await format("a.md", input);

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should be disabled by `embeddedLanguageFormatting: 'off'`", async () => {
    const input = `

\`\`\`js
const x=1;
const y=2;
console.log(x+y);
\`\`\`
`;
    const result = await format("a.md", input, {
      embeddedLanguageFormatting: "off",
    });

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });
});
