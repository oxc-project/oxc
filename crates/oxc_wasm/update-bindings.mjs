// Script to inject code for an extra `ast` getter on `class Oxc` in WASM binding file.

import assert from 'assert';
import { readFileSync, writeFileSync } from 'fs';
import { join as pathJoin } from 'path';
import { fileURLToPath } from 'url';

const path = pathJoin(fileURLToPath(import.meta.url), '../../../npm/oxc-wasm/oxc_wasm.js');

// Extra getter on `Oxc` `get ast() { ... }` that gets the AST as JSON string,
// and parses it to a `Program` object.
//
// JSON parsing uses a reviver function that sets `value` field of `Literal`s for `BigInt`s and `RegExp`s.
// This is not possible to do on Rust side, as neither can be represented correctly in JSON.
// Invalid regexp, or valid regexp using syntax not supported by the platform is ignored.
//
// Note: This code is repeated in `napi/parser/index.js` and `wasm/parser/update-bindings.mjs`.
// Any changes should be applied in those 2 places too.
//
// Unlike `wasm/parser/update-bindings.mjs`, the getter does not cache the `JSON.parse`-ed value,
// because I (@overlookmotel) believe that the `Oxc` class instance is used as a singleton in playground,
// and the value of `astJson` may change after the source text is changed.
// TODO: Check this assumption is correct.
const getterCode = `
  get ast() {
    return JSON.parse(this.astJson, function(key, value) {
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

const insertGetterAfter = 'export class Oxc {';

const code = readFileSync(path, 'utf8');
const parts = code.split(insertGetterAfter);
assert(parts.length === 2);
const [before, after] = parts;
const updatedCode = [before, insertGetterAfter, getterCode, after].join('');
writeFileSync(path, updatedCode);
