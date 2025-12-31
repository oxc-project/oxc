# Custom JavaScript Parsers

> **Warning**: Custom parsers are experimental and not subject to semver. Breaking changes may occur while this feature is under development.

Custom parsers allow oxlint to lint files that use non-standard JavaScript syntax, such as template languages (Glimmer/Ember, Vue, Svelte, Marko) or domain-specific languages that compile to JavaScript.

## How It Works

When a file matches a custom parser's file patterns, oxlint:

1. Loads the JavaScript parser module
2. Parses the file using the parser's `parse()` or `parseForESLint()` function
3. Deserializes the resulting ESTree AST into oxc's internal representation
4. Runs Rust-based lint rules on the deserialized AST
5. Runs any configured JS plugin rules on the original AST

This allows most oxlint rules to work on custom file types without modification.

## Configuration

Custom parsers are configured in `.oxlintrc.json` using the `overrides` field:

```json
{
  "overrides": [
    {
      "files": ["*.gjs", "*.gts"],
      "jsParser": "ember-eslint-parser",
      "jsParserOptions": {
        "sourceType": "module"
      },
      "rules": {
        "no-unused-vars": "error"
      }
    }
  ]
}
```

### Configuration Options

#### `jsParser`

The parser module to use. Can be:
- An npm package name: `"ember-eslint-parser"`
- A relative path to a local parser: `"./my-parser.js"`

The parser must implement the standard ESLint parser interface:
- `parse(code, options)` - Returns an ESTree-compatible AST
- `parseForESLint(code, options)` - Returns `{ ast, scopeManager?, visitorKeys?, services? }`

If both are available, `parseForESLint` is preferred as it can provide scope information.

#### `jsParserOptions`

Options passed to the parser's `parse` or `parseForESLint` function. Common options include:

- `sourceType`: `"module"` or `"script"`
- `ecmaVersion`: ECMAScript version (e.g., `2022`)
- Parser-specific options (consult the parser's documentation)

#### `files`

Glob patterns for files that should use this parser. Patterns are relative to the config file location.

## Parser Interface

Custom parsers must implement the ESLint parser interface:

```typescript
// Minimal interface - just returns the AST
export function parse(code: string, options?: ParserOptions): ESTreeProgram;

// Full interface - returns AST with additional metadata
export function parseForESLint(code: string, options?: ParserOptions): {
  ast: ESTreeProgram;
  scopeManager?: ScopeManager;  // For accurate scope analysis
  visitorKeys?: VisitorKeys;    // Custom AST node visitor keys
  services?: ParserServices;    // Additional services for rules
};
```

### AST Requirements

The returned AST must:
- Have `type: "Program"` as the root node
- Include span information on all nodes (either `start`/`end` or `range: [start, end]`)
- Use standard ESTree node types for JavaScript constructs

For nodes representing custom syntax (e.g., `<template>` tags), use placeholder nodes or custom node types. Diagnostics on unrecognized nodes will be filtered out.

### Scope Manager

If your parser can provide scope information, returning a `scopeManager` enables more accurate linting for rules that depend on scope analysis (e.g., `no-unused-vars`, `no-undef`).

## Examples

### Ember/Glimmer

```json
{
  "overrides": [
    {
      "files": ["*.gjs", "*.gts"],
      "jsParser": "ember-eslint-parser",
      "jsParserOptions": {
        "sourceType": "module",
        "requireConfigFile": false
      }
    }
  ]
}
```

### Custom DSL

For a simple custom language, you can write your own parser:

```javascript
// my-parser.js
export function parseForESLint(code, options) {
  // Parse your DSL and convert to ESTree AST
  const ast = parseMyDSL(code);

  return {
    ast,
    visitorKeys: {
      Program: ["body"],
      // ... define visitor keys for your custom nodes
    }
  };
}
```

```json
{
  "overrides": [
    {
      "files": ["*.mydsl"],
      "jsParser": "./my-parser.js"
    }
  ]
}
```

## Combining with JS Plugins

Custom parsers work alongside JS plugins. You can use a custom parser for file parsing while also using JS plugin rules:

```json
{
  "overrides": [
    {
      "files": ["*.custom"],
      "jsParser": "./parser.js",
      "jsPlugins": ["./plugin.js"],
      "rules": {
        "no-var": "warn",
        "my-plugin/custom-rule": "error"
      }
    }
  ]
}
```

In this setup:
- `no-var` (Rust rule) runs on the deserialized AST
- `my-plugin/custom-rule` (JS rule) runs on the original AST from the parser

## Known Limitations

### Language Server / IDE Support

Custom parsers are **not supported** in the oxlint language server or editor extensions. Files matching custom parser patterns will be silently skipped. This is because the language server cannot execute JavaScript parser modules.

Use the CLI for linting files with custom parsers:

```bash
oxlint --config .oxlintrc.json
```

### ESTree Compatibility

Not all ESTree node types are fully supported for deserialization. If your parser produces uncommon or non-standard nodes, some may be converted to placeholder nodes, and diagnostics on those regions will be filtered out.

Currently known limitations:
- Some TypeScript-specific AST variations may not deserialize correctly
- Very deeply nested or unusual AST structures may cause issues
- Parser-specific extensions to ESTree may not be recognized

### Performance

Custom parser support adds overhead compared to native oxc parsing:
- JSON serialization/deserialization of the AST
- JavaScript execution for the parser
- Additional memory for maintaining both AST representations

For large codebases, consider whether the parser overhead is acceptable for your use case.

### Auto-fix Support

Auto-fix (`--fix`) is supported for custom parser files, provided the parser returns accurate source spans that map directly to positions in the original file. Both Rust rules and JS plugin rules can provide fixes.

Note that oxlint cannot validate that fixes produce syntactically valid output for custom syntax files, since the fixed code may contain syntax that isn't standard JavaScript. Ensure your parser provides correct span information for reliable fix behavior.

### Scope Analysis Accuracy

If the custom parser doesn't provide a `scopeManager`, oxlint builds scope information from the deserialized AST. This may be less accurate than the parser's native scope analysis, potentially causing:
- False positives for `no-unused-vars` on variables used in custom syntax
- False negatives for `no-undef` on undefined variables

For best results, use parsers that implement `parseForESLint` and provide scope information.

## Debugging

To debug custom parser issues:

1. **Verify parser output**: Test your parser independently to ensure it produces valid ESTree
2. **Check file patterns**: Ensure your glob patterns match the intended files
3. **Enable verbose output**: Run with `RUST_LOG=debug oxlint ...` for detailed logs
4. **Simplify configuration**: Start with minimal rules and add more once basic linting works

## Future Work

- Language server support via sidecar Node.js process
- Better error messages for AST deserialization failures
- Auto-fix support for custom syntax
- Improved scope manager integration
