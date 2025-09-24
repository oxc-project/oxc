export function wrap(result) {
  let program, module, comments, errors;
  return {
    get program() {
      if (!program) program = jsonParseAst(result.program);
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
// `[["body", 0, "expression"], ["body", 1, "expression", "arguments", 2]]`.
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
    } catch (_err) { // oxlint-disable-line no-unused-vars
      // Invalid regexp, or valid regexp using syntax not supported by this version of NodeJS
    }
  }
}
