# `oxc_lexer` naming convention

The lexer's SIMD code handles the same concept in several forms at once.
For example, `classify_avx2` deals with which bytes are identifier chars as:

- A pointer to the `word` bitmap.
- 2 SIMD vectors holding a byte mask each.
- A `u64` with one bit per byte.

Short names (`word`, `w0`, `wordm`) don't say which is which.

We will adopt a consistent naming convention, where names say both what a value relates to,
and what form it takes, and migrate the crate to using this convention.

## Philosophy

The aim is clarity.

Clarity requires more verbose names to describe what things are, but verbosity is undesirable
in itself.

So there's a trade-off. Use the shortest name possible which makes it clear what something is.
Sometimes the context makes it obvious e.g. in a function that is all about whitespace,
don't name every variable `whitespace_this`, `whitespace_that`. `this` and `that` are fine
in that context, as long as they're unambiguous.

Consistency contributes to clarity.

Key elements which are used all over the crate should be named the same everywhere.
e.g. `src_len` should always be the var name for the length of the source text.
And `src_len` should _never_ be used for the name of something else (e.g. length of
a _section_ of the source text).

## Forms and their suffixes

| Form                                     | Name                                  | Examples                       |
| ---------------------------------------- | ------------------------------------- | ------------------------------ |
| Single value                             | Singular noun, no suffix              | `kind`, `word_carry`           |
| Array or slice                           | Plural noun, no suffix                | `kinds`, `spans`               |
| Pointer to a value                       | `<singular>_ptr` (bare `ptr` is fine) | `kind_ptr`                     |
| Pointer to start of a range              | `<plural>_ptr`                        | `kinds_ptr`                    |
| SIMD vector (`Vec128`, `Vec256`)         | `<singular>_vec`                      | `src_vec`, `kind_vec`          |
| Scalar bitmask (one bit per byte)        | `<singular>_bits`                     | `word_bits`, `whitespace_bits` |
| Bitmap (whole-source array of bitmasks)  | `<singular>_bitmap`                   | `word_bitmap`                  |
| Lookup table (`VecData128`, a LUT field) | Named after the table                 | `merged_lut_lo` (a `VecData`)  |
| Index                                    | `<singular>_idx`                      | `block_idx`, `src_idx`         |
| Length or count                          | `<singular>_len` / `<singular>_count` | `src_len`, `block_count`       |
| Boolean                                  | `is_*`                                | `is_ts`                        |

- A vector is `_vec` whatever its bytes mean:
  data (`src_vec`, `kind_vec`) or a byte mask from a comparison,
  with each byte `0xFF` or `0x00` (`word_vec`, `not_whitespace_vec`).
- A scalar bitmask is what `movemask` produces: bit `i` describes byte `i`.
  A bitmap covers the whole source as an array of `u64` bitmasks, one per 64-byte block.
  So `word_bits` is one block of `word_bitmap`.
- Bitmaps are currently raw pointers, so they're `word_bitmap_ptr` for now.
  A planned `Bitmap` type will wrap the pointer,
  with methods like `get`, `set`, `clear` and `next0` replacing the free functions in `bitmap.rs`.
  Then the `_ptr` goes: `word_bitmap`.
- Singular or plural:
  - With `_vec`, `_bits` or `_bitmap`, the name before the suffix describes one element,
    so it's singular: `kind_vec`, `hi_nibble_vec`, `word_bits`, `word_bitmap`.
    The suffix carries the plural, as in "word list".
    Not `words_bitmap`, which would also read as machine words.
  - It's plural only when each element holds several things:
    in `word_flags_vec`, each byte is a set of flag bits.
  - An array or slice has no suffix to carry the plural, so its name is plural: `kinds`, `spans`.
  - A pointer is named after what it points at.
    A pointer to the start of an array takes the array's name (`kinds_ptr`, `word_bitmap_ptr`).
    A pointer to a single element would be singular (`kind_ptr`).
  - A vector holding a whole lookup table is the exception.
    It's named after the table (`word_lut_lo_vec`), not after an entry.
    Lane `i` of a per-byte vector is about source byte `i`,
    but the lanes of a table vector are table entries, and a shuffle uses the vector as a whole.
- An index or length says what it indexes or measures: `src_len` and `src_idx`, not `n` and `i`.
  `i` and `j` are preferable for short loops where the meaning is obvious from the context.
- Booleans are the one form marked with a prefix, following the usual Rust convention:
  `is_ts`, not `ts`.

## Combining suffixes

- Suffixes stack in reading order: `word_bitmap_ptr` is a pointer to the word bitmap.
- Qualifiers go before the form suffix: `hi_nibble_vec`, `punct_hash_lo_vec`.
- A chunk index goes last:
  `src_vec0` and `src_vec1` are the first and second 32 bytes of a 64-byte block.
  Likewise `kind_vec0`, `misc_bits1`.

## Qualifiers with fixed meanings

- Paired qualifiers have the same length, so that lines which do the same thing
  for each of the pair line up:

  ```rust
  let kind_top_vec = punct_kind_lut_top_vec.shuffle_u8s(hash_vec);
  let kind_bot_vec = punct_kind_lut_bot_vec.shuffle_u8s(hash_vec);
  ```

- `lo` / `hi`: indexed by a byte's low / high nibble, for nibble lookup tables
  (`word_lut_lo_vec`), or the nibbles themselves (`hi_nibble_vec`).
- `top` / `bot`: the top / bottom half of a table, as written in its definition
  (`halves[0]` / `halves[1]`), e.g. `punct_kind_lut_top_vec`.
  - Not `lo` / `hi`, which mean nibbles. Both halves of `PUNCT_HASH_KINDS` are indexed
    by the same thing (the low 4 bits of the hash), and bit 4 of the hash picks the half.
  - Never a bare `0` / `1`, which mean chunks.
- `not_`: the complement of a mask (`not_whitespace_vec`).

## Naming the concept

- Spell words out: `hi_nibble`, not `hn`; `non_ascii`, not `na`; `whitespace`, not `ws`;
  `word_carry`, not `cw`.
- A vector loaded from a table is named after the table,with `lut` in the singular,
  because it holds one table: `WORD_LUTS.lo` becomes `word_lut_lo_vec`, and the halves
  of the punctuator kinds table are `punct_kind_lut_top_vec` and `punct_kind_lut_bot_vec`.
  The constants stay plural (`WORD_LUTS`), because each holds several tables.
- `hash` is short for "hash table", so it does the job of `lut`:
  `PUNCT_HASH_LO` becomes `punct_hash_lo_vec`, not the longer `punct_hash_lut_lo_vec`.
- The result of looking bytes up in a table drops `lut`:
  `word_flags_vec`, because each entry is a set of flag bits.
  Likewise `hash_vec` and `kind_vec`.
- Loop counters say what they count:
  `block_idx` (index of the block),
  `src_idx` (index in the source of the block's first byte),
  `block_count`.
- Avoid a name that reads as something else:
  `whitespace`, not `space` (the space char);
  `keyword_initial`, not `keyword_init` (initialise).

## Names shared across the pipeline

Each concept has one name in every stage. These replace the old abbreviations:

| Old      | New               | Meaning                                                    |
| -------- | ----------------- | ---------------------------------------------------------- |
| `n`      | `src_len`         | Length of the source                                       |
| `ts`     | `is_ts`           | Lexing TypeScript                                          |
| `st`     | `token_start`     | Bitmap of token starts (trivia included)                   |
| `kwinit` | `keyword_initial` | Bitmap of bytes which can be the first letter of a keyword |
| `opch`   | `op_char`         | Bitmap of bytes which can be part of an operator           |
| `ws`     | `whitespace`      | Whitespace                                                 |

Unchanged: `src`, `cpu`, and the bitmaps `word`, `digit`, `dot`, `misc`.

`token_start` marks the start of every token, trivia included, throughout the pipeline.
"Significant" (non-trivia) is a separate concept, decided from a token's kind (`is_trivia_byte`),
in `compress` (`sig_kinds`) and `disambiguate` (`prev_sig`).

## Specific cases

- A var containing an individual byte of source should be `b` or `byte`, not `c` or `chr`.
  `b` is for bytes, `c` is for `char`s.

## Example

From `classify_avx2`:

```rust
// Before
let v0 = Vec256::load_unaligned(src.add(i), cpu);
let hn0 = v0.shr_u16::<4>() & 0x0f;
let tb0 = v_wblo.shuffle_u8s(v0) & v_wbhi.shuffle_u8s(hn0);
let na0 = v0.lt_i8(0);
let w0 = (tb0 & WORD_MASK).gt_i8(0) | na0;
let wordm = (w0.movemask() as u64) | ((w1.movemask() as u64) << 32);
*word.add(b) = wordm;

// After
let src_vec0 = Vec256::load_unaligned(src_ptr.add(src_idx), cpu);
let hi_nibble_vec0 = src_vec0.shr_u16::<4>() & 0x0f;
let word_flags_vec0 =
    word_lut_lo_vec.shuffle_u8s(src_vec0) & word_lut_hi_vec.shuffle_u8s(hi_nibble_vec0);
let non_ascii_vec0 = src_vec0.lt_i8(0);
let word_vec0 = (word_flags_vec0 & WORD_MASK).gt_i8(0) | non_ascii_vec0;
let word_bits = (word_vec0.movemask() as u64) | ((word_vec1.movemask() as u64) << 32);
*word_bitmap_ptr.add(block_idx) = word_bits;
```

## Status

- Applied to `classify_avx2` and its helper `punct_kinds` (`pipeline/classify/avx2.rs`),
  as a pilot.
- Not yet applied anywhere else, including `classify_generic` and `classify_neon`.
  Until then, the rest of the pipeline uses the old names,
  e.g. the `Lexer` fields and the call to `classify_avx2` in `classify/mod.rs`.
- The pilot only renames and adds comments.
  `classify_avx2`'s parameters and statements are in their original order,
  so codegen is unchanged (checked: identical asm).
- The constants keep their old names for now, including `PUNCT_HASH_KINDS`,
  though its vectors are already `punct_kind_lut_top_vec` and `punct_kind_lut_bot_vec`.

## Open questions

- Struct fields of vector type,
  e.g. Neon's `Luts` (`wb_lo`, `punct_lo`, ...) and `Classified` (`word`, `kind`, ...).
  Should they get `_vec` too, or does the struct's role make it implicit?
- `lo` / `hi` also appear for the 128-bit halves of a 256-bit vector
  (`unpack_lo_u32`, `join_lo_halves`, and `compress_avx2`'s `lo` / `hi`).
  Those need their own qualifier, so `lo` / `hi` keep meaning nibbles.
- The constants and functions named after the old abbreviations:
  `WS_MASK`, `KWINIT_MASK` and `OPCH_MASK` (`classify/luts.rs`, shared with `classify_neon`),
  and `is_ws`, `is_kw_init` and `is_op_char`.
  Rename them with the rest of the pipeline.
- Whether the parameter order (source, outputs, options, `cpu`) should be a rule for every stage.
