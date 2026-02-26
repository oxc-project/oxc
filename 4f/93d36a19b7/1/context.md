# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Fix: no-fallthrough rule misses fallthrough FROM default when not last

## Context

GitHub issue: https://github.com/oxc-project/oxc/issues/11340

When `default` is not the last case in a switch statement, oxlint fails to detect fallthrough FROM the default case to subsequent cases. A previous fix (#12927) corrected `cfg_ids` ordering but didn't address this issue.

**Example - should report error but doesn't:**
```js
switch (x) {
  default:
    foo();   // falls...

### Prompt 2

create pr on top of main

