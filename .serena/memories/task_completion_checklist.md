# Task Completion Checklist for Oxc

Before submitting any changes, complete these steps:

## 1. Format Code

```bash
just fmt
```

## 2. Run All Checks

```bash
just ready
```

This runs:

- `typos` - spell checking
- `just fmt` - formatting
- `just check` - cargo check
- `just test` - all tests
- `just lint` - clippy with deny warnings
- `just doc` - documentation build
- `just ast` - AST generation (if needed)

## 3. Crate-Specific Updates (if applicable)

### After oxc_ast changes:

```bash
just ast
```

### After oxc_minifier changes:

```bash
just minsize
```

### After oxc_parser changes:

```bash
just allocs
```

## 4. Review Snapshot Changes (if applicable)

```bash
cargo insta review
```

## 5. Verify No Uncommitted Changes

```bash
git status
git diff --exit-code --quiet
```

## Quick Single Command

```bash
just ready
```

This will fail if there are issues, ensuring quality before commit.
