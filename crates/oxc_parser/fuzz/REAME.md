# Fuzzer

# Installation

```bash
cargo install cargo-fuzz
```

Run fuzzer for the parser, for 15mins.

```bash
cd crates/oxc_parser/fuzz
cargo fuzz run parser -- -only_ascii=1 -max_total_time=900
```
