# Coverage

The parser is tested against [test262], [babel] and TypeScript for conformance.

Note: tests against regexp are disabled for now.

Clone the test files beforehand

```bash
git submodule update --recursive
```

## Development

```bash
# full run
cargo coverage
cargo coverage js # for test262
cargo coverage babel # for babel
cargo coverage ts # for typescript

# run in watch
cargo watch -x 'coverage js'

# filter for a file path
cargo watch -x 'coverage js --filter filter-file-path'
```

<!-- Links -->
[test262]: https://github.com/tc39/test262
[babel]: https://github.com/babel/babel
