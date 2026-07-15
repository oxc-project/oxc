import { printRawSync } from "./bindings.js";
import { returnBufferToCache } from "./raw-transfer-back/buffer.js";
import { encodeProgram } from "./raw-transfer-back/serialize.js";
import { rawTransferBackSupported } from "./supported.js";

export { rawTransferBackSupported } from "./supported.js";

/**
 * Print an ESTree AST to JavaScript / TypeScript source code.
 *
 * The AST must be in the shape `oxc-parser` produces (ESTree / TS-ESTree compatible).
 *
 * @example
 * import { parseSync } from "oxc-parser";
 * import { print } from "oxc-codegen";
 *
 * const { program } = parseSync("test.js", "const x = 1 + 2;");
 * print(program).code; // "const x = 1 + 2;\n"
 *
 * @param {Object} program - ESTree `Program` object
 * @param {Object} [options] - Print options
 * @param {string} [options.sourceText] - Original source text; required for printing comments
 *   and for accurate source maps
 * @param {string} [options.filename] - Source filename, used as the `source` of the source map
 * @param {boolean} [options.sourcemap] - Produce a source map, returned as `map` on the result
 * @param {boolean} [options.singleQuote] - Use single quotes instead of double quotes
 * @param {boolean} [options.minify] - Remove whitespace
 * @param {boolean|Object} [options.comments] - Print comments (requires `sourceText`)
 * @param {'space'|'tab'} [options.indentChar] - Indentation character
 * @param {number} [options.indentWidth] - Characters per indentation level
 * @param {number} [options.initialIndent] - Initial indentation level
 * @returns {Object} - Object of form `{ code, map?, errors }`
 */
export function print(program, options) {
  if (program === null || typeof program !== "object" || program.type !== "Program") {
    throw new TypeError(
      "Expected a `Program` node from `oxc-parser`, got " +
        (program === null ? "null" : typeof program !== "object" ? typeof program : program.type),
    );
  }

  if (!rawTransferBackSupported()) {
    throw new Error(
      "`print` is not supported on this platform. " +
        "It requires a 64-bit little-endian system, and NodeJS >= v22.0.0 or Deno >= v2.0.0.",
    );
  }

  const { buffer, programOffset, sourceStart, sourceLen } = encodeProgram(
    program,
    options?.sourceText,
  );
  try {
    return printRawSync(buffer.block, programOffset, sourceStart, sourceLen, options);
  } finally {
    returnBufferToCache(buffer);
  }
}
