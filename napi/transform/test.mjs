import assert from 'assert';
import oxc from './index.js';

console.log(`Testing on ${process.platform}-${process.arch}`);

function test(ret, expected) {
  console.log(ret.code);
  console.log(ret.map);
  for (const error of ret.errors) {
    console.log(error);
  }
  assert.equal(ret.code, expected.code);
  assert.deepEqual(ret.map, expected.map);
  assert(ret.errors.length == 0);
}

test(oxc.isolatedDeclaration('test.ts', 'class A {}', { sourcemap: true }), {
  code: 'declare class A {}\n',
  map: {
    mappings: 'AAAA,cAAM,EAAE,CAAE',
    names: [],
    sources: ['test.ts'],
    sourcesContent: ['class A {}'],
    version: 3,
  },
});

test(oxc.transform('test.ts', 'class A<T> {}', { sourcemap: true }), {
  code: 'class A {}\n',
  map: {
    mappings: 'AAAA,MAAM,EAAK,CAAE',
    names: [],
    sources: ['test.ts'],
    sourcesContent: ['class A<T> {}'],
    version: 3,
  },
});

// Test react refresh plugin
test(
  oxc.transform(
    'test.tsx',
    `
  import { useState } from "react";
  export const App = () => {
    const [count, setCount] = useState(0);
    return <button onClick={() => setCount(count + 1)}>count is {count}</button>;
  };
`,
    { react: { refresh: {} } },
  ),
  {
    code: 'var _s = $RefreshSig$();\n' +
      'import { useState } from "react";\n' +
      'import { jsxs as _jsxs } from "react/jsx-runtime";\n' +
      'export const App = () => {\n' +
      '\t_s();\n' +
      '\tconst [count, setCount] = useState(0);\n' +
      '\treturn _jsxs("button", {\n' +
      '\t\tonClick: () => setCount(count + 1),\n' +
      '\t\tchildren: ["count is ", count]\n' +
      '\t});\n' +
      '};\n' +
      '_s(App, "oDgYfYHkD9Wkv4hrAPCkI/ev3YU=");\n' +
      '_c = App;\n' +
      'var _c;\n' +
      '$RefreshReg$(_c, "App");\n',
  },
);
