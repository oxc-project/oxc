---
name: pr-title
description: Write or revise Oxc PR titles using repository history and precise scopes.
---

# PR Titles

## Describe the final change

Inspect the PR's actual diff against its base. For a stacked PR, use its parent branch so the title describes only that PR. For work not yet in a PR, use the intended changes and diff. Rewrite stale titles around the final behavior and scope.

Choose the type from the change's purpose and the repository's vocabulary. Describe the concrete behavior, fix, or operation in a concise subject. Avoid vague subjects such as "fix bug" or "improve handling." Match observed length conventions without imposing an arbitrary universal character limit.

## Oxc conventions

PR titles use Conventional Commits with a required scope: `type(scope): subject`. Prefer a scope identifying the affected rule or transform over a broad subsystem scope when the change is specific to one:

- A lint rule: `fix(linter/<plugin-name>/<rule-name>): ...`, for example `fix(linter/eslint/prefer-const): skip fixes when a declarator is reassigned`.
- A particular transform: `fix(transformer/<transform-name>): ...`, for example `fix(transformer/async-generator): preserve return values`.

Apply the same scope precision to other appropriate types, such as `feat` or `refactor`. Derive canonical plugin, rule, and transform names from the code, documentation, and relevant history. Use public hyphenated names such as `prefer-const` rather than mechanically copying Rust filenames such as `prefer_const.rs`. Do not invent a transform scope from an unrelated helper or file name.

For shared infrastructure or changes spanning several rules or transforms, use the narrowest shared scope that describes the change, such as `linter/unicorn`, `linter`, or `transformer`. For changes spanning independent subsystems, follow Oxc's comma-separated scope convention where useful, such as `fix(semantic,linter): ...`. Supporting tests or incidental helper edits do not by themselves require broader scopes.

Use `!` only for an actual breaking change. Check `.github/workflows/pr.yml` for permitted types and scope syntax.

## Deliver the title

When asked only for a title, return the chosen line without extra commentary unless a material ambiguity needs explanation. Apply it when the user has requested creating or updating the PR. A wording-only request does not authorize remote edits, commits, or rewriting existing history.

## Check history when uncertain

If uncertain about the title's type, scope, or wording, inspect recent `git log` subjects on the default branch, focusing on the affected paths. Ignore merge boilerplate and trailing PR numbers such as `(#12345)`. Follow repository instructions and `.github/workflows/pr.yml` over patterns inferred from history.
