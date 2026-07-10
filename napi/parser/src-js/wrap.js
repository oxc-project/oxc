export function wrap(result) {
  // Eagerly move the serialized AST out of the native `ParseResult`, so its Rust-side
  // memory is freed as soon as this function returns.
  //
  // The native object's memory is only freed by a NAPI finalizer, and finalizers only
  // run on event-loop turns. If the serialized AST (typically 10-20x the size of the
  // source text) stayed inside the native object, code which calls `parseSync` in a
  // loop without yielding to the event loop would retain the serialized AST of every
  // file it parses - gigabytes over enough files - with no way for GC to reclaim it,
  // because V8 cannot see native memory. Moving it into a JS string here hands it to
  // V8, which frees it under normal GC pressure.
  // Deserializing the AST itself (`JSON.parse`) remains lazy, as do the other fields,
  // whose conversion cost is not worth paying for callers that never read them
  // (their retained native memory is small compared to the serialized AST).
  const programJson = result.program;
  let program, module, comments, errors;
  return {
    get program() {
      if (!program) program = jsonParseAst(programJson);
      return program;
    },
    get module() {
      if (!module) module = result.module;
      return module;
    },
    get comments() {
      if (!comments) comments = result.comments;
      return comments;
    },
    get errors() {
      if (!errors) errors = result.errors;
      return errors;
    },
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
