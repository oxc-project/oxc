export function wrap(result) {
  // Take every field out of the native `ParseResult` before returning.
  //
  // Each native getter does a `mem::take`, so reading a field frees its Rust-side memory.
  // Fields which are never read stay alive until the NAPI finalizer runs, which only happens
  // on an event-loop turn after a GC. V8 cannot see this native memory, so it does not
  // collect any sooner. Code which only reads `errors`, or calls `parseSync` in a loop,
  // would retain the serialized AST of every file it parses.
  //
  // Once taken, the values are ordinary JS values, which V8 frees under normal GC pressure.
  // Only `JSON.parse` of the AST stays lazy.
  const { module, comments, errors } = result;
  let programJson = result.program,
    program;
  return {
    get program() {
      if (!program) {
        program = jsonParseAst(programJson);
        // Release the JSON string. Otherwise it is kept alive as long as the AST is.
        programJson = null;
      }
      return program;
    },
    module,
    comments,
    errors,
  };
}

// Used by `napi/playground/scripts/patch.js`.
//
// Set `value` field of `Literal`s which are `BigInt`s or `RegExp`s.
//
// Returned JSON contains an array `fixes` with paths to these nodes
// e.g. for `123n; foo(/xyz/)`, `fixes` will be
// `[["body", 0, "expression"], ["body", 1, "expression", "arguments", 0]]`.
//
// Walk down the AST to these nodes and alter them.
// Compiling the list of fixes on Rust side avoids having to do a full AST traversal on JS side
// to locate the likely very few `Literal`s which need fixing.
export function jsonParseAst(programJson) {
  const { node: program, fixes } = JSON.parse(programJson);
  for (const fixPath of fixes) {
    applyFix(program, fixPath);
  }
  return program;
}

function applyFix(program, fixPath) {
  let node = program;
  for (const key of fixPath) {
    node = node[key];
  }

  if (node.bigint) {
    node.value = BigInt(node.bigint);
  } else {
    try {
      node.value = RegExp(node.regex.pattern, node.regex.flags);
    } catch {
      // Invalid regexp, or valid regexp using syntax not supported by this version of NodeJS
    }
  }
}
