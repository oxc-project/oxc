# Session Context

## User Prompts

### Prompt 1

Implement the following plan:

# Unified CI Change Detection

## Context

The CI uses three fragmented change detection mechanisms: native GitHub `paths:` filters, `dorny/paths-filter` action, and custom JS scripts with `cargo tree`. The `dorny/paths-filter` jobs have **hardcoded path lists that miss transitive dependencies** — e.g., the `minification` job only checks `oxc_minifier` and `oxc_codegen`, but `oxc_minifier` depends on `oxc_semantic`, `oxc_parser`, `oxc_ast`, etc. Changes to those ...

### Prompt 2

# Simplify: Code Review and Cleanup

Review all changed files for reuse, quality, and efficiency. Fix any issues found.

## Phase 1: Identify Changes

Run `git diff` (or `git diff HEAD` if there are staged changes) to see what changed. If there are no git changes, review the most recently modified files that the user mentioned or that you edited earlier in this conversation.

## Phase 2: Launch Three Review Agents in Parallel

Use the Agent tool to launch all three agents concurrently in a singl...

### Prompt 3

can we use a reusable action for check-changes

### Prompt 4

create pr

