import { describe, expect, it } from 'vitest';

import oxc from '../index';

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
  // Do not keep normal comments
  export class B {}
  `;

  it('matches output', () => {
    const ret = oxc.isolatedDeclaration('test.ts', code, { sourcemap: true });
    expect(ret).toStrictEqual({
      code: '/**\n' +
        '* jsdoc 1\n' +
        '*/\n' +
        'export declare class A {\n' +
        '\t/**\n' +
        '\t* jsdoc 2\n' +
        '\t*/\n' +
        '\tfoo: string;\n' +
        '}\n' +
        'export declare class B {}\n',
      map: {
        mappings: ';;;AAIE,OAAO,cAAM,EAAE;;;;CAIb;AACD;AAED,OAAO,cAAM,EAAE,CAAE',
        names: [],
        sources: ['test.ts'],
        sourcesContent: [code],
        version: 3,
      },
      errors: [],
    });
  });
});
