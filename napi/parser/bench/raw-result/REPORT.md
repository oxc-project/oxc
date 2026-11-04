# Raw-result representation: local proof of concept

AI-assisted investigation of main at `62a8e7818e`, using matching locally built
parser bindings, generated deserializers and JS printer (0.157.0). The primary
checkout and external reproduction were preserved. Implementation:
[`eager.js`](../../src-js/raw-transfer/eager.js). Method, commands and caveats:
[README](README.md). [All samples and checks](results.json).

Historical five paired fresh-process repetitions, Apple M4 Pro, Node 26.13.0.
These runs discarded results during warmup, including the retained workload:

| Workload                           | Main median | POC median |          Change |
| ---------------------------------- | ----------: | ---------: | --------------: |
| Eager raw parsing, discard results |      1.441s |     1.036s | 28.1% less time |
| Parse + JS print + maps            |      2.966s |     2.498s | 15.8% less time |
| Retain all 32,267 results          |      2.020s |     2.169s |  7.4% more time |

Every raw/maps pair favored the POC. Retention pairs overlapped (three slower,
two faster); there is no consistent retained-AST benefit. Median peak RSS fell
1132 → 507 MiB for raw and 1155 → 559 MiB for mapped printing, while full retention
was similar at 4312 → 4269 MiB. Warmups, preloading and forced GC precede timing;
profiles and GC instrumentation are separate. Earlier heavily contended partial
runs were excluded, and all final individual samples/ranges are preserved.

The follow-up investigated private-field stamping and read-only data properties.
Neither removed the retained regression, so the runtime POC remains unchanged.
In separate five-repetition confirmations through the public implementation,
data properties took 2.809s versus main's 2.659s and the original POC's 2.709s;
literal getters with private-field stamping took 2.667s versus 2.587s and 2.631s.
A single literal-getter construction sample was faster, but neither trial
established a complete retained-workload improvement. Both passed the 13 focused tests
and 64 contract combinations plus async cases, with representation differences
explicitly checked.

The benchmark now matches warmup retention to the measured workload and releases
warmup owners before forced GC. Five counterbalanced repetitions of the original
POC measured 3.166s main versus 3.819s POC with discarded warmup (+20.6%), and
2.842s versus 2.869s with retained warmup (+1.0%; three slower, two faster pairs).
This corrects methodology, not runtime behavior. Matched V8 source and GC traces
support survival feedback affecting allocation-site pretenuring; a traced control
with that feature disabled still left a gap. The retained regression remains
unresolved. Background contention and visible rate changes limit comparisons;
absolute timings from different series must not be pooled. All completed samples,
including screening outliers, manifests, exact trial patches and diagnostic
summaries are in [the follow-up evidence](retention-results.json).

The evidence supports **old-space accessor pairs retaining getter closures and
their captured ASTs across minor GC**. In three fresh controlled repetitions,
all 128 wrappers died after two minor GCs, but all closure-backed payloads
survived and were promoted; major GC reclaimed them. Private-backed payloads
were reclaimed during minor GC. Matched V8 source confirms
[old-space accessor-pair allocation](https://github.com/nodejs/node/blob/v26.12.1/deps/v8/src/heap/factory-base.cc#L67)
and [remembered-set scanning](https://github.com/nodejs/node/blob/v26.12.1/deps/v8/src/heap/scavenger.cc#L2444). The whole corpus showed 1514.7 →
36.6 MB promoted, while allocation sampling was nearly unchanged at 3.785 →
3.776 GiB. Disabling adaptive pretenuring or Maglev preserved the gap. This is
strong causal evidence, without a custom V8 build attributing every byte.

The POC keeps private per-result data and shared own getters, including tokens.
It preserves Object.prototype, key order, accessor flags, read-only assignment
and normal field reads. **It changes detached getters, cross-result receiver
calls, ordinary proxies, inherited access and getter identity.** A result also
keeps private data after all accessors are deleted; main can release that data
at major GC while the emptied owner stays alive. Three fresh controls confirmed
this additional retention tradeoff. The tests and README make the differences
explicit. Helper closures, explicit accessors
and bound getters did not establish a comparable compatible improvement.
This is an experimental API tradeoff, not a merge-ready transparent replacement.

Full-corpus AST/comment/error/module parity, exact code/maps, ordered imports,
output reparse and printer fixed points passed. The 33,579,124 nodes and 214,521
imports match prior evidence. Generated output remains 217,840,905 bytes,
SHA-256 `c56c4b008d33671dc659dc7f48e677d98d96cdb789173e36acc8934e56818cc5`.
All 178,470 NAPI tests passed, as did focused Node 26.13 tests, parser conformance
with unchanged snapshots, generation, formatting and focused lint.
Those full-suite results remain applicable because the follow-up restored the
exact original runtime, native binding, generated deserializers and printer.
The follow-up reran focused tests and contract checks and exercised the updated
benchmark harness separately.

Baseline JSON/raw differences for metadata ranges and literal lone surrogates
were recorded separately. Local parent tests pass; the existing package file
whitelist still omits parent deserializers, confirmed by a package dry run.

Next: agree on an explicit experimental accessor contract, measure realistic
bounded retention and diverse/Unicode-heavy source across Node versions, and
consider a V8-side young-accessor-pair experiment that preserves closures. The
corpus is synthetic and ASCII-heavy. No full-build speedup or Rust grammar
hot-function ranking is established by these measurements.
