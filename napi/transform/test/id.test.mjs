import { assert, describe, it } from 'vitest';

import oxc from './index.js';

describe('isolated declaration', () => {
  const code = `
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

  it('matches output', () => {
    const ret = oxc.isolatedDeclaration('test.ts', code, { sourcemap: true });
    assert.deepEqual(ret, {
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
        mappings: ';;;AAIE,OAAO,cAAM,EAAE;;;;CAIb;AACD',
        names: [],
        sources: ['test.ts'],
        sourcesContent: [code],
        version: 3,
      },
      errors: [],
    });
  });
});
