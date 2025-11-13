# Ember Plugin Test Coverage

This directory contains comprehensive tests for `eslint-plugin-ember` integration with oxlint.

## Test Files

### 1. `test-ember-plugin.gjs`
**Purpose**: Basic integration test  
**Expected Violations**: 
- `ember/require-computed-property-dependencies`

**Status**: ✅ Passing

### 2. `test-comprehensive.gjs`
**Purpose**: Multiple rule violations in one file  
**Expected Violations**:
- `ember/require-computed-property-dependencies`

**Status**: ✅ Passing

### 3. `test-empty-component.gjs`
**Purpose**: Test empty Glimmer component class rule  
**Expected Violations**: 
- `ember/no-empty-glimmer-component-classes` (may not trigger as expected)

**Status**: ⚠️ Rule may not trigger - needs investigation

### 4. `test-computed-dependencies.gjs`
**Purpose**: Comprehensive test of computed property dependencies rule  
**Expected Violations**:
- `ember/require-computed-property-dependencies` (multiple violations)

**Status**: ✅ Passing - Detects 4 violations

### 5. `test-valid-component.gjs`
**Purpose**: Component with no rule violations (should pass)  
**Expected Violations**: None

**Status**: ✅ Passing

## Running Tests

### Run all tests:
```bash
npm run test:comprehensive
# or
node test-runner.js
```

### Run individual test:
```bash
node ../../apps/oxlint/dist/cli.js test-ember-plugin.gjs --disable-nested-config
```

## Test Results

Current success rate: **80-100%** (depending on rule configuration)

### Working Rules
- ✅ `ember/require-computed-property-dependencies` - Fully working, detects missing dependencies

### Rules Needing Investigation
- ⚠️ `ember/no-empty-glimmer-component-classes` - May need different test case or rule configuration
- ⚠️ `ember/no-get` - May need different test patterns

## Known Issues

1. **String literal deserialization**: Some test files with string literals in class properties may cause deserialization errors. Workaround: Use empty strings or numeric values.

2. **Nested property access**: The `require-computed-property-dependencies` rule correctly detects nested property access (e.g., `items.length`) as requiring the nested property in dependencies, not just the parent property.

3. **Empty component rule**: The `no-empty-glimmer-component-classes` rule may not trigger as expected. This could be because:
   - The rule definition differs from expectations
   - The component needs to be truly empty (no template)
   - The rule requires different configuration

## Adding New Tests

To add a new test:

1. Create a `.gjs` file with intentional violations
2. Add it to `test-runner.js` in the `testFiles` array:
   ```javascript
   {
     file: 'your-test.gjs',
     description: 'Description of what you\'re testing',
     expectedViolations: ['ember/rule-name'],
     shouldPass: false
   }
   ```
3. Run the test runner to verify

## Future Test Ideas

- Test with TypeScript `.gts` files
- Test with more complex template structures
- Test with multiple components in one file
- Test edge cases with computed properties
- Test other ember rules (no-get, no-set, etc.)

