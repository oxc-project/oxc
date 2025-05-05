// Note: This code is repeated in `wrap.cjs`.
// Any changes should be applied in that file too.

import visitorKeys from './generated/visitor-keys.mjs';

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

// Used by napi/playground/patch.mjs
export function jsonParseAst(programJson) {
  const program = JSON.parse(programJson);
  transformNode(program);
  return program;
}

function transformNode(node) {
  if (!node) return;
  if (Array.isArray(node)) {
    for (const el of node) {
      transformNode(el);
    }
    return;
  }

  if (node.type === 'Literal') {
    transformLiteral(node);
    return;
  }

  const keys = visitorKeys[node.type];
  if (!keys) return;
  for (const key of keys) {
    transformNode(node[key]);
  }
}

function transformLiteral(node) {
  // Set `value` field of `Literal`s for `BigInt`s and `RegExp`s.
  // This is not possible to do on Rust side, as neither can be represented correctly in JSON.
  if (node.bigint) {
    node.value = BigInt(node.bigint);
  } else if (node.regex) {
    try {
      node.value = RegExp(node.regex.pattern, node.regex.flags);
    } catch (_err) {
      // Invalid regexp, or valid regexp using syntax not supported by this version of NodeJS
    }
  }
}
