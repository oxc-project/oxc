// String literal printing (port of `str.rs`, pretty mode: fixed quote).

import { CAT_OTHER, write, writeNoLast, writeWithMapNoLast } from "./write.ts";
import { debugAssert } from "../asserts.ts";

import type { State } from "../state.ts";
import type * as ESTree from "../../../../npm/oxc-types/types.d.ts";

/**
 * Characters or sequences which may need escaping or special handling.
 *
 * The quote is fixed to `"` in pretty mode, so `'` and `` ` `` are deliberately absent -
 * they are never escaped, and must not drag a string onto the slow path.
 */
const STRING_ESCAPE_REGEX =
  // eslint-disable-next-line no-control-regex
  /[\0\x07\b\v\f\n\r\x1B\\"\u2028\u2029\xA0\uD800-\uDFFF]|<\/script/i;

/**
 * Print a string literal, quotes and all.
 *
 * Pretty mode always uses double quotes, matching Oxc's default options, so there is no quote to choose.
 * Almost no string needs escaping, so the scan for characters or sequences which do is all that happens on the common path.
 */
export function printString(state: State, value: string, node: ESTree.Node): void {
  // Quote is fixed - double, matching `oxc_codegen`'s default option
  writeWithMapNoLast(state, '"', node);

  // Almost no string needs escaping, so the loop which performs escaping sits in its own function
  if (!STRING_ESCAPE_REGEX.test(value)) {
    writeNoLast(state, value);
  } else {
    printEscapedStringContents(state, value);
  }

  write(state, '"', CAT_OTHER);
}

/**
 * Print the contents of a string which contains at least one character needing an escape.
 *
 * Runs of ordinary characters are copied in chunks rather than one at a time, so the cost is in
 * the escapes rather than the length. Kept out of `printString` because it almost never runs.
 */
function printEscapedStringContents(state: State, value: string): void {
  let chunkStart = 0;

  const { length } = value;
  for (let i = 0; i < length; i++) {
    let escape: string | null = null;

    const code = value.charCodeAt(i);
    switch (code) {
      case 0:
        if (i + 1 < length) {
          const next = value.charCodeAt(i + 1);
          if (next >= "0".charCodeAt(0) && next <= "9".charCodeAt(0)) {
            escape = "\\x00";
            break;
          }
        }
        escape = "\\0";
        break;
      case 7:
        escape = "\\x07";
        break;
      case 8:
        escape = "\\b";
        break;
      case 11:
        escape = "\\v";
        break;
      case 12:
        escape = "\\f";
        break;
      case 10:
        escape = "\\n";
        break;
      case 13:
        escape = "\\r";
        break;
      case 27:
        escape = "\\x1B";
        break;
      case 92:
        escape = "\\\\";
        break;
      case 34: // "
        escape = '\\"';
        break;
      case 60: // <
        // `</script` -> `<\/script`
        if (/^<\/script/i.test(value.slice(i, i + 8))) {
          escape = "<\\";
          break;
        }
        continue;
      case 0x2028:
        escape = "\\u2028";
        break;
      case 0x2029:
        escape = "\\u2029";
        break;
      case 0xa0:
        escape = "\\xA0";
        break;
      default:
        if (code >= 0xd800 && code <= 0xdfff) {
          // Escape lone surrogates. Paired surrogates pass through.
          if (code <= 0xdbff && i + 1 < length) {
            const next = value.charCodeAt(i + 1);
            if (next >= 0xdc00 && next <= 0xdfff) {
              // Valid pair, both halves stay in the chunk
              i++;
              continue;
            }
          }
          escape = "\\u" + code.toString(16);
          break;
        }

        continue;
    }

    debugAssert(escape != null);

    writeNoLast(state, value.slice(chunkStart, i));
    writeNoLast(state, escape);
    chunkStart = i + 1;
  }

  writeNoLast(state, value.slice(chunkStart));
}

// `</script` -> `<\/script` in template literal quasis.
/**
 * Break up any `</script` in text which is printed raw, so the output cannot end an inline script
 * element early.
 *
 * For template literal quasis, which are printed from their raw text and so never pass through the
 * string escaper.
 */
export function escapeScriptCloseTag(text: string): string {
  if (!text.includes("</")) return text;
  return text.replace(/<\/(script)/gi, "<\\/$1");
}
