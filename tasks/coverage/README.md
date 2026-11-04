# Coverage tests

Tools are tested against [test262], [babel] and [TypeScript] for conformance.

Clone the test repositories beforehand:

```bash
just submodules
```

## Development

```bash
# full run
cargo coverage

# run a single tool against its supported conformance suites
cargo coverage parser
cargo coverage transformer
cargo coverage minifier

# run in watch
cargo watch -x 'coverage minifier'

# filter for a file path
cargo watch -x 'coverage minifier --filter filter-file-path'

# find crash scene by turning off rayon and print out the test cases in serial
cargo coverage --debug

# Run after submodules are updated
UPDATE_SNAPSHOT=1 just c
```

<!-- Links -->

[test262]: https://github.com/tc39/test262
[babel]: https://github.com/babel/babel
[TypeScript]: https://github.com/microsoft/TypeScript
