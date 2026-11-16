# Pull request guidance

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

Include a [playground](https://playground.oxc.rs/) link for the Example whenever the playground supports the affected behavior: the same input, with the options and enabled rules that match the command used for the outputs. Put the link next to the input it reproduces and label it so readers know what it shows (for example, "playground for this example"). Do not add separate links for examples quoted from linked issues. Use the repository's [oxc-playground-url skill](skills/oxc-playground-url/SKILL.md) to build and decode the link. If a playground reproduction is not possible, state the concrete limitation in the description instead of silently omitting the link. The playground runs the released version; it does not run the PR's code.

For runtime correctness fixes, make the example expose the bug when run: invoke the affected function and print the result or assert the expected behavior. Show a difference such as one run throwing while the other succeeds or different console output. A declaration that never gets called is not enough to demonstrate a runtime bug. Keep the example deterministic and self-contained. If you include the playground's execution comparison, list the steps to reproduce it; otherwise provide a local run command. Compare the input against output from both commits locally to demonstrate the fix, and report the actual values or errors. Playground comparison uses the deployed version and does not compare the parent commit with the PR.

Decode the shared link to check the saved input and options. Opening it is optional; open it only when the description depends on how the released version differs from the branch.

For a refactor, state what must stay identical and how it was compared, such as byte-identical output and unchanged conformance results between the parent commit and the change.
