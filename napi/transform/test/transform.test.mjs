import { assert, describe, it } from 'vitest';

import oxc from './index.js';

describe('transform', () => {
  const code = 'class A<T> {}';

  it('matches output', () => {
    const ret = oxc.transform('test.ts', code, { sourcemap: true });
    assert.deepEqual(ret, {
      code: 'class A {}\n',
      errors: [],
      map: {
        mappings: 'AAAA,MAAM,EAAK,CAAE',
        names: [],
        sources: ['test.ts'],
        sourcesContent: ['class A<T> {}'],
        version: 3,
      },
    });
  });

  it('lang', () => {
    const ret = oxc.transform('test.vue', code, { lang: 'ts' });
    assert.equal(ret.code, 'class A {}\n');
  });
});

describe('react refresh plugin', () => {
  const code = `import { useState } from "react";
  export const App = () => {
    const [count, setCount] = useState(0);
    return <button onClick={() => setCount(count + 1)}>count is {count}</button>;
  };`;

  it('matches output', () => {
    const ret = oxc.transform('test.tsx', code, { jsx: { refresh: {} } });
    assert.equal(
      ret.code,
      'var _s = $RefreshSig$();\n' +
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
    );
  });
});

describe('define plugin', () => {
  const code = 'if (process.env.NODE_ENV === "production") { foo; }';

  it('matches output', () => {
    const ret = oxc.transform('test.tsx', code, {
      define: {
        'process.env.NODE_ENV': '"development"',
      },
    });
    assert.equal(ret.code, '');
  });
});

describe('inject plugin', () => {
  const code = 'let _ = Object.assign';

  it('matches output', () => {
    const ret = oxc.transform('test.tsx', code, {
      inject: {
        'Object.assign': 'foo',
      },
    });
    assert.equal(ret.code, 'import $inject_Object_assign from "foo";\nlet _ = $inject_Object_assign;\n');
  });
});
