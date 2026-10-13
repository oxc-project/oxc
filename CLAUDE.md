# Oxc

Build, test, lint and format commands, as defined in `justfile` and `package.json`:

- Type-check the Rust workspace: `just check`
- Run the Rust tests: `just test` (extra arguments go to `cargo test`, e.g. `just test -p oxc_parser`)
- Lint the Rust code with warnings denied: `just lint`
- Format Rust, JavaScript and other files: `just fmt`
- Build the Node.js packages: `pnpm build`
- Run the Node.js package tests: `pnpm test`
- Run the full local check sequence before pushing: `just ready`
