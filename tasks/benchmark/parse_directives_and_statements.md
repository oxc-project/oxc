# `parse_directives_and_statements` Optimization Notes

Tracking CodSpeed results while iterating on targeted parser micro-optimizations.

## PR

- PR: #20921
- Branch: `c/03-31-perf_split_loop`

## Results

| Optimization | Change | CodSpeed result | Notes |
| --- | --- | --- | --- |
| 1 | Split top-level parsing path from non-top-level parsing path. | No measurable change. | CodSpeed comment: "Merging this PR will not alter performance." `48` untouched benchmarks, `8` skipped. Compared `f175eeb` vs `main` (`23db851`). |
| 2 | Split directive-prologue parsing from the steady-state statement loop. | No measurable change. | CodSpeed comment remained "Merging this PR will not alter performance." `48` untouched benchmarks, `8` skipped. Updated comment compared `ee62534` vs `main` (`23db851`). |
| 3 | Remove the directive `StringLiteral` clone by consuming the parsed statement. | Pending | Not pushed yet. |
| 4 | Pending | Pending | |
