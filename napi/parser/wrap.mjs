export function wrap(result) {
  let program, module, comments, errors;
  return {
    get program() {
      if (!program) {
        program = jsonParseAst(result.program);
      }
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
export function jsonParseAst(ast) {
  return JSON.parse(ast, function(key, value) {
    // Set `value` field of `Literal`s for `BigInt`s and `RegExp`s.
    // This is not possible to do on Rust side, as neither can be represented correctly in JSON.
    if (value === null && key === 'value' && Object.hasOwn(this, 'type') && this.type === 'Literal') {
      if (Object.hasOwn(this, 'bigint')) {
        return BigInt(this.bigint);
      }
      if (Object.hasOwn(this, 'regex')) {
        const { regex } = this;
        try {
          return RegExp(regex.pattern, regex.flags);
        } catch (_err) {
          // Invalid regexp, or valid regexp using syntax not supported by this version of NodeJS
        }
      }
    }
    return value;
  });
}
