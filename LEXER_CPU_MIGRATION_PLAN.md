# Lexer CPU migration plan

How to move the rest of `oxc_lexer`'s pipeline onto the CPU primitives in
`crates/oxc_lexer/src/cpu/` (in `pipeline/cpu/` until step G1).

Function lists come from a call graph built before Phase A by matching function names, so each step
re-checks its list as it goes.

## Goal

- No `#[cfg(all(target_arch = ..., target_feature = ...))]` left in the pipeline. Every CPU choice goes
  through `cpu_select!`.
- On x86_64, the whole pipeline is compiled twice below the one runtime switch in `Lexer::lex_raw`. One copy
  is for AVX2 + BMI2 + POPCNT, and the other is generic.
- Single-copy code (rule 2) is the exception. It's compiled once.

## Already done

- The primitives: the `Cpu` trait, `DynCpu`, the `Avx2` / `Neon` / `Generic` types, `BASELINE`, `cpu_select!`
  and `cpu_versions!`.
- The runtime switch `lex_raw_impl`, and the stages `classify`, `misc_pre`, `misc_post`, `coalesce` and
  `compress`.
- `bitmap::bm_any`.
- A fix to `cpu_versions!`, so `#[inline]` functions don't bloat the generic version.
  - Without it, rustc's MIR inliner inlined `__generic` into the outer function, which made an `#[inline]`
    function act as if it was `#[inline(always)]` in its generic version.
  - Now `__generic` gets `#[target_feature(enable = "sse2")]` when SSE2 is enabled for the whole crate, which
    stops that. The macro's docs explain it.
  - With step B3 applied, the fix cut the generic code from 42,972 instructions back to 36,692, where B2 had
    36,676.
- Phases A to E. Their sections below say what each step did.
  Some steps differ from the original plan.
  - B9 also did step D5, converting `cook_short` to `cpu_select!` (rule 9).
  - B12 gave `unicode_ws_len_at_unchecked` `#[inline(always)]` instead of `cpu_versions` (rule 1).
  - D1 added `array_map` to `find/avx2.rs`, because `std`'s `<[T; N]>::map` broke `any_of`.
  - D7 also made `veq` and `mm` in `chunk.rs` safe functions.
- The audit at the end of "Verification for each commit". "Audit after Phase E" has its results.
- Fixes after the audit, which restore the inlining from before the migration ("Observed drift").
- Steps G1 and G2, in one commit.
- Still to do: Phase G from G3, and Phase F.
- "Observed drift" below records how the codegen has changed so far.

## Rules

1. Each function reachable from `lex_raw_impl` keeps its current inline attribute, and that attribute decides
   how it's migrated.
   - An `#[inline(always)]` function stays `#[inline(always)]`. It takes `cpu` if anything it calls needs one.
   - Any other function gets `#[apply(cpu_versions)]`. That covers `#[inline]`, `#[inline(never)]` and no
     attribute. The function keeps its inline attribute, which the macro puts on the entry points.
   - Inline attributes don't change, except in steps A1, A2, B12 and E1, and the fixes after the
     audit.
   - Fixes follow the assembly. A function which was inlined before the migration, but is called
     out of line now, gets `#[inline(always)]` instead of `cpu_versions`. B12 was the first. It
     keeps `cpu` if anything it calls needs one.
2. Single-copy code is compiled once. It takes no `cpu`.
   - A single-copy function which calls anything that needs a CPU type
     starts with `let cpu = BASELINE;`, and passes `cpu` on.
   - Single-copy code is Level 3 of `disambiguate` (`disambiguate/context/`), `#[cold]` functions, and
     functions reached only from those.
   - Outside Level 3, the functions reached only from single-copy code are `Prev::is_member_dot`,
     `Tokens::op_len`, `Tokens::template_head`, `Lanes::hex4`, `decode_char_at` and `ident_start_run_end`.
   - Entry points into single-copy code are `#[inline(never)]`, so they can't be inlined into the copies of
     their callers.
   - Tests pass `BASELINE` directly to the functions they call.
3. The CPU-specific versions which a `cpu_select!` picks between don't get `cpu_versions`.
   - An AVX2 version gets `#[target_feature(enable = "avx2,bmi2,popcnt")]` and keeps its inline attribute,
     like `bm_any_avx2`. If it's `#[inline(always)]`, it stays that way, like `load256` in `chunk.rs`.
     - Intrinsics which don't access memory are safe to call inside a `#[target_feature]` function,
       so they need no `unsafe` block there. Enabling AVX2 for the whole crate doesn't make them
       safe anywhere else.
   - A generic version stays as it is. Only the `Neon` and `Generic` arms call it, so it's always compiled for
     the right CPU.
   - If a generic version calls code that needs a CPU type, it takes `cpu: impl Cpu`, and rule 1 then applies
     to it.
4. Any function which uses SIMD primitives (e.g. `_mm*` functions) takes the matching CPU type, e.g.
   `cpu: Avx2` or `_cpu: Avx2`. Phase D made the cfg'd code meet this.
   - Using them inside a `cpu_select!` arm which binds the matching type also counts, as in `coalesce`'s
     `prefetch`.
5. `cpu` is always passed as an argument. It's never stored in `Tokens` or any other struct.
6. A function names its parameter `_cpu` while nothing it calls uses it yet.
7. A step versions a function only after all its callers have a CPU type. So no step needs to pass a temporary
   `BASELINE`, and the step order below follows the call graph.
8. Each step is one commit.
9. No function is both cfg'd on target features and `cpu_versions`.
   - If a step would give a cfg'd function a CPU type, the same step converts it to a `cpu_select!`
     dispatcher, as Phase D does. B9 did this for `cook_short`.

## Expected asm

The Neon version on aarch64 and the AVX2 version on x86_64 run on nearly every machine,
so their code should change as little as possible.
It can't stay identical, because LLVM now does inlining
which rustc's MIR inliner did before (note 3).
The generic version on x86_64 only runs on CPUs without AVX2, so its speed matters less.
What does matter more there is that it doesn't bloat binaries.

Compare each function before the step with its copies after it.

| Build                  | Each old function matches     | Also expected                |
| ---------------------- | ----------------------------- | ---------------------------- |
| aarch64                | its `__neon` copy, nearly     | Nothing else                 |
| x86_64 with AVX2 flags | its `__avx2` copy, nearly     | `__generic` copies (note 1)  |
| x86_64                 | its `__generic` copy, roughly | New `__avx2` copies (note 2) |

1. Single-copy code calls the `__generic` copies in builds with AVX2 flags, because `BASELINE` is
   always `Generic` on x86_64. Before Phase D, they were the same code as the `__avx2` copies
   under global AVX2 flags. Since Phase D, they use the generic versions of the CPU-specific code.
2. In the x86_64 build, count the instructions of the generic code, which is everything outside the `__avx2`
   copies. Small changes are fine. A big jump means generic code is being inlined or duplicated more.
3. Differences which are expected, and judged at the end of the migration:
   - The `#[target_feature]` entry points stop rustc's MIR inliner inlining versioned functions, so
     LLVM does all of that inlining. Register allocation and block layout shift in big functions, in
     all builds.
   - Inlining decisions can move. A function called from both copies of a caller may stop being
     inlined, until a later step versions it. A function whose callers get their own copies may
     start being inlined, or the reverse.
   - Standard library functions can't be versioned, e.g. `str::trim_matches`, which `str::trim`
     calls. So some of this is permanent.
   - A closure passed to a `std` function is compiled without AVX2 if LLVM doesn't inline that
     function. Then the intrinsics in the closure are called out of line. So AVX2 code doesn't
     pass closures to `std`. `find/avx2.rs` has `array_map` for this (D1).
   - If a function which was inlined before goes out of line, it gets `#[inline(always)]`
     instead (rule 1).
4. Steps with intended codegen changes:
   - A2, in all builds.
   - B9 for `cook_short`, and Phase D, on x86_64 only. In x86_64 builds, the `__avx2` copies start
     using the AVX2 versions. In builds with AVX2 flags, the `__generic` copies and single-copy code
     start using the generic versions. That's what the runtime-detected build will do.
   - Phase F.

## Observed drift

How the codegen has changed so far, compared with A2, the last commit before B1.
Some of it was fixed after the audit ("Fixes after the audit" below).
The rest is judged at the end of the migration (note 3 of "Expected asm").

Totals after each phase:

| Code                       | A2     | After B | After C | After D | After E | After fixes |
| -------------------------- | ------ | ------- | ------- | ------- | ------- | ----------- |
| x86_64 build, generic code | 36,523 | 36,444  | 36,455  | 36,455  | 36,455  | 36,546      |
| x86_64 build, AVX2 copies  | 3,586  | 16,360  | 23,131  | 24,151  | 24,565  | 24,726      |
| aarch64, in total          | 35,511 | 35,199  | 34,534  | 34,534  | 34,534  | 35,356      |

Phase B:

- `cook` is now inlined into `push_string`, `push_template` and `push_atom`.
  They're called out of line from `carve`, `carve_jsx` and `compress` instead of `cook`,
  so the number of calls per literal is the same. On aarch64 since B7 and B8, on x86_64 since B9.
  Fixed after the audit.
- On x86_64, `scan_number` and `scan_fraction_exponent` are now inlined into `coalesce` (B11),
  as they already were on aarch64.
- Register allocation and block layout changed in `carve` and `carve_jsx`, in all builds.
- On aarch64, `cook_decode::<false, true>` is 52 instructions longer (B9).
- `str::trim_matches` is called out of line from `classify` (B3). This is permanent.

Phase C:

- On x86_64, the AVX2 copy of `type_args_at` has only one caller,
  so it's inlined into the AVX2 copy of `gt_run_split` (C5).
  That copy grows from about 950 instructions to about 3,000.
- `Tokens::next_start`, `ident_kw` and `ident_is` are now called out of line (C7).
  Their callers are in Levels 2 and 3 only. `not_operator_position` doesn't call them.
  Fixed after the audit.
- In Level 3, `Walk::step` no longer inlines `Tokens::next_start`, `Walk::asi`,
  `Walk::end_region_for` or `Walk::continues_after_type_break` (C7).
  It shrinks from about 1,950 instructions to about 1,140.
  Fixed after the audit, apart from one call to `Tokens::word_kw` on aarch64.
- On x86_64, each copy of `lex_raw_impl` now inlines `DisambiguateState::begin`,
  with `Walks::restart` and `Brackets::begin` (C4, C8). This runs once per lex.
- On x86_64, `resolve_lists` and `resolve_groups` call `HashMap::insert` out of line,
  instead of `Closers::set` (C9). It's a cold path, and permanent, because `std` can't be versioned.
- `line_terminator_after` is now inlined into `line_break_in`, which is called out of line instead
  (C11). Fixed after the audit.

Phase D:

- aarch64 code is unchanged.
- With AVX2 flags, the AVX2 copies have the same instructions as before.
  Registers and instruction order are shuffled in `carve`, `carve_jsx` and `delim_memo_opener` (D1).
  `scan_block_comment_avx2` has 1 `mov` fewer (D2).
- In the x86_64 build, the AVX2 copies now use the AVX2 versions, as intended (note 4).
  The AVX2 copies of `carve`, `carve_jsx`, `coalesce`, `push_string`, `push_template` and
  `push_atom` now have the same instructions as in the build with AVX2 flags.
  Their inlining of `Tokens::word_kw` and `Tokens::ident_kw` now matches it too (D3).

Phase E:

- aarch64 code is unchanged.
- On x86_64, the AVX2 copy of `Lexer::ensure` has only one caller,
  so it's now inlined into the AVX2 copy of `lex_raw_impl` (E5).
  That adds about 410 instructions, and it runs once per lex.
  `Lexer::lex` and the generic copy of `lex_raw_impl` call the generic copy out of line, as before.

Fixes after the audit:

- Only a problem seen in the assembly is fixed.
  A function which was inlined at A2, but is called out of line now in the x86_64 build, gets
  `#[inline(always)]` (rule 1).
  The AVX2 copies are compared with A2's build with AVX2 flags, and everything else with A2's
  x86_64 build.
- In each case, a wrapper had gone out of line, with the function it wraps inlined into it.
  So the number of calls was the same, but the call was in a different place.
  Each fix restored A2's shape in all 3 builds.
  - `Tokens::ident_is` and `Tokens::ident_kw` are inlined again, and `walk::ident_is` is called
    out of line again (C7).
  - `line_break_in` is inlined again, and `line_terminator_after` is called out of line again
    (C11).
  - `push_string`, `push_template` and `push_atom` are inlined again, and `cook` is called out of
    line again (B7 to B9).
- In Level 3, LLVM now rates the call sites in `Walk::step` as cold, so it inlines nothing there
  which costs more than 45 (C7). The functions which it no longer inlined get `#[inline(always)]`.
  - `Tokens::next_start` and `Tokens::base_kind`. They need no CPU type, so they lose their `cpu`
    parameter, as in B12.
  - `Walk::asi`, `Walk::continues_after_type_break` and `Walk::end_region_for`, and the
    `Walk::pop` and `Walk::pop_concise` which `asi` calls.
- `Walks::run_rules` also got `#[inline(always)]`, as an exception, so its `true` always folds.
  It was already inlined.

Not fixed:

- On aarch64, `Walk::step` calls `Tokens::word_kw` out of line once, inside the inlined
  `Walk::asi`. `#[inline(always)]` would also inline it at the x86_64 call sites, which were out
  of line at A2 too.
- `std` code which both copies of its caller call. LLVM doesn't inline it into either copy.
  - Real builds are the same. At `opt-level = 3`, each crate compiles its own copy of a generic
    `std` function, so other crates' calls don't add callers to `oxc_lexer`'s copy, even with fat
    LTO.
  - `HashMap::insert`, called from `resolve_lists` and `resolve_groups` on x86_64 (C9).
    - `#[inline(never)]` on the versioned `Closers::set` doesn't help. `HashMap::insert` then
      has 2 callers and stays out of line, so each insert makes 2 calls instead of 1.
    - Making `Closers::set` single-copy code with `#[inline(never)]`, and no CPU type, restores
      A2's shape in all 3 builds. Tried, but not done, because it's neither cold nor Level 3.
    - Using `hashbrown::HashMap` directly, with its `inline-more` feature, doesn't help.
      `std` builds `hashbrown` without `inline-more`, but `insert` stays out of line with it too.
  - `str::trim_matches`, called from `classify` (B3). It only runs for comments which start with
    `@__KEY__`, `#__KEY__` or a non-ASCII byte.
- Functions which are inlined now, but weren't before.
  - `type_args_at` into the AVX2 copy of `gt_run_split` (C5), if the bigger `gt_run_split` costs
    speed.
  - `DisambiguateState::begin` (C4) and `Lexer::ensure` (E5) into the copies of `lex_raw_impl`.
  - `scan_number` and `scan_fraction_exponent` into `coalesce` on x86_64 (B11).

Benchmarks so far:

- aarch64, A2 against B12, before B12 changed to `#[inline(always)]`: nothing beyond layout noise.
  Default builds were 1.3% to 2.7% slower,
  but builds with all functions and loops aligned to 64 bytes were within 0.2%.

When to judge it:

- aarch64 now.
  - Its build is final, after the fixes after the audit.
  - The last benchmark was at B12, before Phase C.
- x86_64 after Phase F, by comparing the AVX2 copies in the runtime-detected build
  with the old build with AVX2 flags.
  - Since Phase D, the AVX2 copies in the x86_64 build use the AVX2 versions of the CPU-specific
    code. After the fixes, each AVX2 function has the same instructions as in the build with AVX2
    flags, except `compress_blocks_avx2`, which differed at A2 too.
    With AVX2 flags, `lex_raw_impl`'s AVX2 copy is inlined into `lex_raw`.
  - With AVX2 flags, `std` code is compiled for AVX2 too. In the runtime-detected build, it isn't.
  - This Mac can't time x86_64. The CI lexer benchmark runs on x86_64.
    After Phase F, `just bench-lexer` on x86_64 runs the runtime-detected build.

## Phase A. Single-copy boundaries (done)

- **A1.** `#[inline(never)]` on the cold functions which lack it.
  - `lex_html_open_comment`, `lex_html_close_comment` and `compress::common::invalid_diags`.
  - No codegen change expected. All three are out of line in all 3 builds already.
- **A2.** `#[inline(never)]` on the Level 3 entry points, `context::scan::{after, before, angles_before}`.
  - Intended codegen change in all 3 builds. Today `before` and `angles_before` are inlined into their
    callers, along with `local_walk`. `after` is out of line already.

## Phase B. Carve and the code it calls (done)

- **B1.** The carve entry.
  - `carve` (`#[inline(never)]`), `carve_js` and `carve_jsx` get `cpu_versions`.
  - `lex_raw_impl` passes `cpu` to `carve`.
- **B2.** `carve/common.rs`.
  - `lex_string`, `lex_template_segment`, `lex_slash` and `lex_regex` take `cpu`.
- **B3.** `carve/comments.rs`.
  - `lex_line_comment`, `lex_block_comment`, `html_open_comment_at` and `html_close_comment_at` take `cpu`.
  - `cpu_versions` on `html_close_at_line_start`, `classify_exact`, `comment_body`,
    `contains_license_or_preserve`, `classify`, `is_property_key_annotation`, `is_coverage_ignore_file` and
    its nested `starts_with_directive`, `has_nl_cr`, `push_comment_record` and `is_lex_ws`.
  - The cold `lex_html_open_comment` and `lex_html_close_comment` use `BASELINE`.
- **B4.** `carve/jsx/angle_brackets.rs`.
  - `cpu_versions` on `jsx_over_type_params` and `ts_angle_verdict`.
  - It comes before `names.rs` because `ts_angle_verdict` calls `jsx_skip_trivia_fast`.
- **B5.** `carve/jsx/names.rs`.
  - `jsx_skip_trivia_fast` takes `cpu`.
  - `cpu_versions` on `jsx_names_equal_fast`, `is_jsx_name_byte_at`, `jsx_name_continues_after`,
    `jsx_skip_trivia`, `jsx_names_equal`, `jsx_name_next` and `jsx_name_end`.
- **B6.** `find_line_terminator` in `carve/common.rs`.
  - It comes after `names.rs` because `jsx_skip_trivia` calls it.
  - The cold HTML comment functions use `BASELINE`.
- **B7.** `Lanes` string methods.
  - `cpu_versions` on `push_string`, `push_template`, `push_string_raw` and `push_regex_flags`.
- **B8.** `Lanes` atom and number methods.
  - `cpu_versions` on `push_atom` and `push_number`.
  - In `compress`, `emit_value` takes `cpu`, and `lanes_post_avx2` passes its `Avx2`.
  - `lanes_post_generic` takes `cpu: impl Cpu` and gets `cpu_versions` (rule 3). `lanes_post` passes it the
    CPU type from its `Neon` and `Generic` arms.
- **B9.** `Lanes` cooking.
  - `cpu_versions` on `cook`, `cook_short_dispatch`, `cook_decode`, `push_cp` and `emit_oct`.
  - `cook_short` calls `cook_decode`, so it's converted to `cpu_select!` here, which was step D5
    (rule 9).
    - `cook_short` becomes an `#[inline(always)]` dispatcher.
    - `cook_short_avx2` gets `#[target_feature]` and takes `cpu: Avx2`, which it passes to
      `cook_decode`.
    - `cook_short_generic` gets `cpu_versions`.
- **B10.** `Lanes` number parsing.
  - `cpu_versions` on `parse_number`, `radix_digit` and `validate_number`.
- **B11.** `scan/common.rs`.
  - `cpu_versions` on `scan_quoted`, `scan_regex`, `scan_tmpl_text`, `scan_ident_esc`, `scan_number` and
    `scan_fraction_exponent`.
  - `misc_pre_impl` takes `cpu` from `misc_pre`, and `coalesce`'s `glue_number` passes its own.
- **B12.** `bytes::unicode_ws_len_at_unchecked` gets `#[inline(always)]`.
  - It only wraps the `#[inline(always)]` `unicode_ws_len`, so it needs no CPU type.
  - With `cpu_versions`, x86_64 called it out of line from `misc_pre` for every non-ASCII byte.

## Phase C. Disambiguate Levels 1 and 2, and shared code (done)

In each step, the Level 3 functions which call the functions it versions
start with `let cpu = BASELINE;` if they don't yet (rule 2).
26 Level 3 functions ended up with it.
So did `Tokens::template_head` and `Prev::is_member_dot`, which only Level 3 calls (rule 2).

- **C1.** `operator/mod.rs`.
  - `cpu_versions` on `not_operator_position`, `ends_decl` and `paren_close_is_regex`.
- **C2.** `type_context/tsx.rs`.
  - `cpu_versions` on `jsx_over_generic`.
- **C3.** `type_context/runs.rs`.
  - `cpu_versions` on `gt_run_split`, `run_shortcut`, `list_in_any_context`, `type_reference_head` and
    `return_type_colon`.
- **C4.** `DisambiguateState`.
  - `cpu_versions` on `begin` (called from `lex_raw_impl`) and `restart` (called from `coalesce`).
  - It comes before `walk.rs` and `closers.rs` because both methods call into them.
  - Both also call Level 3's `Walks::restart`, which isn't `#[inline(never)]`.
    It's a small reset, so it's inlined into both copies.
- **C5.** `type_context/type_list.rs`.
  - `cpu_versions` on `gt_follower`, `type_list_legal`, `keyword_type`, `type_args_at` and
    `list_is_type_args`.
  - It comes before `Tokens`, `closers.rs` and `type_context/bytes.rs` because it calls all three.
- **C6.** `type_context/bytes.rs`.
  - `cpu_versions` on `lt_run_split` (called from `coalesce`), `arrow_after_params`, `skip_trivia_fwd`,
    `list_closer`, `group_closer`, `past_raw`, `resolve_lists`, `kill`, `resolve_groups`, `skip_raw_literal`,
    `past_raw_string` and `raw_template_end`.
  - `skip_trivia_nl` takes `cpu`.
- **C7.** `Tokens` methods.
  - `cpu_versions` on `base_kind`, `word_kw`, `ident_kw`, `next_start`, `next_sig`, `peek`, `prev_sig`,
    `prev_token`, `property_name`, `member_dot`, `numeric_dot`, `run_len`, `line_break_between`, `ident_is`
    and `match_delim_back`.
  - `Tokens::op_len` calls nothing versioned, so it's unchanged.
- **C8.** `common/walk.rs`.
  - `cpu_versions` on the free `prev_sig` (called from `carve_jsx`), `Brackets::begin`, `Brackets::word`,
    `bracket_word`, `match_delim_back`, `delim_memo_opener`, `delim_slot` and `ident_is`.
  - It comes after `Tokens` because `Tokens::prev_sig` calls the free `prev_sig`.
- **C9.** `common/closers.rs`.
  - `cpu_versions` on `Resolved::closer`, `Closers::clear`, `Closers::get`, `Closers::set` and
    `Closers::set_args`.
- **C10.** `keywords::kw_match_at`.
- **C11.** The remaining functions in `pipeline/bytes.rs`.
  - `cpu_versions` on `unicode_ws_len_at`, `line_terminator_after`, `line_break_in` and `block_comment_end`.
  - Only `disambiguate` calls them, except `line_break_in`, which carve's `ts_angle_verdict` also
    calls. That's fine, because B4 already versioned `ts_angle_verdict`.

## Phase D. CPU-specific code to `cpu_select!` (done)

Each step replaces a cfg'd pair of functions with an `#[inline(always)]` dispatcher which takes
`cpu: impl Cpu` and uses `cpu_select!`.

- The AVX2 versions follow rules 3 and 4.
- The `avx2` module's cfg becomes `target_arch = "x86_64"` alone.
- The `Neon` arm calls the generic version, unless there's a Neon one.

None of the cfg'd functions left calls anything which needs a CPU type. The lexer functions they
call are all `#[inline(always)]`, and the rest come from `std` or `memchr`. So no Phase C step has
to bring a Phase D step forward under rule 9.

Steps:

- **D1.** `find/`.
  - `find` and `bracket_bits` become dispatchers in `find/mod.rs`.
  - `find_avx2` and `bracket_bits_avx2` get `#[target_feature]` and take `cpu: Avx2`, which they
    pass to `any_of`.
  - `any_of` takes `_cpu: Avx2` too, but stays `#[inline(always)]` without `#[target_feature]`. The
    two can't be combined.
  - `find` is called from `carve_js`, `carve_jsx`, `find_line_terminator`, `scan_quoted`,
    `scan_regex` and `scan_tmpl_text`. The last four rename `_cpu` to `cpu`.
    `bracket_bits` is called from `bracket_word`, which renames its `_cpu` too.
  - `find_avx2` is `#[inline]`, so only LLVM can inline it, but it's still inlined at every call
    site in both x86_64 builds.
  - `any_of` now uses an `#[inline(always)]` `array_map` instead of `<[T; N]>::map`.
    - `map` is `std` code, which is compiled without AVX2.
    - In the x86_64 build, LLVM didn't inline `map` for 6 needles into the AVX2 copies of
      `carve_jsx` and `Brackets::word`.
    - So for each needle, every 32-byte block called `_mm256_set1_epi8` and `_mm256_cmpeq_epi8`
      out of line.
- **D2.** `scan/`.
  - `scan_block_comment` and `scan_line_comment` become dispatchers.
  - `scan_block_comment_avx2` and `scan_line_comment_avx2` get `#[target_feature]` and take `_cpu: Avx2`.
  - Their callers `lex_line_comment`, `lex_block_comment` and `jsx_skip_trivia` already have `cpu`.
- **D3.** `keywords::text_matches`.
  - All 3 versions stay `#[inline(always)]`. The x86_64 one takes `_cpu: Avx2`, and the Neon one takes
    `_cpu: Neon`.
  - The x86_64 one only uses SSE2, as before.
  - `kw_match` takes `cpu`. `coalesce`'s `kw_verify_batch` and `kw_match_word`, and `kw_match_at`,
    pass theirs, renaming `_cpu` to `cpu`. Tests pass `BASELINE`.
- **D4.** `lanes.rs` `span_has_bs` and `span_has_bs_or_cr`.
  - The AVX2 versions get `#[target_feature]` and take `_cpu: Avx2`.
  - Their only caller is `cook`, which already has `cpu`.
- **D5.** Done in B9.
- **D6.** `carve/jsx/hyphens.rs` `jsx_glue_hyphens`.
  - Both versions stay `#[inline(always)]`. The AVX2 one takes `_cpu: Avx2`.
  - Its only caller is `carve_jsx`, which already has `cpu`.
- **D7.** The `chunk.rs` wrappers `load256`, `veq` and `mm` take `_cpu: Avx2`.
  - Their callers pass their `Avx2`. The callers are `classify_avx2` and the AVX2 versions from D1, D2 and D6.
    `any_of`, the 2 `scan_*_avx2` and `jsx_glue_hyphens_avx2` rename `_cpu` to `cpu`.
  - `veq` and `mm` become safe functions, like `veq` in `classify/neon.rs`. The `Avx2` type
    guarantees their only safety condition. `load256` stays `unsafe`, because of its pointer.
  - No codegen change.

## Phase E. Gaps in code already migrated (done)

These steps didn't depend on the other phases.
All of them except E5 changed no code in any build.

- **E1.** `token_view` gets `#[inline(always)]`.
  - `cpu_versions` can't take its explicit lifetime parameter.
  - Inlining it everywhere is already the status quo. No `token_view` symbol exists in any of the 3 builds, so
    all 7 call sites inline it.
  - A second copy wouldn't gain anything. Its body only builds a `Tokens` from slices.
- **E2.** `cpu_versions` on `Lanes::clear`, called from `lex_raw_impl`.
- **E3.** `cpu_versions` on `write_sentinels`, called from `lex_raw_impl` and `compress`.
- **E4.** `cpu_versions` on `utf8_seq_check`, called from `misc_pre`.
  - It stays inlined into each copy of `misc_pre`, so it doesn't repeat B12.
- **E5.** `cpu_versions` on `Lexer::ensure`, called from `lex_raw_impl`.
  - `Lexer::lex` also calls it, outside the runtime switch. It starts with `let cpu = BASELINE;`
    (rule 2), which adds no runtime dispatch.
  - On x86_64, the AVX2 copy of `ensure` is now inlined into the AVX2 copy of `lex_raw_impl`
    ("Observed drift").
- **E6.** `cpu_versions` on `get_pair_luts`. `compress_blocks_avx2` passes its `Avx2`.

## Audit after Phase E

This is the audit at the end of "Verification for each commit", done on LLVM IR for x86_64 and
aarch64.

- Method:
  - Build with `--emit=llvm-ir -C opt-level=0 -C debuginfo=1`. Nothing is inlined then, except
    `#[inline(always)]` code, which rustc's MIR inliner still inlines.
  - Walk every function reachable from `Lexer::lex_raw`, stopping at `#[cold]` functions and the
    Level 3 entry points.
  - Classify each lexer function by its name, or by the attributes above its `fn` line.
- Every function reached is a `cpu_versions` entry, a CPU-specific version or single-copy code,
  apart from the 3 groups below.
- None of the 3 groups is out of line in any release build. Each is inlined into the copies of its
  callers. So the codegen is right, but only because LLVM chose to inline them.
- In the x86_64 build, the only call from non-AVX2 code into AVX2 code is the runtime switch,
  `lex_raw` calling `__lex_raw_impl_avx2`.

The 3 groups:

1. Level 3 functions which versioned code calls, but which aren't `#[inline(never)]` entry points.
   - `Walks::run_rules`, called from `gt_run_split`. Outside tests it only returns `true`.
   - `Walks::restart`, called from `DisambiguateState::begin` and `DisambiguateState::restart`,
     and the `Walk::reset` it calls. It runs twice per lex.
2. `#[inline]` helpers in `token.rs`, outside the pipeline.
   - `is_trivia_byte`, called from `build_spans_avx2`, `build_spans_generic`, `Tokens::next_sig`,
     `prev_sig`, `not_operator_position` and `type_list_legal`.
   - `TokenKind::from_u8_unchecked`, called from `munch_walk`. `TokenKind::from_u8` is only
     reached from the `debug_assert!` in `from_u8_unchecked`.
3. Derived trait impls. Like `std` code, they can't be versioned.
   - `PartialEq::eq` on `TokenKind`, `DiagCode`, `JMode`, `JFrameKind` and `context::After`.
   - `Default::default` on `Walk`, `Frame`, `FrameKind` and `Expect`, reached only from
     `Walk::reset`.

Decided:

- Only a problem seen in the assembly is fixed, so none of the 3 groups changes.
- The one exception is `#[inline(always)]` on both versions of `Walks::run_rules`, so its `true`
  always folds into `gt_run_split`. `#[inline(never)]` would stop that.

## Phase G. Marking code which is compiled twice

The aim:

- `#[apply(cpu_versions)]` marks every function which is compiled twice, once for each CPU type,
  whatever its inline attribute. Nothing else gets it.
  - An `#[inline(always)]` function goes through `cpu_versions` unchanged, after the same checks
    as any other, including its `cpu` parameter. So removing `#[inline(always)]` versions it, with
    no other change.
  - A helper which single-copy code shares is still compiled twice. Single-copy code uses the
    copy for `BASELINE`.
- `#[apply(cpu_version(Avx2))]`, `cpu_version(Neon)` and `cpu_version(Generic)` mark CPU-specific
  code, which is compiled once, for one CPU.
  - They check the CPU type parameter. They add `#[cfg(target_arch = ...)]`, and
    `#[target_feature(...)]` unless the function is `#[inline(always)]`.
  - `cpu_version(Generic)` is designed in step G5. Generic versions might take `cpu: Baseline`,
    because the `Neon` arms call them too.
- Unmarked code is compiled once. That's single-copy code, and helpers which only it uses.
  The exceptions are derived trait impls, `std` and other crates.

Steps:

- **G1.** Move `pipeline/cpu/` to `cpu/` (done).
  - It's no longer under `pipeline`'s lint policy.
  - Its items are `pub`. In a private module that doesn't widen their visibility, and
    `redundant_pub_crate` doesn't fire. Both `to_dyn`s expect `inline_always`.
  - Codegen is unchanged. Symbol names change, because they include the CPU types' module path.
- **G2.** The code which `cpu_versions!` generates is lint-clean outside `pipeline` (done, in the
  same commit as G1).
  - In an `unsafe fn`, the entry points' calls to `__impl` are unsafe. That relied on `pipeline`
    allowing `unsafe_op_in_unsafe_fn`. Now the entry points allow it themselves.
  - The aarch64 `Generic` arms get `unsafe` blocks, like the other arms.
  - A method's outer method allows `unused_unsafe` and `undocumented_unsafe_blocks` once, for all
    its arms. A function can't, because `__impl` is nested inside it and holds the function's
    body. So a function's arms keep their own attributes.
  - The test `read_byte` has its own `unsafe` block, and the new `plus_byte` tests an `unsafe`
    method.
  - Codegen is unchanged.
- **G3.** `cpu_versions!` supports explicit lifetime parameters, so `token_view` can be marked.
- **G4.** `cpu_versions!` passes `#[inline(always)]` functions through after all its checks,
  instead of erroring. Rule 1 changes to match.
- **G5.** The `cpu_version(..)` macros, with tests.
- **G6.** AVX2 and Neon versions, and helpers which only they use, get `cpu_version(Avx2)` or
  `cpu_version(Neon)`, replacing their hand-written `#[cfg]` and `#[target_feature]`.
  Helpers without a CPU type get one, e.g. `lic_first` and `array_map`.
- **G7.** Generic versions get `cpu_version(Generic)`. They take a CPU type wherever G8 marks
  something they call, and their dispatchers' arms pass it.
  - `cook_short_generic` and `lanes_post_generic` lose `cpu_versions`. That might let rustc's MIR
    inliner inline them again, as before the fix to `cpu_versions!`. Check the codegen.
- **G8.** Mark every compile-twice function, and give it `cpu` where it has none.
  - About 50 helpers without `cpu`, 13 with it, and 14 `cpu_select!` dispatchers.
  - Helpers which both an AVX2 and a generic version call, e.g. `glue_hyphen_at`,
    `lic_verify_at` and `len_mask_ptr`.
  - Level 3 code which versioned code calls: `Walks::restart`, `Walk::reset` and
    `Walks::run_rules`.
  - `token_view`.
  - `unicode_ws_len_at_unchecked`, `next_start` and `base_kind` get `cpu` back.
  - Several commits, by module, callers before callees.
- **G9.** The same in `token.rs`: `is_trivia_byte`, `TokenKind::from_u8`,
  `TokenKind::from_u8_unchecked`, `StringSpan::new` and `StringSpan::with_cooked_invalid`.
  The `token` module and these functions are private to the crate now.
- **G10.** Extend the audit to `#[inline(always)]` functions, through the inlined scopes in the
  debug info, and check that every compile-twice function is marked. Then update the rules.

Codegen isn't expected to change in any step, apart from G7's 2 functions.
Panic line numbers change wherever lines move.

## Phase F. Runtime detection

To be planned in detail later.

- The `IMPLEMENTATION` const in `lib.rs`.
- Real detection with `is_x86_feature_detected!` in `DynCpu::detect`.
- CI.

## Verification for each commit

- `just fmt` and `just ready-lexer` must pass before each commit.
  - `ready-lexer` lints, tests and runs conformance for the host, x86_64, and x86_64 with AVX2, and checks
    that the conformance snapshots are unchanged.
- aarch64 without Neon, ignoring the warning that Neon must be enabled for the ABI:

  ```sh
  CARGO_TARGET_AARCH64_APPLE_DARWIN_RUSTFLAGS="-C target-feature=-neon" \
    cargo clippy -p oxc_lexer --target aarch64-apple-darwin
  ```

  - The same `cargo test`.

- Docs
  - `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --document-private-items --all-features -p oxc_lexer`, for
    the host and for `--target x86_64-apple-darwin`.
- Asm, for aarch64, x86_64, and x86_64 with `-C target-feature=+avx2,+bmi2,+popcnt`.
  - Build before and after the step from the same path, with a scratch target dir.
  - Build each with the command below. For x86_64, add `-C llvm-args=-x86-asm-syntax=intel`.

    ```sh
    cargo rustc -p oxc_lexer --release --target <target> -- \
      --emit=asm -C debuginfo=0 -C codegen-units=1 -C lto=off
    ```

  - Compare per function against the "Expected asm" table, and count the generic code's instructions in the
    x86_64 build.
  - Also compare against a build of A2, the last commit before B1, to track the drift across steps.
- After the last step, audit every function reachable from `lex_raw_impl`.
  - Each one must be `#[inline(always)]`, `cpu_versions`, a CPU-specific version, or single-copy code.
  - The compiler catches a caller which doesn't pass a CPU type to a versioned function. It doesn't catch a
    function which was missed and never versioned.
