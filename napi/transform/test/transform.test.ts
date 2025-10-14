import { Worker } from 'node:worker_threads';
import { describe, expect, it, test } from 'vitest';

import { HelperMode, transform, transformAsync } from '../index';

describe('simple', () => {
  const code = 'export class A<T> {}';

  it('matches output', () => {
    const ret = transform('test.ts', code, { sourcemap: true });
    expect(ret).toMatchObject({
      code: 'export class A {}\n',
      errors: [],
      helpersUsed: {},
      map: {
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
    const ret = transform('test.ts', code, {
      typescript: { declaration: {} },
      sourcemap: true,
    });
    expect(ret.declarationMap).toMatchObject({
      names: [],
      sources: ['test.ts'],
      sourcesContent: ['export class A<T> {}'],
      version: 3,
    });
  });
});

describe('transformAsync', () => {
  const code = 'export class A<T> {}';

  it('should work asynchronously', async () => {
    const ret = await transformAsync('test.ts', code, { sourcemap: true });
    expect(ret).toMatchObject({
      code: 'export class A {}\n',
      errors: [],
      helpersUsed: {},
      map: {
        names: [],
        sources: ['test.ts'],
        sourcesContent: ['export class A<T> {}'],
        version: 3,
      },
    });
  });

  it('should produce the same result as sync transform', async () => {
    const sourceCode = `
      const add = (a, b) => a + b;
      console.log(add(1, 2));
    `;

    const syncResult = transform('test.js', sourceCode, { target: 'es2015' });
    const asyncResult = await transformAsync('test.js', sourceCode, { target: 'es2015' });

    expect(asyncResult.code).toEqual(syncResult.code);
    expect(asyncResult.errors).toEqual(syncResult.errors);
    expect(asyncResult.helpersUsed).toEqual(syncResult.helpersUsed);
  });

  it('should handle errors properly', async () => {
    const invalidCode = 'export class { invalid syntax';
    const ret = await transformAsync('test.ts', invalidCode);
    expect(ret.errors.length).toBeGreaterThan(0);
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
    ['es6', 'a ** b;\n'],
    ['es2015', 'a ** b;\n'],
    ['es2016', 'async function foo() {}\n'],
    ['es2017', '({ ...x });\n'],
    ['es2017', 'try {} catch {}\n'],
    ['es2019', 'a?.b;\n'],
    ['es2019', 'a ?? b;\n'],
    ['es2021', 'class foo {\n\tstatic {}\n}\n'],
    ['es2025', 'using handlerSync = openSync();\n'],
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
      "import _classPrivateFieldInitSpec from "@oxc-project/runtime/helpers/classPrivateFieldInitSpec";
      var _a = /* @__PURE__ */ new WeakMap();
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
    [
      HelperMode.Runtime,
      'import _objectSpread from "@oxc-project/runtime/helpers/objectSpread2";\n_objectSpread({}, x);\n',
    ],
  ];

  test.each(data)('%s', (mode, expected) => {
    const code = `({ ...x })`;
    const ret = transform('test.js', code, {
      target: 'es2015',
      helpers: { mode },
    });
    expect(ret.code).toEqual(expected);
    expect(ret.helpersUsed).toStrictEqual({
      objectSpread2: '@oxc-project/runtime/helpers/objectSpread2',
    });
  });
});

describe('modules', () => {
  it('should transform `export =` and `import =`', () => {
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
    expect(ret.declaration).toEqual(
      'declare const _default: () => void;\nexport = _default;\n',
    );
  });
});

describe('jsx', () => {
  const code = `const foo: Foo = <div/>`;

  it('enables jsx transform by default', () => {
    const ret = transform('test.tsx', code);
    expect(ret.code).toMatchInlineSnapshot(`
      "import { jsx as _jsx } from "react/jsx-runtime";
      const foo = /* @__PURE__ */ _jsx("div", {});
      "
    `);
  });

  it('configures jsx', () => {
    const ret = transform('test.tsx', code, {
      jsx: {
        importSource: 'xxx',
      },
    });
    expect(ret.code).toMatchInlineSnapshot(`
      "import { jsx as _jsx } from "xxx/jsx-runtime";
      const foo = /* @__PURE__ */ _jsx("div", {});
      "
    `);
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
    expect(ret.code).toMatchInlineSnapshot(
      `
      "import { useState } from "react";
      import { jsxs as _jsxs } from "react/jsx-runtime";
      var _s = $RefreshSig$();
      export const App = () => {
      	_s();
      	const [count, setCount] = useState(0);
      	return /* @__PURE__ */ _jsxs("button", {
      		onClick: () => setCount(count + 1),
      		children: ["count is ", count]
      	});
      };
      _s(App, "oDgYfYHkD9Wkv4hrAPCkI/ev3YU=");
      _c = App;
      var _c;
      $RefreshReg$(_c, "App");
      "
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
        __TEST_DEFINE__: '"replaced"',
      },
    });
    expect(ret.code).toEqual('console.log({ __TEST_DEFINE__: "replaced" });\n');
  });

  it('replaces undefined', () => {
    const code = 'new Foo()';
    const ret = transform('test.js', code, {
      define: {
        Foo: 'undefined',
      },
    });
    // Replaced `undefined` with `void 0` by DCE.
    expect(ret.code).toEqual('new (void 0)();\n');
  });

  it('keeps debugger', () => {
    const code = 'Foo; debugger;';
    const ret = transform('test.js', code, {
      define: {
        Foo: 'Bar',
      },
    });
    expect(ret.code).toEqual('Bar;\ndebugger;\n');
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
    expect(ret.code).toEqual(
      'import $inject_Object_assign from "foo";\nlet _ = $inject_Object_assign;\n',
    );
  });
});

describe('legacy decorator', () => {
  it('matches output', () => {
    const code = `
      export default @dce class C {
        @dce
        prop = 0;
        method(@dce param) {}
      }
    `;
    const ret = transform('test.tsx', code, {
      decorator: {
        legacy: true,
      },
    });
    expect(ret.code).toMatchInlineSnapshot(`
      "import _decorate from "@oxc-project/runtime/helpers/decorate";
      import _decorateParam from "@oxc-project/runtime/helpers/decorateParam";
      let C = class C {
      	prop = 0;
      	method(param) {}
      };
      _decorate([dce], C.prototype, "prop", void 0);
      _decorate([_decorateParam(0, dce)], C.prototype, "method", null);
      C = _decorate([dce], C);
      export default C;
      "
    `);
  });

  describe('emitDecoratorMetadata', () => {
    it('matches output', () => {
      const code = `
    export default @dce class C {
      @dce
      prop = 0;
      method(@dce param) {}
    }
  `;
      const ret = transform('test.tsx', code, {
        decorator: {
          legacy: true,
          emitDecoratorMetadata: true,
        },
      });
      expect(ret.code).toMatchInlineSnapshot(`
        "import _decorateMetadata from "@oxc-project/runtime/helpers/decorateMetadata";
        import _decorate from "@oxc-project/runtime/helpers/decorate";
        import _decorateParam from "@oxc-project/runtime/helpers/decorateParam";
        let C = class C {
        	prop = 0;
        	method(param) {}
        };
        _decorate([dce, _decorateMetadata("design:type", Object)], C.prototype, "prop", void 0);
        _decorate([
        	_decorateParam(0, dce),
        	_decorateMetadata("design:type", Function),
        	_decorateMetadata("design:paramtypes", [Object]),
        	_decorateMetadata("design:returntype", void 0)
        ], C.prototype, "method", null);
        C = _decorate([dce], C);
        export default C;
        "
      `);
    });
  });
});

describe('worker', () => {
  it('should run', async () => {
    const code = await new Promise((resolve, reject) => {
      const worker = new Worker('./test/worker.mjs');
      worker.on('error', (err) => {
        reject(err);
      });
      worker.on('exit', (code) => {
        resolve(code);
      });
    });
    expect(code).toBe(0);
  });
});

describe('typescript', () => {
  describe('options', () => {
    test('removeClassFieldsWithoutInitializer', () => {
      const code = `
        class Foo {
          a: number;
          b: number = 1;
        }
      `;
      const ret = transform('test.ts', code, {
        typescript: {
          removeClassFieldsWithoutInitializer: true,
        },
      });
      expect(ret.code).toMatchInlineSnapshot(`
        "class Foo {
        	b = 1;
        }
        "
      `);
    });

    test('align `useDefineForClassFields: false`', () => {
      const code = `
        class Foo {
          a: number;
          b: number = 1;
          @dec
          c: number;
        }
        class StaticFoo {
          static a: number;
          static b: number = 1;
          @dec
          static c: number;
        }
      `;
      const ret = transform('test.ts', code, {
        assumptions: {
          setPublicClassFields: true,
        },
        target: 'es2020',
        typescript: {
          removeClassFieldsWithoutInitializer: true,
        },
        decorator: {
          legacy: true,
        },
      });
      expect(ret.code).toMatchInlineSnapshot(`
        "import _decorate from "@oxc-project/runtime/helpers/decorate";
        class Foo {
        	constructor() {
        		this.b = 1;
        	}
        }
        _decorate([dec], Foo.prototype, "c", void 0);
        class StaticFoo {}
        StaticFoo.b = 1;
        _decorate([dec], StaticFoo, "c", void 0);
        "
      `);
    });
  });
});

describe('styled-components', () => {
  test('matches output', () => {
    const code = `
      import styled, { css } from 'styled-components';

      styled.div\`color: red;\`;
      const v = css(["color: red;"]);
    `;
    const ret = transform('test.js', code, {
      plugins: {
        styledComponents: {
          pure: true,
        },
      },
    });
    expect(ret.code).toMatchInlineSnapshot(`
			"import styled, { css } from "styled-components";
			styled.div.withConfig({
				displayName: "test",
				componentId: "sc-3q0sbi-0"
			})(["color:red;"]);
			const v = /* @__PURE__ */ css(["color: red;"]);
			"
		`);
  });
});
