# eslint-plugin-ember Integration Test Status

## Overview

This directory contains a comprehensive test for `eslint-plugin-ember` integration with oxlint.

## Test Files

- **`.oxlintrc.json`** - Configuration with `eslint-plugin-ember` enabled
- **`test-ember-plugin.gjs`** - Test file with intentional ember rule violations
- **`test-ember-plugin.js`** - Test runner script

## Current Status

### ✅ What's Working

1. **Plugin Loading**: `eslint-plugin-ember` successfully loads via `jsPlugins` configuration
2. **Parser Integration**: Custom parser (`ember-eslint-parser`) works correctly
3. **Full AST Storage**: Full AST with Glimmer nodes is stored and accessible
4. **Context Methods**: Added ESLint compatibility methods:
   - `context.getFilename()` - Returns file path
   - `context.getSourceCode()` - Returns SourceCode object
5. **Rule Configuration**: Ember rules are correctly configured in `.oxlintrc.json`
6. **Rule Execution**: ✅ **Rules are executing successfully!** Ember plugin rules can now lint `.gjs` files and report violations.
7. **Store Management**: Fixed plugin store cloning to ensure registered plugins are retained in ConfigStore
8. **Rule ID Mapping**: Fixed mapping between Rust ExternalRuleId and JavaScript registeredRules array indices

## Configuration

The test uses the following configuration in `.oxlintrc.json`:

```json
{
  "parser": "ember-eslint-parser",
  "parserOptions": {
    "ecmaVersion": 2022,
    "sourceType": "module"
  },
  "jsPlugins": ["eslint-plugin-ember"],
  "rules": {
    "ember/no-empty-glimmer-component-classes": "error",
    "ember/require-computed-property-dependencies": "error",
    "ember/no-get": "warn"
  }
}
```

## Running the Test

```bash
cd tests/ember-parser-test
npm install
npm run test:ember
```

Or directly:
```bash
node test-ember-plugin.js
```

## Expected Behavior

✅ **All working!** The test successfully:
1. Loads `eslint-plugin-ember` successfully
2. Parses `.gjs` files with `ember-eslint-parser`
3. Executes ember rules on the test file
4. Reports violations (currently detecting `ember/require-computed-property-dependencies`)

## Test Results

Running the test shows:
- ✅ Plugin loads without errors
- ✅ Rules execute and report violations
- ✅ Example violation detected: `ember(require-computed-property-dependencies): Use of undeclared dependencies in computed property: name.length`

## Next Steps

1. ✅ **Fixed Rust Panic**: Resolved rule ID indexing issue by fixing store cloning
2. ✅ **Rule Execution Verified**: Rules are executing and reporting violations
3. **Add More Test Cases**: Test additional ember rules and edge cases
4. ✅ **Documentation**: Updated to reflect working state

## Implementation Details

### Context Compatibility Methods Added

To support `eslint-plugin-ember`, we added ESLint compatibility methods to the `Context` class:

```typescript
// apps/oxlint/src-js/plugins/context.ts

getFilename(): string {
  return getInternal(this, 'call `context.getFilename()`').filePath;
}

getSourceCode(): SourceCode {
  getInternal(this, 'call `context.getSourceCode()`');
  return SOURCE_CODE;
}
```

These methods allow plugins that use ESLint's method-based API to work with oxlint's context object.

### Store Management Fix

Fixed a critical bug where `ExternalPluginStore` was being re-initialized as empty during `ConfigStore` construction. The fix ensures that:

1. `ExternalPluginStore`, `ExternalPlugin`, and `ExternalRule` implement `Clone`
2. `ConfigBuilder::build()` clones the existing stores instead of creating empty ones
3. Registered plugins are retained throughout the configuration lifecycle

### Rule ID Mapping Fix

Added `ruleIdToIndex` mapping in JavaScript to correctly translate `ExternalRuleId` (from Rust) to the internal `registeredRules` array index. This ensures rules can be correctly retrieved when executing.

## Related Files

- `apps/oxlint/src-js/plugins/context.ts` - Context class with compatibility methods
- `apps/oxlint/src-js/plugins/full_ast_store.ts` - Full AST storage for custom nodes
- `apps/oxlint/src-js/plugins/source_code.ts` - SourceCode implementation with full AST support

