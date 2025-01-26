import { describe, expect, it, test } from 'vitest';

import { HelperMode, transform } from '../index';

describe('simple', () => {
  const code = 'export class A<T> {}';

  it('matches output', () => {
    const ret = transform('test.ts', code, { sourcemap: true });
    expect(ret).toStrictEqual({
      code: 'export class A {}\n',
      errors: [],
      helpersUsed: {},
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
    const ret = transform('test.vue', code, { lang: 'ts' });
    expect(ret.code).toEqual('export class A {}\n');
  });

  it('uses the `declaration` option', () => {
    const ret = transform('test.ts', code, { typescript: { declaration: {} } });
    expect(ret.declaration).toEqual('export declare class A<T> {}\n');
  });

  it('uses the `sourcemap` option', () => {
    const ret = transform('test.ts', code, { typescript: { declaration: {} }, sourcemap: true });
    expect(ret.declarationMap).toStrictEqual(
      {
        'mappings': 'AAAA,OAAO,cAAM,EAAE,GAAG,CAAE',
        'names': [],
        'sources': [
          'test.ts',
        ],
        'sourcesContent': [
          'export class A<T> {}',
        ],
        'version': 3,
      },
    );
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
      const ret = transform('test.ts', code);
      expect(ret.code.trim()).toEqual(code);
    }
  });
});

describe('target', () => {
  const data = [
    ['es5', '() => {};\n'],
    ['es6', 'a ** b;\n'],
    ['es2015', 'a ** b;\n'],
    ['es2016', 'async function foo() {}\n'],
    ['es2017', '({ ...x });\n'],
    ['es2017', 'try {} catch {}\n'],
    ['es2019', 'a?.b;\n'],
    ['es2019', 'a ?? b;\n'],
    ['es2021', 'class foo {\n\tstatic {}\n}\n'],
  ];

  test.each(data)('transform %s', (target, code) => {
    // Also test array syntax.
    const ret = transform('test.js', code, { target: [target] });
    expect(ret.errors.length).toBe(0);
    expect(ret.code).toBeDefined();
    expect(ret.code).not.toEqual(code);
  });

  test.each(data)('no transform esnext: %s', (_target, code) => {
    const ret = transform('test.js', code, { target: 'esnext' });
    expect(ret.errors.length).toBe(0);
    expect(ret.code).toBeDefined();
    expect(ret.code).toEqual(code);
  });

  it('should turn off class propertiers because plugin is not ready', () => {
    const code = 'class Foo {\n\t#a;\n}\n';
    const ret = transform('test.js', code, { target: 'es2015' });
    expect(ret.errors.length).toBe(0);
    expect(ret.code).toBeDefined();
    expect(ret.code).toMatchInlineSnapshot(`
      "import _classPrivateFieldInitSpec from "@babel/runtime/helpers/classPrivateFieldInitSpec";
      var _a = new WeakMap();
      class Foo {
      	constructor() {
      		_classPrivateFieldInitSpec(this, _a, void 0);
      	}
      }
      "
    `);
  });
});

describe('helpers', () => {
  const data: Array<[HelperMode, string]> = [
    [HelperMode.External, 'babelHelpers.objectSpread2({}, x);\n'],
    [HelperMode.Runtime, 'import _objectSpread from "@babel/runtime/helpers/objectSpread2";\n_objectSpread({}, x);\n'],
  ];

  test.each(data)('%s', (mode, expected) => {
    const code = `({ ...x })`;
    const ret = transform('test.js', code, {
      target: 'es2015',
      helpers: { mode },
    });
    expect(ret.code).toEqual(expected);
    expect(ret.helpersUsed).toStrictEqual({
      objectSpread2: '@babel/runtime/helpers/objectSpread2',
    });
  });
});

describe('modules', () => {
  it('should transform export = and import ', () => {
    const code = `
export = function foo (): void {}
import bar = require('bar')
console.log(bar)
`;
    const ret = transform('test.ts', code, {
      typescript: {
        declaration: {},
      },
    });
    expect(ret.code).toMatchInlineSnapshot(`
      "module.exports = function foo() {};
      const bar = require("bar");
      console.log(bar);
      "
    `);
    expect(ret.declaration).toEqual('declare const _default: () => void;\nexport = _default;\n');
  });
});

describe('jsx', () => {
  const code = `const foo: Foo = <div/>`;

  it('enables jsx transform by default', () => {
    const ret = transform('test.tsx', code);
    expect(ret.code).toEqual('import { jsx as _jsx } from "react/jsx-runtime";\nconst foo = _jsx("div", {});\n');
  });

  it('configures jsx', () => {
    const ret = transform('test.tsx', code, {
      jsx: {
        importSource: 'xxx',
      },
    });
    expect(ret.code).toEqual('import { jsx as _jsx } from "xxx/jsx-runtime";\nconst foo = _jsx("div", {});\n');
  });

  it('can preserve jsx transform', () => {
    const ret = transform('test.tsx', code, {
      jsx: 'preserve',
    });
    expect(ret.code).toEqual('const foo = <div />;\n');
  });
});

describe('react refresh plugin', () => {
  const code = `import { useState } from "react";
  export const App = () => {
    const [count, setCount] = useState(0);
    return <button onClick={() => setCount(count + 1)}>count is {count}</button>;
  };`;

  it('matches output', () => {
    const ret = transform('test.tsx', code, { jsx: { refresh: {} } });
    expect(ret.code).toEqual(
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
    const ret = transform('test.tsx', code, {
      define: {
        'process.env.NODE_ENV': '"development"',
      },
    });
    expect(ret.code).toEqual('');
  });

  it('handles typescript declare global', () => {
    const code = 'declare let __TEST_DEFINE__: string; console.log({ __TEST_DEFINE__ });';
    const ret = transform('test.ts', code, {
      define: {
        '__TEST_DEFINE__': '"replaced"',
      },
    });
    expect(ret.code).toEqual('console.log({ __TEST_DEFINE__: "replaced" });\n');
  });
});

describe('inject plugin', () => {
  const code = 'let _ = Object.assign';

  it('matches output', () => {
    const ret = transform('test.tsx', code, {
      inject: {
        'Object.assign': 'foo',
      },
    });
    expect(ret.code).toEqual('import $inject_Object_assign from "foo";\nlet _ = $inject_Object_assign;\n');
  });
});
