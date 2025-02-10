import { describe, expect, test } from 'vitest';

import { parseSync } from '../index.js';

describe('esm', () => {
  // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import#syntax
  // https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/export#syntax
  let code = `
import defaultExport from "module-name";
import * as name from "module-name";
import { export1 } from "module-name";
import { export1 as alias1 } from "module-name";
import { default as alias } from "module-name";
import { export1, export2 } from "module-name";
import { export1, export2 as alias2, /* … */ } from "module-name";
import { "string name" as alias } from "module-name";
import defaultExport, { export1, /* … */ } from "module-name";
import defaultExport, * as name from "module-name";
import "module-name";

export let name1, name2/*, … */; // also var
export const name1 = 1, name2 = 2/*, … */; // also var, let
export function functionName() { /* … */ }
export class ClassName { /* … */ }
export function* generatorFunctionName() { /* … */ }
export const { name1, name2: bar } = o;
export const [ name1, name2 ] = array;

export { name1, /* …, */ nameN };
export { variable1 as name1, variable2 as name2, /* …, */ nameN };
export { variable1 as "string name" };
export { name1 as default /*, … */ };

export default expression;
export default function functionName() { /* … */ }
export default class ClassName { /* … */ }
export default function* generatorFunctionName() { /* … */ }
export default function () { /* … */ }
export default class { /* … */ }
export default function* () { /* … */ }

export * from "module-name";
export * as name1 from "module-name";
export { name1, /* …, */ nameN } from "module-name";
export { import1 as name1, import2 as name2, /* …, */ nameN } from "module-name";
export { default, /* …, */ } from "module-name";
export { default as name1 } from "module-name";
`.split('\n').map((s) => s.trim()).filter(Boolean);

  test.each(code)('%s', (s) => {
    const ret = parseSync('test.js', s);
    expect(ret.program.body.length).toBeGreaterThan(0);
    expect(ret.errors.length).toBe(0);
    expect(JSON.stringify(ret.module, null, 2)).toMatchSnapshot();
    expect(ret.module.hasModuleSyntax).toBe(true);
    if (s.startsWith('import')) {
      expect(ret.module.staticImports.length).toBe(1);
      expect(ret.module.staticExports.length).toBe(0);
    }
    if (s.startsWith('export')) {
      expect(ret.module.staticImports.length).toBe(0);
      expect(ret.module.staticExports.length).toBe(1);
    }
  });
});

describe('hasModuleSyntax', () => {
  test('import.meta', () => {
    const ret = parseSync('test.js', 'import.meta.foo');
    expect(ret.module.hasModuleSyntax).toBe(true);
    expect(ret.module.importMetas).toEqual([{ start: 0, end: 11 }]);
  });

  test('import expression', () => {
    const ret = parseSync('test.js', "import('foo')");
    expect(ret.module.hasModuleSyntax).toBe(false);
    expect(ret.module.dynamicImports).toStrictEqual([{ start: 0, end: 13, moduleRequest: { start: 7, end: 12 } }]);
  });

  test('script', () => {
    const ret = parseSync('test.js', "require('foo')");
    expect(ret.module.hasModuleSyntax).toBe(false);
  });
});
