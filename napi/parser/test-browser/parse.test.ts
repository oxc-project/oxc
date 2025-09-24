import { expect, test } from 'vitest';
import { parseAsync, parseSync } from '../src-js/wasm.js';

test('parseSync', () => {
  const result = parseSync('test.js', 'ok');
  expect(result.program).toMatchInlineSnapshot(`
    {
      "body": [
        {
          "end": 2,
          "expression": {
            "end": 2,
            "name": "ok",
            "start": 0,
            "type": "Identifier",
          },
          "start": 0,
          "type": "ExpressionStatement",
        },
      ],
      "end": 2,
      "hashbang": null,
      "sourceType": "module",
      "start": 0,
      "type": "Program",
    }
  `);
});

test('parseAsync', async () => {
  const result = await parseAsync('test.js', 'ok');
  expect(result.program).toMatchInlineSnapshot(`
    {
      "body": [
        {
          "end": 2,
          "expression": {
            "end": 2,
            "name": "ok",
            "start": 0,
            "type": "Identifier",
          },
          "start": 0,
          "type": "ExpressionStatement",
        },
      ],
      "end": 2,
      "hashbang": null,
      "sourceType": "module",
      "start": 0,
      "type": "Program",
    }
  `);
});
