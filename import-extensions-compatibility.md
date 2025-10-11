# import/extensions Rule: ESLint Compatibility Guide

**Date**: 2025-01-11
**Rule**: `import/extensions`
**Plugin**: `eslint-plugin-import`
**oxc Compatibility**: **70%**

---

## Quick Reference

| Feature                                             | ESLint | oxc | Notes                                                               |
| --------------------------------------------------- | ------ | --- | ------------------------------------------------------------------- |
| Basic configs (`always`, `never`, `ignorePackages`) | ✅     | ✅  | Fully compatible                                                    |
| Per-extension overrides                             | ✅     | ✅  | Fully compatible                                                    |
| Relative imports                                    | ✅     | ✅  | Fully compatible                                                    |
| Package imports                                     | ✅     | ✅  | Fully compatible                                                    |
| TypeScript type imports                             | ✅     | ✅  | Fully compatible                                                    |
| Query strings                                       | ✅     | ✅  | Fully compatible                                                    |
| **Custom extensions** (.vue, .svelte, .hbs)         | ✅     | ❌  | **Not supported** - See [Workaround](#custom-extensions-workaround) |
| **Path aliases** (@/)                               | ✅     | ⚠️   | **Bug** - @/ treated as scoped package                              |
| **Path group overrides**                            | ✅     | ❌  | Not implemented                                                     |
| **Multiple resolvers**                              | ✅     | ❌  | Not implemented                                                     |

---

## Fully Compatible Features (90% of use cases)

### 1. Basic Configuration

oxc fully supports the three main configuration modes:

#### `'always'` - Require extensions for all imports

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "always"]
  }
}
```

**Behavior**: All imports must include file extensions

```javascript
// ✅ PASS
import bar from './bar.json';
import Component from './Component.jsx';
import foo from './foo.js';

// ❌ FAIL
import bar from './bar';
import foo from './foo';
```

---

#### `'never'` - Never use extensions

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "never"]
  }
}
```

**Behavior**: No imports should have file extensions

```javascript
// ✅ PASS
import bar from './bar';
import Component from './Component';
import foo from './foo';

// ❌ FAIL
import bar from './bar.json';
import foo from './foo.js';
```

---

#### `'ignorePackages'` - Require extensions except for packages

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "ignorePackages"]
  }
}
```

**Behavior**: Require extensions for relative imports, but not for packages

```javascript
// ✅ PASS
import lodash from 'lodash'; // Package - no extension needed
import React from 'react'; // Package - no extension needed
import bar from './bar.json';
import foo from './foo.js';

// ❌ FAIL
import bar from './bar'; // Relative - extension required
import foo from './foo'; // Relative - extension required
```

---

### 2. Per-Extension Overrides

You can specify different rules for different file types:

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "always", {
      "js": "never",     // JS files should not have extensions
      "jsx": "never",    // JSX files should not have extensions
      "ts": "never",     // TS files should not have extensions
      "tsx": "never",    // TSX files should not have extensions
      "json": "always"   // JSON files must have extensions
    }]
  }
}
```

**Supported extensions**: `js`, `jsx`, `ts`, `tsx`, `json`

```javascript
// ✅ PASS
import Component from './Component'; // .jsx inferred
import config from './config.json'; // .json explicit
import utils from './utils'; // .js inferred

// ❌ FAIL
import Component from './Component.jsx'; // jsx should be omitted
import config from './config'; // json must be explicit
import utils from './utils.js'; // js should be omitted
```

---

### 3. Configuration Inheritance

When using `ignorePackages` with per-extension overrides, per-extension configs take precedence:

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "ignorePackages", {
      "js": "never",
      "ts": "never"
    }]
  }
}
```

**Behavior**:

- First argument `"ignorePackages"` sets the default: require extensions for non-packages
- Per-extension overrides (`js: "never"`, `ts: "never"`) override the default
- Result: JS/TS files don't need extensions, other types follow `ignorePackages` rule

```javascript
// ✅ PASS
import lodash from 'lodash'; // Package, no extension needed
import bar from './bar'; // TS inferred, no extension needed
import data from './data.json'; // JSON explicit (follows ignorePackages)
import foo from './foo'; // JS inferred, no extension needed

// ❌ FAIL
import bar from './bar.ts'; // ts override says "never"
import foo from './foo.js'; // js override says "never"
```

**This behavior is identical between oxc and ESLint** ✅

---

### 4. TypeScript Type Imports

By default, type-only imports are ignored:

```javascript
// ✅ PASS - Type imports ignored by default
import type { MyType } from './types';
export type { OtherType } from './other';
```

To check type imports, use `checkTypeImports`:

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "always", {
      "checkTypeImports": true
    }]
  }
}
```

```javascript
// ✅ PASS
import type { MyType } from './types.ts';

// ❌ FAIL
import type { MyType } from './types';  // Missing .ts extension
```

---

### 5. Import Types Supported

| Import Type            | Example                        | oxc Support       |
| ---------------------- | ------------------------------ | ----------------- |
| **Relative**           | `import './foo'`               | ✅                |
| **Parent directory**   | `import '../bar'`              | ✅                |
| **Bare packages**      | `import 'lodash'`              | ✅                |
| **Scoped packages**    | `import '@babel/core'`         | ✅                |
| **Package subpaths**   | `import 'lodash/fp'`           | ✅                |
| **Path alias ~/**      | `import '~/common/utils'`      | ✅                |
| **Path alias @/**      | `import '@/components/Button'` | ⚠️ Bug (see below) |
| **Query strings**      | `import './foo.js?v=1'`        | ✅                |
| **Directory imports**  | `import '.'` or `import '..'`  | ✅                |
| **Export re-exports**  | `export { foo } from './foo'`  | ✅                |
| **Require statements** | `require('./foo')`             | ✅                |

---

## Known Differences (10% of use cases)

### 1. Path Alias Detection (@/) - **BUG**

**Status**: ⚠️ **Known Bug** - Will be fixed in Phase 2
**Priority**: P1 (Critical)
**Tracking**: See implementation plan Phase 2.1

#### The Problem

oxc currently treats `@/` as a scoped package (like `@babel/core`) instead of a path alias.

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "ignorePackages"]
  }
}
```

```javascript
// ❌ INCORRECT BEHAVIOR (oxc)
import Button from '@/components/Button';
// oxc treats this as scoped package → ignores it (no error)
// ESLint treats this as path alias → requires extension (error)

// ✅ CORRECT BEHAVIOR (ESLint)
import Button from '@/components/Button.jsx'; // Must have extension
```

#### Affected Projects

This bug affects projects using `@/` as a path alias, which is very common in:

- **Vue.js** projects (default Vite/Vue CLI setup)
- **React** projects with custom path aliases
- **Next.js** projects
- **Nuxt** projects

#### Workaround

**Option 1**: Use `~/` instead of `@/` for path aliases

```javascript
// tsconfig.json or jsconfig.json
{
  "compilerOptions": {
    "paths": {
      "~/*": ["./src/*"]  // Use ~/ instead of @/
    }
  }
}
```

```javascript
// ✅ WORKS with oxc
import Button from '~/components/Button.jsx';
```

**Option 2**: Disable the rule for files using `@/` aliases

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": "off"  // Disable entirely
  }
}
```

#### When Will This Be Fixed?

This is a **P1 (Critical) bug** scheduled for Phase 2 (Weeks 3-4) of the implementation plan. The fix involves:

1. Distinguishing `@/` (path alias) from `@org/pkg` (scoped package)
2. Checking for slash position: `@/` has slash at position 1, `@org/` has slash at position 4+

---

### 2. Custom File Extensions (.vue, .svelte, .hbs, etc.)

**Status**: ❌ **Not Supported**
**Priority**: P1 (Critical) - 32 priority points
**Tracking**: See implementation plan Phase 3.1

#### The Problem

oxc only supports **5 hardcoded extensions**: `js`, `jsx`, `ts`, `tsx`, `json`

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "always", {
      "js": "never",
      "vue": "always"  // ❌ This is ignored by oxc
    }]
  }
}
```

```javascript
// ❌ NOT ENFORCED (oxc)
import Component from './Component'; // .vue extension not checked

// ✅ ENFORCED (ESLint with settings)
import Component from './Component.vue'; // Required
```

#### Affected Projects

- **Vue.js** projects (`.vue` files)
- **Svelte** projects (`.svelte` files)
- **Handlebars** projects (`.hbs` files)
- **Stylus** projects (`.styl` files)
- **CSS Modules** (`.module.css` files)
- Any project using custom file extensions

#### Workaround {#custom-extensions-workaround}

**Option 1**: Disable the rule for framework files

```javascript
// .eslintrc.json
{
  "overrides": [
    {
      "files": ["*.vue", "*.svelte"],
      "rules": {
        "import/extensions": "off"
      }
    }
  ]
}
```

**Option 2**: Use ESLint for framework files, oxc for JS/TS files

```bash
# Check JS/TS with oxc (fast)
oxc lint src/**/*.{js,ts,jsx,tsx}

# Check Vue files with ESLint (accurate)
eslint src/**/*.vue
```

#### When Will This Be Supported?

This is the **highest priority feature** (P1, 32 points) scheduled for Phase 3 (Months 2-3). Implementation requires:

1. Runtime extension registration system
2. Dynamic per-extension configuration
3. Update validation logic to support arbitrary extensions

**Estimated effort**: 24 hours

---

### 3. Package Detection - Different Approach

**Status**: ⚠️ **Different Implementation**
**Priority**: P1 (High) - 20 priority points
**Tracking**: See implementation plan Phase 2.2

#### The Difference

| Feature                   | ESLint                           | oxc                |
| ------------------------- | -------------------------------- | ------------------ |
| **Resolution method**     | Filesystem (checks node_modules) | String heuristics  |
| **Symlink support**       | ✅ Yes                           | ❌ No              |
| **Custom module folders** | ✅ Yes (via settings)            | ❌ No              |
| **Speed**                 | Slower (I/O)                     | Faster (no I/O)    |
| **Accuracy**              | Higher                           | Lower (edge cases) |

#### Edge Cases Where They Differ

**Example 1**: Custom external module folders

```javascript
// .eslintrc.json
{
  "settings": {
    "import/external-module-folders": ["node_modules", ".pnpm", "vendor"]
  }
}
```

- **ESLint**: Recognizes packages in `.pnpm/` and `vendor/` folders
- **oxc**: Only uses string heuristics (doesn't check filesystem)

**Example 2**: Symlinked packages

```javascript
import utils from 'symlinked-package';
```

- **ESLint**: Checks if `node_modules/symlinked-package` exists (even if symlinked)
- **oxc**: Uses string pattern (treats as package if not relative/absolute)

#### Impact

For most projects (95%), the behavior is the same. Differences appear in:

- **Monorepos** with custom package locations
- **pnpm** with symlinked dependencies
- **Yarn workspaces** with custom setups
- **Custom module resolution** systems

#### Philosophy

- **ESLint**: "Check what actually exists on disk"
- **oxc**: "Validate syntax without I/O for speed"

Both approaches are valid. oxc prioritizes speed and determinism, while ESLint prioritizes accuracy.

---

### 4. Path Group Overrides

**Status**: ❌ **Not Implemented**
**Priority**: P2 (High) - 15 priority points
**Tracking**: See implementation plan Phase 3.2

#### What This Is

ESLint supports pattern-based rule overrides for specific import paths:

```javascript
// .eslintrc.json
{
  "rules": {
    "import/extensions": ["error", "ignorePackages", {
      "pathGroupOverrides": [
        // Enforce extensions for monorepo packages
        {
          "pattern": "packages/*/src/**",
          "action": "enforce",
          "extensions": { "js": "always", "ts": "always" }
        },
        // Ignore extensions for generated code
        {
          "pattern": "**/generated/**",
          "action": "ignore"
        }
      ]
    }]
  }
}
```

#### Use Cases

- **Monorepos**: Different rules for different packages
- **Generated code**: Ignore linting for generated files
- **Legacy code**: Different rules for old vs new code
- **Vendor code**: Ignore third-party code patterns

#### Workaround

Use ESLint overrides (less powerful, but similar):

```javascript
// .eslintrc.json
{
  "overrides": [
    {
      "files": ["packages/*/src/**"],
      "rules": {
        "import/extensions": ["error", "always"]
      }
    },
    {
      "files": ["**/generated/**"],
      "rules": {
        "import/extensions": "off"
      }
    }
  ]
}
```

**Limitation**: ESLint overrides apply to **file paths**, not **import specifiers**.

---

### 5. Multiple Resolvers (webpack, TypeScript)

**Status**: ❌ **Not Implemented**
**Priority**: P2 (Medium) - 9 priority points
**Tracking**: See implementation plan Phase 4.2

#### What This Is

ESLint supports pluggable module resolvers:

```javascript
// .eslintrc.json
{
  "settings": {
    "import/resolver": {
      // Use webpack resolver
      "webpack": {
        "config": "webpack.config.js"
      },
      // Or TypeScript resolver
      "typescript": {
        "alwaysTryTypes": true,
        "project": "./tsconfig.json"
      }
    }
  }
}
```

#### Impact

- **Webpack projects**: Can't use webpack-specific resolution
- **TypeScript path mapping**: Limited support without TS resolver
- **Custom resolvers**: Can't use custom resolution logic

#### Workaround

oxc uses built-in Node.js-style resolution only. For most projects, this is sufficient.

---

## Error Message Differences

### ESLint (with resolution)

```
Missing file extension "js" for "./foo"
```

**Specific**: Tells you exactly which extension is missing

### oxc (syntax-only)

```
Missing file extension in import declaration
```

**Generic**: Tells you an extension is missing but not which one

### Why?

oxc doesn't resolve files (no filesystem I/O), so it can't determine the actual file extension. This is a tradeoff for speed and determinism.

---

## Migration Guide

### From ESLint to oxc

#### Step 1: Check for unsupported features

Run this checklist:

- [ ] Are you using **custom extensions** (.vue, .svelte, .hbs)?
  - → **Blocker**: Disable rule or wait for Phase 3
- [ ] Are you using **@/ path aliases**?
  - → **Bug**: Switch to ~/ or wait for Phase 2
- [ ] Are you using **path group overrides**?
  - → Use ESLint overrides instead
- [ ] Are you using **custom resolvers** (webpack, TS)?
  - → Will work differently, test thoroughly

#### Step 2: Test your configuration

Create a test file with various import patterns:

```javascript
// test-imports.js
import utils from '@/utils'; // Path alias
import pkg from '@babel/core'; // Scoped package
import lodash from 'lodash';
import bar from './bar.js';
import Component from './Component.vue'; // Custom extension
import foo from './foo';
```

Run both linters and compare:

```bash
# ESLint
eslint test-imports.js

# oxc
oxc lint test-imports.js
```

#### Step 3: Adjust configuration

If oxc reports different errors:

1. **Path aliases**: Switch from `@/` to `~/`
2. **Custom extensions**: Add override to disable rule
3. **Path group overrides**: Use ESLint overrides instead

#### Step 4: Document differences

Add comments to your config explaining any differences:

```javascript
// .eslintrc.json
{
  "rules": {
    // Note: oxc doesn't support .vue extensions yet
    // See: /path/to/import-extensions-compatibility.md
    "import/extensions": ["error", "ignorePackages", {
      "js": "never",
      "ts": "never"
      // "vue": "always"  // Commented out for oxc compatibility
    }]
  }
}
```

---

## Compatibility Testing

To verify your configuration works with oxc, run:

```bash
# Run oxc linter
cargo run -p oxc_linter --example linter -- src/

# Or if using oxc CLI
oxc lint src/
```

Compare results with ESLint:

```bash
# Run ESLint
eslint src/
```

If you see different errors, refer to the [Known Differences](#known-differences-10-of-use-cases) section.

---

## Future Roadmap

### Phase 1 (Weeks 1-2) - **CURRENT**

- ✅ Add missing test cases
- ✅ Create compatibility documentation (this file)
- ✅ Add explanatory comments to tests

### Phase 2 (Weeks 3-4)

- 🔄 Fix @/ path alias bug
- 🔄 Improve package detection algorithm
- 🔄 Add configuration validation
- **Target**: 80% compatibility

### Phase 3 (Months 2-3)

- 🔜 Custom extension support (.vue, .svelte, etc.)
- 🔜 Path group overrides
- 🔜 Integration test suite
- **Target**: 90% compatibility

### Phase 4 (Months 4-6)

- 🔜 Resolver infrastructure
- 🔜 Full resolution integration
- 🔜 Multi-parser testing
- **Target**: 90%+ compatibility (full ESLint parity)

---

## FAQ

### Q: Should I use oxc or ESLint for import/extensions?

**A**: Depends on your project:

**Use oxc if**:

- You're working with standard JS/TS projects
- You want fast linting
- You don't use framework-specific extensions
- You don't use @/ as a path alias (or can switch to ~/)

**Use ESLint if**:

- You use Vue, Svelte, or other frameworks
- You need @/ path alias support
- You need custom resolver support
- You need path group overrides

**Use both**:

- oxc for JS/TS files (speed)
- ESLint for framework files (accuracy)

### Q: Will oxc ever have 100% ESLint compatibility?

**A**: The goal is 90%+ compatibility. Some ESLint features (like custom resolvers) may not be implemented due to architectural differences. oxc prioritizes speed and determinism over 100% compatibility.

### Q: Can I contribute to improving compatibility?

**A**: Yes! See the implementation plan (`testing-harness-implementation-plan.md`) for details on what needs to be done. The highest priority items are:

1. Custom extension support (Phase 3.1)
2. Path alias bug fix (Phase 2.1)
3. Package detection improvements (Phase 2.2)

### Q: Where can I report issues?

**A**: Report issues at: https://github.com/oxc-project/oxc/issues

Include:

- Your configuration
- Example code
- Expected behavior (from ESLint)
- Actual behavior (from oxc)

---

## See Also

- [Testing Coverage Analysis](./testing-coverage-analysis.md) - Detailed gap analysis
- [Implementation Plan](./testing-harness-implementation-plan.md) - Roadmap for improvements
- [oxc Test Analysis](./oxc-import-extensions-tests.md) - Complete oxc test catalog
- [ESLint Test Analysis](./eslint-extensions-test-analysis.md) - Complete ESLint test catalog

---

## Document Metadata

**Last Updated**: 2025-01-11
**oxc Version**: 1.22.0
**ESLint Plugin Version**: eslint-plugin-import@latest
**Compatibility Level**: 70%
**Maintainer**: oxc-project
**Status**: Active
