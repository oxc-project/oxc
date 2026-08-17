import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

import { transformSync } from "../index.js";

test("compiles TSX while preserving JSX", () => {
  const result = transformSync(
    "Component.tsx",
    fs.readFileSync(new URL("fixtures/component.tsx", import.meta.url), "utf8"),
  );

  assert.equal(result.fatal, false, JSON.stringify(result.errors));
  assert.match(result.code, /react\/compiler-runtime/);
  assert.match(result.code, /_c\(/);
  assert.match(result.code, /<div>/);
  assert.doesNotMatch(result.code, /props: \{/);
});

test("accepts JSX input", () => {
  const result = transformSync(
    "Component.jsx",
    fs.readFileSync(new URL("fixtures/component.jsx", import.meta.url), "utf8"),
  );

  assert.equal(result.fatal, false, JSON.stringify(result.errors));
  assert.match(result.code, /<span>/);
});
