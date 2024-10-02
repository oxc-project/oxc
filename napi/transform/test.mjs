import assert from 'assert';
import oxc from './index.js';

console.log(`Testing on ${process.platform}-${process.arch}`);

function test(ret, expected) {
  assert.equal(ret.code, expected.code);
  assert.deepEqual(ret.map, expected.map);
  assert(ret.errors.length == 0);
}

const id = `
/**
 * jsdoc 1
 */
export class A {
  /**
   * jsdoc 2
   */
  foo = "bar";
}
`;

test(oxc.isolatedDeclaration('test.ts', id, { sourcemap: true }), {
  code: '/**\n' +
    '* jsdoc 1\n' +
    '*/\n' +
    'export declare class A {\n' +
    '\t/**\n' +
    '\t* jsdoc 2\n' +
    '\t*/\n' +
    '\tfoo: string;\n' +
    '}\n',
  map: {
    mappings: ';;;AAIA,OAAO,cAAM,EAAE;;;;CAIb;AACD',
    names: [],
    sources: ['test.ts'],
    sourcesContent: [id],
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
    { jsx: { refresh: {} } },
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

// Test define plugin
// TODO: should be constant folded
test(
  oxc.transform('test.ts', 'if (process.env.NODE_ENV === "production") { foo; }', {
    define: {
      'process.env.NODE_ENV': 'false',
    },
  }),
  {
    code: 'if (false === "production") {\n\tfoo;\n}\n',
  },
);

// Test inject plugin
test(
  oxc.transform('test.ts', 'let _ = Object.assign', {
    inject: {
      'Object.assign': 'foo',
    },
  }),
  {
    code: 'import $inject_Object_assign from "foo";\nlet _ = $inject_Object_assign;\n',
  },
);

console.log('Success.');
