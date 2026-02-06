import { describe, expect, it } from "vitest";
import { format } from "../../dist/index.js";

describe("Format js-in-mdx with prettier-plugin-oxfmt", () => {
  it("should format code block in .mdx", async () => {
    const input = `
# MDX

\`\`\`js
const greet = (name) => {
return \`Hello, \${name}!\`;
}
console.log(greet('World')    )
\`\`\`

Some more text here.

\`\`\`typescript
interface Person { name: string;
age: number;
}

console.log(person);
\`\`\`

`;
    const result = await format("a.mdx", input);

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should format import/export in .mdx", async () => {
    const input = `
import React from 'react';
  import {useState} from "react"

- foo
  - bar

export const MyComponent = () => (
  <div><h1>Hello, MDX!</h1>
<p>This is a sample MDX file.</p>
  </div>
);
`;
    const result = await format("a.mdx", input);

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should work oxfmt-ignore in import/export", async () => {
    const input = `
import React from 'react';
// oxfmt-ignore
  import {useState} from "react"

- foo
  - bar

export const MyComponent = () => (
  <div><h1>Hello, MDX!</h1>
{/* oxfmt-ignore */}
<p>This is a sample MDX file.</p>
  </div>
);
`;
    const result = await format("a.mdx", input);

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });

  it("should be disabled by prettier-ignore", async () => {
    const input = `
# Mdx

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
    const result = await format("a.mdx", input);

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
    const result = await format("a.mdx", input, {
      embeddedLanguageFormatting: "off",
    });

    expect(result.code).toMatchSnapshot();
    expect(result.errors).toStrictEqual([]);
  });
});
