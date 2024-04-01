The sourcemap implement port from [rust-sourcemap](https://github.com/getsentry/rust-sourcemap), but has some different with it.

- Encode sourcemap at parallel, including quote `sourceContent` and encode token to `vlq` mappings.
- Avoid `Sourcemap` some methods overhead, like `SourceMap::tokens()`.