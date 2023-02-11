# Coverage

The parser is tested against [test262] for conformance.

Note: tests against regexp are disabled for now.

Clone the test files beforehand

```bash
git submodule update --init --recursive --remote --merge
```

## Development

```bash
# full run
cargo coverage

# run in watch
cargo watch -x 'coverage js'

# filter for a file path
cargo watch -x 'coverage js --filter filter-file-path'
```

<!-- Links -->
[test262]: https://github.com/tc39/test262
