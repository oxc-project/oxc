# Emscripten NAPI bindings

These scripts build the published NAPI packages for Cloudflare Workers. The
playground package is intentionally excluded because it has no npm release
workflow.

The Rust crates are compiled as `wasm32-unknown-emscripten` static libraries,
linked with Emnapi's single-threaded basic NAPI archive, and then linked by
Emscripten into a worker-specialized ES module and `.wasm` file.

## Local build

Install and activate Emscripten `5.0.3`, then run:

```sh
rustup target add wasm32-unknown-emscripten
pnpm build-emscripten
```

Run the direct Emnapi smoke tests and real workerd tests with:

```sh
for package in parser minify transform transform-react; do
  pnpm -C "napi/${package}" test-emscripten
  pnpm -C wasm/emscripten test "../../napi/${package}"
done
```

The generated artifacts are written to `target/emscripten/<package>`.

## Publishing

The NAPI release workflow builds and tests Emscripten alongside the native and
WASI targets. It publishes one companion package before each root package:

- `@oxc-parser/binding-wasm32-emscripten`
- `@oxc-minify/binding-wasm32-emscripten`
- `@oxc-transform/binding-wasm32-emscripten`
- `@oxc-transform-react/binding-wasm32-emscripten`

The companion is added to the root package's `optionalDependencies` at the same
version. Users continue importing the root package; Wrangler selects its
`workerd` conditional export.

Before the first automated release, each companion name must be published once
and configured for npm trusted publishing. Subsequent releases are handled by
`reusable_release_napi.yml`.
