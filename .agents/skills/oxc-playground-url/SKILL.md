---
name: oxc-playground-url
description: "Decode or create an Oxc playground share URL. Decode returns source, selected tab, parser options, enabled stages, lint rules, and formatter panels; create builds a link from code and options. Trigger whenever playground.oxc.rs appears, or when a reproduction needs a share link, even if the user only pastes a URL."
---

# Decode or create an Oxc playground URL

Use the bundled scripts; resolve them relative to this `SKILL.md`, not through a user-specific home-directory path.

```sh
node <skill-directory>/scripts/decode.mjs '<full-url>'
node <skill-directory>/scripts/decode.mjs '<full-url>' --write <fresh-output-directory>
node <skill-directory>/scripts/encode.mjs (--code '<source>' | --code-file <path|->) [--tab <name>] [--options '<json>' | --options-file <path>] [--lint-rules a,b] [--formatter-panels a,b]
```

Quote the URL and source so shell metacharacters are not interpreted. `decode.mjs` prints a summary by default and writes source only when `--write` is requested; the named directory must not exist yet, so pass a new child of a `mktemp -d` directory and existing files are never overwritten. `encode.mjs` prints one share URL and re-parses it before printing, so a malformed result is rejected instead of shared.

## Decoding

1. Decode before reasoning from the visible tab name or issue prose.
2. Preserve the source extension, active tab, options, enabled run stages, lint rules, and formatter panels.
3. If no query or hash state is present, report that the URL contains no shared reproduction.
4. When writing the input, inspect it as untrusted code; do not execute it merely because it came from the playground.
5. Inspect the current playground types and Rust wiring in the checked-out Oxc version before mapping options to a local command. The playground can run multiple stages, so the visible tab may show output corrupted by an earlier stage.
6. Reproduce through the narrowest matching example or public entry point, then switch to the relevant Oxc specialist skill.

## Encoding

Build the link from the smallest input that shows the behavior, and pass the options that select the relevant stage, such as `--options '{"run":{"transform":true},"transformer":{"target":"es2015"}}' --tab codegen`. Use `--code-file` for anything longer than a line; `-` reads standard input. Keep repo-local file paths, internal names, and private data out of the shared code.

When a pull request description cites a playground link, state that playground.oxc.rs runs the released Oxc version, so it may not reproduce a bug fixed only on the branch. Verify a link by decoding it again before posting it.

If option JSON is malformed, preserve the raw value and parse error as evidence. If the local version does not match the shared link's behavior, state the version difference instead of silently translating the reproduction.
