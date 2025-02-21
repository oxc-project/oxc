// Script to inject code for an extra `program` getter on `class ParseResult` in WASM binding files.

import assert from 'assert';
import { readFileSync, writeFileSync } from 'fs';
import { join as pathJoin } from 'path';
import { fileURLToPath } from 'url';

const pkgDirPath = pathJoin(fileURLToPath(import.meta.url), '../../../npm/parser-wasm');

const bindingFilename = 'oxc_parser_wasm.js';

// Extra getter on `ParseResult` `get program() { ... }` that gets the program as JSON string,
// and parses it to a `Program` object.
//
// JSON parsing uses a reviver function that sets `value` field of `Literal`s for `BigInt`s and `RegExp`s.
// This is not possible to do on Rust side, as neither can be represented correctly in JSON.
// Invalid regexp, or valid regexp using syntax not supported by the platform is ignored.
//
// The getter caches the result to avoid re-parsing JSON every time `result.program` is accessed.
//
// Note: This code is repeated in `napi/parser/index.js` and `crates/oxc-wasm/update-bindings.mjs`.
// Any changes should be applied in those 2 places too.
const getterCode = `
  __program;

  get program() {
    if (this.__program) return this.__program;
    return this.__program = JSON.parse(this.programJson, function(key, value) {
      if (value === null && key === 'value' && Object.hasOwn(this, 'type') && this.type === 'Literal') {
        if (Object.hasOwn(this, 'bigint')) {
          return BigInt(this.bigint);
        }
        if (Object.hasOwn(this, 'regex')) {
          const { regex } = this;
          try {
            return RegExp(regex.pattern, regex.flags);
          } catch (_err) {}
        }
      }
      return value;
    });
  }
`.trimEnd().replace(/  /g, '    ');

const insertGetterAfter = 'class ParseResult {';

for (const dirName of ['node', 'web']) {
  const path = pathJoin(pkgDirPath, dirName, bindingFilename);
  const code = readFileSync(path, 'utf8');

  const parts = code.split(insertGetterAfter);
  assert(parts.length === 2);
  const [before, after] = parts;
  const updatedCode = [before, insertGetterAfter, getterCode, after].join('');
  writeFileSync(path, updatedCode);
}
