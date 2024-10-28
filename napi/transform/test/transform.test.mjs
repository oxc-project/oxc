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
    console.log(ret.code);
    assert.equal(
      ret.code,
      `import { useState } from "react";
import { jsxs as _jsxs } from "react/jsx-runtime";
var _s = $RefreshSig$();
export const App = () => {
	_s();
	const [count, setCount] = useState(0);
	return _jsxs("button", {
		onClick: () => setCount(count + 1),
		children: ["count is ", count]
	});
};
_s(App, "oDgYfYHkD9Wkv4hrAPCkI/ev3YU=");
_c = App;
var _c;
$RefreshReg$(_c, "App");
`,
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
