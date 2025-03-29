import { assertType, describe, it } from 'vitest';

import type { Node, Statement } from '../index';
import { parseSync } from '../index';

describe('parse', () => {
  const code = '/* comment */ foo';

  it('checks type', async () => {
    const ret = parseSync('test.js', code);
    assertType<Statement>(ret.program.body[0]);
  });

  it('Node type', () => {
    function example(node: Node) {
      node.type satisfies string;
      switch (node.type) {
        case 'FunctionDeclaration': {
          example(node.body);
          break;
        }
        case 'BlockStatement': {
          for (const child of node.body) {
            example(child);
          }
          break;
        }
      }
    }
    const ret = parseSync('test.js', code);
    example(ret.program);
  });
});
