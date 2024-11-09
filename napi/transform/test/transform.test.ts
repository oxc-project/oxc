import { assert, describe, it } from 'vitest';

import oxc from './index';

describe('simple', () => {
  const code = 'export class A<T> {}';

  it('matches output', () => {
    const ret = oxc.transform('test.ts', code, { sourcemap: true });
    assert.deepEqual(ret, {
      code: 'export class A {}\n',
      errors: [],
      map: {
        mappings: 'AAAA,OAAO,MAAM,EAAK,CAAE',
        names: [],
        sources: ['test.ts'],
        sourcesContent: ['export class A<T> {}'],
        version: 3,
      },
    });
  });

  it('uses the `lang` option', () => {
    const ret = oxc.transform('test.vue', code, { lang: 'ts' });
    assert.equal(ret.code, 'export class A {}\n');
  });

  it('uses the `declaration option`', () => {
    const ret = oxc.transform('test.ts', code, { typescript: { declaration: true } });
    assert.equal(ret.declaration, 'export declare class A<T> {}\n');
  });
});

describe('transform', () => {
  it('should not transform by default', () => {
    const cases = [
      '() => {};',
      'a ** b;',
      'async function foo() {}',
      '({ ...x });',
      'try {} catch {}',
      'a ?? b;',
      'a ||= b;',
      'class foo {\n\tstatic {}\n}',
    ];
    for (const code of cases) {
      const ret = oxc.transform('test.ts', code);
      assert.equal(ret.code.trim(), code);
    }
  });
});

describe('modules', () => {
  it('should transform export = and import ', () => {
    const code = `
export = function foo (): void {}
import bar = require('bar')
`;
    const ret = oxc.transform('test.ts', code, {
      typescript: {
        declaration: true,
      },
    });
    assert.deepEqual(ret, {
      code: 'module.exports = function foo() {};\nconst bar = require("bar");\n',
      declaration: 'declare const _default: () => void;\nexport = _default;\n',
      errors: [],
    });
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
  it('matches output', () => {
    const code = 'if (process.env.NODE_ENV === "production") { foo; }';
    const ret = oxc.transform('test.tsx', code, {
      define: {
        'process.env.NODE_ENV': '"development"',
      },
    });
    assert.equal(ret.code, '');
  });

  it('handles typescript declare global', () => {
    const code = 'declare let __TEST_DEFINE__: string; console.log({ __TEST_DEFINE__ });';
    const ret = oxc.transform('test.ts', code, {
      define: {
        '__TEST_DEFINE__': '"replaced"',
      },
    });
    assert.equal(ret.code, 'console.log({ __TEST_DEFINE__: "replaced" });\n');
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
