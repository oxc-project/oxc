import { attachCommentGroup } from "./raw-transfer/attached-comments.js";

export function wrap(result, attachComments = false) {
  let program, module, comments, errors;
  function init() {
    if (!program) {
      if (attachComments) comments = result.comments;
      program = jsonParseAst(result.program, comments);
    }
  }
  return {
    get program() {
      init();
      return program;
    },
    get module() {
      if (!module) module = result.module;
      return module;
    },
    get comments() {
      if (attachComments) init();
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
export function jsonParseAst(programJson, comments) {
  const { node: program, fixes, commentFixes } = JSON.parse(programJson);
  for (const fixPath of fixes) {
    applyFix(program, fixPath);
  }
  if (commentFixes !== undefined) {
    if (program.range !== undefined) {
      for (const comment of comments) comment.range = [comment.start, comment.end];
    }
    for (const [path, index, placement, kind, newlines, content, container] of commentFixes) {
      let node = program;
      for (const key of path) node = node[key];
      const comment = comments[index];
      comment.kind = kind;
      comment.newlines = newlines;
      comment.content = content;
      comment.container = null;
      attachCommentGroup(
        node,
        [[comment, placement]],
        container?.[0] ?? "",
        container?.[1] ?? 0,
        container?.[2] ?? 0,
      );
    }
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
