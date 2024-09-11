# Coverage

Tools are tested against [test262], [babel] and [TypeScript] for conformance.

Clone the test repositories beforehand:

```bash
just submodules
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

# find crash scene by turning off rayon and print out the test cases in serial
cargo coverage -- --debug

# Run after submodules are updated
UPDATE_SNAPSHOT=1 just c
```

<!-- Links -->

[test262]: https://github.com/tc39/test262
[babel]: https://github.com/babel/babel
[TypeScript]: https://github.com/microsoft/TypeScript
