import { expect, test } from "vitest";

// Note this points directly to `dist/wasm.js` rather than using `#oxc-parser` because
// browser tests should exercise the published browser entry point.
import { parse, parseSync } from "../dist/wasm.js";

test("parseSync", () => {
  const result = parseSync("test.mjs", "ok");
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

test("parse", async () => {
  const result = await parse("test.mjs", "ok");
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
