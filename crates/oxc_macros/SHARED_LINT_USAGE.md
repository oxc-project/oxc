# Using `declare_oxc_shared_lint!` Macro

This document explains how to use the `declare_oxc_shared_lint!` macro for creating linting rules that are shared across multiple plugins.

## Overview

The `declare_oxc_shared_lint!` macro allows you to define a lint rule once and use it in multiple plugins (e.g., jest and vitest) while sharing the same documentation. This prevents duplication and ensures consistency.

## When to Use

Use this macro when:
1. You have a lint rule that applies to multiple plugins
2. The rule has the same behavior and documentation across all plugins
3. You want to avoid duplicating documentation

## How It Works

The macro requires:
1. A shared module that contains the rule implementation and a `DOCUMENTATION` constant
2. Thin wrapper structs in each plugin folder that use the macro

## Example: valid-title Rule

### Step 1: Create the Shared Module

In `crates/oxc_linter/src/rules/shared/valid_title.rs`:

```rust
/// Shared documentation for the valid-title rule used by both jest and vitest plugins
#[cfg(feature = "ruledocs")]
pub const DOCUMENTATION: Option<&str> = Some(
    r#"### What it does

Checks that the titles of Jest and Vitest blocks are valid.

### Why is this bad?

Titles that are not valid can be misleading.

### Examples

Examples of **incorrect** code for this rule:
```javascript
describe('', () => {});
```
"#,
);

#[cfg(not(feature = "ruledocs"))]
pub const DOCUMENTATION: Option<&str> = None;

// ... rest of implementation
pub struct ValidTitleConfig {
    // config fields
}

impl ValidTitleConfig {
    pub fn from_configuration(value: &serde_json::Value) -> Result<Self, Error> {
        // parse configuration
    }

    pub fn run_rule(config: &Self, node: &Node, ctx: &LintContext) {
        // rule implementation
    }
}
```

### Step 2: Create Plugin Wrappers

In `crates/oxc_linter/src/rules/jest/valid_title.rs`:

```rust
use oxc_macros::declare_oxc_shared_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::{PossibleJestNode, shared::valid_title as SharedValidTitle},
};

#[derive(Debug, Default, Clone)]
pub struct ValidTitle(Box<SharedValidTitle::ValidTitleConfig>);

declare_oxc_shared_lint!(
    ValidTitle,
    jest,
    correctness,
    conditional_fix,
    shared_docs = crate::rules::shared::valid_title
);

impl Rule for ValidTitle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        SharedValidTitle::ValidTitleConfig::from_configuration(&value)
            .map(|config| Self(Box::new(config)))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        SharedValidTitle::ValidTitleConfig::run_rule(&self.0, jest_node, ctx);
    }
}
```

In `crates/oxc_linter/src/rules/vitest/valid_title.rs`:

```rust
use oxc_macros::declare_oxc_shared_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::{PossibleJestNode, shared::valid_title as SharedValidTitle},
};

#[derive(Debug, Default, Clone)]
pub struct ValidTitle(Box<SharedValidTitle::ValidTitleConfig>);

declare_oxc_shared_lint!(
    ValidTitle,
    vitest,
    correctness,
    conditional_fix,
    shared_docs = crate::rules::shared::valid_title
);

impl Rule for ValidTitle {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        SharedValidTitle::ValidTitleConfig::from_configuration(&value)
            .map(|config| Self(Box::new(config)))
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        SharedValidTitle::ValidTitleConfig::run_rule(&self.0, jest_node, ctx);
    }
}
```

### Step 3: Register the Rules

In `crates/oxc_linter/src/rules.rs`, add the module declarations:

```rust
pub(crate) mod jest {
    // ... other rules
    pub mod valid_title;
}

pub(crate) mod vitest {
    // ... other rules
    pub mod valid_title;
}

pub(crate) mod shared {
    pub mod valid_title;
}
```

## Macro Parameters

The `declare_oxc_shared_lint!` macro accepts the following parameters:

1. **Rule struct name**: The name of your rule struct (e.g., `ValidTitle`)
2. **Plugin name**: The plugin this instance is for (e.g., `jest`, `vitest`)
3. **Category**: The rule category (e.g., `correctness`, `style`, `suspicious`)
4. **Fix type** (optional): The auto-fix capability (e.g., `fix`, `conditional_fix`, `pending`)
5. **shared_docs**: Path to the shared documentation module

## Benefits

1. **No documentation duplication**: Write documentation once, use it everywhere
2. **Consistency**: All plugin implementations use the exact same documentation
3. **Easier maintenance**: Update documentation in one place
4. **Less boilerplate**: Shorter plugin wrapper files

## Testing

After implementing a shared rule:

1. Build the linter: `cargo build -p oxc_linter`
2. Run tests: `cargo test -p oxc_linter --lib plugin_name::rule_name`
3. Run all tests: `cargo test -p oxc_linter --lib`

## Comparison with `declare_oxc_lint!`

### Before (with `declare_oxc_lint!`):

```rust
// jest/valid_title.rs (60+ lines of docs)
declare_oxc_lint!(
    /// ### What it does
    /// 
    /// Checks that the titles of Jest and Vitest blocks are valid.
    /// ... [60 more lines of documentation] ...
    ValidTitle,
    jest,
    correctness,
    conditional_fix
);

// vitest/valid_title.rs (same 60+ lines of docs duplicated)
declare_oxc_lint!(
    /// ### What it does
    /// 
    /// Checks that the titles of Jest and Vitest blocks are valid.
    /// ... [60 more lines of documentation] ...
    ValidTitle,
    vitest,
    correctness,
    conditional_fix
);
```

### After (with `declare_oxc_shared_lint!`):

```rust
// shared/valid_title.rs (60 lines of docs, once)
pub const DOCUMENTATION: Option<&str> = Some(
    r#"### What it does
    
Checks that the titles of Jest and Vitest blocks are valid.
... [60 lines of documentation] ...
"#,
);

// jest/valid_title.rs (simple reference)
declare_oxc_shared_lint!(
    ValidTitle,
    jest,
    correctness,
    conditional_fix,
    shared_docs = crate::rules::shared::valid_title
);

// vitest/valid_title.rs (simple reference)
declare_oxc_shared_lint!(
    ValidTitle,
    vitest,
    correctness,
    conditional_fix,
    shared_docs = crate::rules::shared::valid_title
);
```

This saves ~60 lines per additional plugin!
