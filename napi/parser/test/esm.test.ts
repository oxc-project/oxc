import { describe, expect, it, test } from 'vitest';

import * as oxc from '../index.js';

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
    const ret = oxc.parseSync(s, { sourceFilename: 'test.ts' });
    expect(ret.program.body.length).toBeGreaterThan(0);
    expect(ret.errors.length).toBe(0);
    expect(JSON.stringify(ret.module, null, 2)).toMatchSnapshot();
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
