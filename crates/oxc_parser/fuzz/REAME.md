# Fuzzer

# Installation

```bash
cargo binstall cargo-fuzz
```

Run fuzzer for the parser, for 15 minutes.

```bash
cd crates/oxc_parser/fuzz
cargo fuzz run parser -- -only_ascii=1 -max_total_time=900
```
