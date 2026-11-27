---
name: pr-writer
description: Write or revise Oxc pull request titles and descriptions, including scoped titles, before-and-after examples, verification evidence, and playground links.
---

# Oxc pull request guide

## PR titles

Base the title on the final diff against the PR's base. For a stacked PR, compare it with its parent so the title describes only that PR. Before a PR exists, use the intended diff. Choose a type that matches the purpose of the change and write a concise subject describing the concrete behavior or operation.

Use a scoped Conventional Commit title: `type(scope): subject`. Use `!` only for a breaking change. See the [PR title check](../../../.github/workflows/pr.yml) for permitted types and scope syntax.

Prefer the narrowest scope that describes the change:

- A lint rule: `fix(linter/<plugin-name>/<rule-name>): ...`, for example `fix(linter/eslint/prefer-const): skip fixes when a declarator is reassigned`.
- A specific transform: `fix(transformer/<transform-name>): ...`, for example `fix(transformer/async-generator): preserve return values`.
- Shared infrastructure or several rules or transforms: use a common scope such as `linter/unicorn`, `linter`, or `transformer`.
- Independent subsystems: use comma-separated scopes when useful, such as `fix(semantic,linter): ...`. Supporting tests and incidental helper edits do not require broader scopes.

Use canonical public names such as `prefer-const`, not Rust filenames such as `prefer_const.rs`. If unsure about the type, scope, or wording, inspect recent `git log` subjects on the default branch for the affected paths. Follow this guide and the title check over patterns inferred from history.

When asked only for a title, return the title alone unless a material ambiguity needs explanation. A wording-only request does not authorize remote edits, commits, or rewriting history.

## PR descriptions

For behavior fixes and optimizations in any crate, lead with the problem and include an example in this shape:

### Example

Input:

```js
// self-contained code that demonstrates the problem
```

Before output:

```text
output or diagnostics produced at the parent commit
```

After output:

```text
output or diagnostics produced with the change
```

Use the appropriate code fence for each output. Capture both results by running the affected tool with the same input and options rather than writing them by hand. Explain the difference after the outputs. For parser and linter changes, show the relevant diagnostics or their absence; for size improvements, report byte counts with matching options.

Include a [playground](https://playground.oxc.rs/) link for the Example whenever the playground supports the affected behavior: the same input, with the options and enabled rules that match the command used for the outputs. Put the link next to the input it reproduces and label it so readers know what it shows (for example, "playground for this example"). Do not add separate links for examples quoted from linked issues. Use the repository's [oxc-playground-url skill](../oxc-playground-url/SKILL.md) to build and decode the link. If a playground reproduction is not possible, state the concrete limitation in the description instead of silently omitting the link. The playground runs the released version; it does not run the PR's code.

For runtime correctness fixes, make the example expose the bug when run: invoke the affected function and print the result or assert the expected behavior. Show a difference such as one run throwing while the other succeeds or different console output. A declaration that never gets called is not enough to demonstrate a runtime bug. Keep the example deterministic and self-contained. If you include the playground's execution comparison, list the steps to reproduce it; otherwise provide a local run command. Compare the input against output from both commits locally to demonstrate the fix, and report the actual values or errors. Playground comparison uses the deployed version and does not compare the parent commit with the PR.

Decode the shared link to check the saved input and options. Opening it is optional; open it only when the description depends on how the released version differs from the branch.

For a refactor, state what must stay identical and how it was compared, such as byte-identical output and unchanged conformance results between the parent commit and the change.
