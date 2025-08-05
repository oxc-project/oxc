# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.80.0] - 2025-08-03

### 📚 Documentation

- 45e2fe8 rust: Fix typos and grammar mistakes in Rust documentation comments (#12715) (Copilot)
- de1de35 rust: Add comprehensive README.md documentation for all Rust crates (#12706) (Copilot)




## [0.78.0] - 2025-07-24

### 🚜 Refactor

- 124d376 allocator: Remove unnecessary `Send` impl for `FixedSizeAllocator` (#12426) (overlookmotel)
- c375981 napi/oxlint: Simplify atomic operations (#12425) (overlookmotel)
- c1b2f48 napi/oxlint: Prevent lint warning in release mode (#12408) (overlookmotel)


## [0.77.3] - 2025-07-20

### 🚀 Features

- 6d2b549 napi/oxlint: Pass AST in buffer to JS (#12350) (overlookmotel)
- b0db2d7 allocator: `FixedSizeAllocator` store flag recording if owned by both Rust and JS (#12381) (overlookmotel)
- bc0fbe5 allocator: `AllocatorPool` store IDs in `Allocator`s (#12310) (overlookmotel)
- de006a1 allocator: Add `Allocator::end_ptr` method (#12330) (overlookmotel)

### 🚜 Refactor

- 319fc3b allocator/fixed-size: Store `alloc_ptr` in the memory block backing the allocator (#12380) (overlookmotel)
- 8fe1aec ast_tools, allocator, napi/parser: Rename vars (#12379) (overlookmotel)
- dfe54b4 allocator: Move all fixed size allocator code into 1 file (#12309) (overlookmotel)



## [0.77.1] - 2025-07-16

### 🐛 Bug Fixes

- 13c5783 allocator: Fix `FixedSizeAllocator` pointer maths (#12299) (overlookmotel)

### 🚜 Refactor

- 04e6a2f allocator: Improve documentation on pointer alignment (#12307) (overlookmotel)
- 2f9bd11 allocator: Fixed size allocator leave space for metadata after arena (#12278) (overlookmotel)
- 39ef911 napi/parser, allocator: Raw transfer: store buffer size and align as consts (#12275) (overlookmotel)
- f130a0c allocator: Disable fixed size allocators on unsupported platforms (#12272) (overlookmotel)

### ⚡ Performance

- cc1e9fc napi/parser: Raw transfer: reduce size of buffer by 16 bytes (#12277) (overlookmotel)


## [0.77.0] - 2025-07-12

### 💥 BREAKING CHANGES

- facd3cd allocator: [**BREAKING**] Remove `vec!` macro (#12206) (overlookmotel)

### 🚀 Features

- 152e59d napi/oxlint: Read source text into start of allocator (#12122) (overlookmotel)
- 8d710a2 allocator: Add `Allocator::alloc_bytes_start` method (#12083) (overlookmotel)
- 704350a allocator: Fixed size allocators (#12082) (overlookmotel)

### 🚜 Refactor

- 6ff6643 allocator: Add error type to `RawVec` (#12204) (overlookmotel)
- a9482f2 allocator: Remove dead code from `Vec` (#12203) (overlookmotel)
- 068669f allocator: Add `AllocatorWrapper` abstraction to `AllocatorPool` (#12081) (overlookmotel)

### 📚 Documentation

- b3a076b allocator: Extend doc comments for `Vec` and `RawVec` (#12205) (overlookmotel)


## [0.76.0] - 2025-07-08

### 💥 BREAKING CHANGES

- 1108a5c allocator: [**BREAKING**] Remove `DerefMut` impl from `AllocatorGuard` (#12077) (overlookmotel)

### 📚 Documentation

- 9cf5552 allocator: Improve doc comments for `AllocatorPool` (#12076) (overlookmotel)

### ⚡ Performance

- d732e85 allocator: `Allocator::from_raw_parts` get offset of chunk footer field as const (#12080) (overlookmotel)


## [0.75.1] - 2025-07-03

### 📚 Documentation

- ff1d42f allocator: Update comments about `bumpalo` version (#12033) (overlookmotel)


## [0.75.0] - 2025-06-25

### 💥 BREAKING CHANGES

- 9a2548a napi/parser: [**BREAKING**] Add `range` option (#11728) (Bacary Bruno Bodian)


## [0.74.0] - 2025-06-23

### 🚀 Features

- f2ce5ad allocator: Add `Allocator::alloc_slice_copy` method (#11822) (overlookmotel)
- 4ba5258 allocator: Add `Allocator::alloc_layout` method (#11821) (overlookmotel)



## [0.73.1] - 2025-06-17

### 🚀 Features

- 38dc614 oxc_linter: Reuse allocators (#11736) (camc314)

### 🚜 Refactor

- 01e52bc allocator: Re-order code (#11759) (overlookmotel)

### 📚 Documentation

- 9fefb46 allocator: Improve `Allocator` code examples (#11670) (overlookmotel)

### ⚡ Performance

- 2641030 allocator: Reduce operations while `Mutex` lock is held in `AllocatorPool` (#11761) (overlookmotel)
- f539f64 allocator: Remove `Arc` from `AllocatorPool` (#11760) (overlookmotel)



# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project does not adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) until v1.0.0.

## [0.72.0] - 2025-05-24

- c16ea02 allocator: [**BREAKING**] Remove `String` (#11262) (overlookmotel)

### Features

- c901b5b allocator: Introduce `StringBuilder` (#11257) (overlookmotel)
- 03390ad allocator: `TakeIn` trait with `AllocatorAccessor` (#11201) (Boshen)
- 4feeeee span: Add `Atom::from_strs_array_in` method (#11261) (overlookmotel)

### Bug Fixes

- 250e56f allocator/vec: Fix unsoundness in `Vec::extend_from_slices_copy` (#11200) (overlookmotel)

### Refactor

- ddcf248 allocator: Use type alias for `InnerVec` (#11209) (overlookmotel)
- cef5452 allocator: `TakeIn::take_in_box` accept any `AllocatorAccessor` (#11216) (overlookmotel)
- 6827543 allocator: `InnerVec` use `Alloc` trait (#11199) (overlookmotel)
- a2ab84b allocator: Introduce `Alloc` trait (#11198) (overlookmotel)
- 8ec0c74 allocator/vec: Access `len` and `cap` fields via getters/setters (#11081) (overlookmotel)

## [0.71.0] - 2025-05-20

### Features

- c60382d allocator/vec2: Change `len` and `cap` fields from `usize` to `u32` (#10884) (Dunqing)

### Performance

- a6057c7 allocator/vec: Remove `SetLenOnDrop` (#11079) (overlookmotel)
- 7e69c08 allocator/vec: Remove `alloc_guard` from `RawVec::from_raw_parts_in` + clarify safety docs (#11073) (overlookmotel)
- 5dcd0f1 allocator/vec2: Reorder `RawVec` fields (#11050) (Dunqing)

### Documentation

- 7c84a56 allocator/vec: Correct safety comments for `RawVec::append_elements` (#11072) (overlookmotel)

### Refactor

- f081757 allocator/vec: Simplify comparison macros (#11178) (overlookmotel)
- b4b7d09 allocator/vec: Remove `RawVec::with_capacity_zeroed_in` (#11177) (overlookmotel)
- 44630a9 allocator/vec: Re-order arguments to `RawVec::from_raw_parts_in` (#11176) (overlookmotel)
- 31c5169 allocator/vec: Rename vars and lifetimes (#11175) (overlookmotel)
- aa76a16 allocator/vec: Re-order methods (#11080) (overlookmotel)
- 2f05c54 allocator/vec: Limit scope of `#[expect(clippy::cast_possible_truncation)]` (#11074) (overlookmotel)
- fc2f040 allocator/vec: Clarify comment (#11071) (overlookmotel)
- 7d54577 allocator/vec2: Move `len` field from `Vec` to `RawVec` (#10883) (Dunqing)

## [0.69.0] - 2025-05-09

### Bug Fixes

- 446d9b3 allocator: `Allocator::from_raw_parts` do not use const assertion for endianness test (#10888) (overlookmotel)

### Refactor

- b16331e ast/estree: Generalize concatenating fields with `Concat2` (#10848) (overlookmotel)

## [0.68.0] - 2025-05-03

### Features

- 4b4b09e allocator: Add `String::set_len` method (#10757) (overlookmotel)
- d5f66fb allocator: Implement `Display` for `Box` (#10731) (overlookmotel)
- 3cd3d23 allocator/vec2: Align `RawVec::reserve` with standard library implementation (#10701) (Dunqing)
- 7f2f247 allocator/vec2: Add specialized `grow_one` method (#9855) (Dunqing)
- 6ce3bbb allocator/vec2: Introduce `extend_desugared` method as `extend` internal implementation (#10670) (Dunqing)

### Performance

- 4eaef66 allocator/vec2: Align min amortized cap size with `std` (#9857) (Dunqing)
- 04e0390 allocator/vec2: Replace `self.reserve(1)` calls with `self.grow_one()` for better efficiency (#9856) (Dunqing)
- 2dc4779 allocator/vec2: Calling `Bump::grow` or `Bump::shrink` at the call site directly instead of calling `realloc` (#10686) (Dunqing)
- b4953b4 allocator/vec2: Resolve performance regression for `extend` by marking reserve as `#[cold]` and `#[inline(never)]` (#10675) (Dunqing)

### Documentation

- c48f6ae allocator: Document cargo features (#10695) (overlookmotel)

## [0.66.0] - 2025-04-23

### Testing

- 227febf allocator: Ignore a slow doc test (#10521) (Dunqing)

## [0.64.0] - 2025-04-17

### Documentation

- 63d4aa6 allocator: Fix quotes in comment (#10353) (overlookmotel)

### Refactor

- e4c80b4 allocator/vec2: Import `handle_alloc_error` function from `allocator_api2` instead of writing a custom one (#9860) (Dunqing)

## [0.63.0] - 2025-04-08

### Features

- c198578 allocator: Add `TakeIn::take_in_box` method (#10169) (Dunqing)

## [0.62.0] - 2025-04-01

### Features

- 8cd7430 allocator: `TakeIn` trait (#9969) (overlookmotel)
- 6a8c2fd allocator/vec2: Align the `retain` method with the standard implementation (#9752) (Dunqing)

### Refactor

- c971328 allocator/vec2: Rename parameters and method name to align with `std` (#9858) (Dunqing)

## [0.61.1] - 2025-03-21

### Features

- bc0670c tasks,oxc_allocator: Add new method clone_in_with_semantic_ids for `CloneIn` trait (#9894) (IWANABETHATGUY)

## [0.61.0] - 2025-03-20

### Features

- 38ad787 data_structures: Add `assert_unchecked!` macro (#9885) (overlookmotel)

## [0.59.0] - 2025-03-18

### Features

- 65643bc allocator: Remove drop operations from Vec2 (#9679) (Dunqing)
- 5cc614a allocator: Replace allocator_ap2's Vec with Vec2 (#9656) (Dunqing)
- caa477c allocator/vec: Remove `ManuallyDrop` wrapper (#9742) (Dunqing)

### Performance

- 17a9320 allocator/vec2: Optimize reserving memory (#9792) (Dunqing)

### Refactor

- d13817e allocator: Improve safety of `String::from_utf8_unchecked` (#9772) (overlookmotel)
- a1a8b93 allocator/vec: Add comment about lifetime bound on `CloneIn` for `Vec` (#9771) (overlookmotel)

## [0.58.0] - 2025-03-13

- f2b0cc1 allocator: [**BREAKING**] Remove `Vec::into_boxed_slice` method (#9735) (Dunqing)

### Features

- 65d9662 allocator: Add `Vec2::retain_mut` method (#9655) (Dunqing)
- 3943563 allocator: Connect `Vec2` module and make it compile (#9647) (Dunqing)
- 3d4400c allocator: Add `Vec2` (#9646) (Dunqing)

### Performance

- 89b6e4c allocator: Remove overflow checks from `String::from_strs_array_in` (#9650) (overlookmotel)

### Documentation

- daf7a1e allocator/vec: Fix link in doc comment for `Vec2` (#9729) (overlookmotel)

### Refactor

- 5c5e010 allocator/vec: Disable lint warnings in `vec2` files (#9730) (overlookmotel)
- 6c86961 allocator/vec: Comment out feature-gated methods (#9728) (overlookmotel)

### Testing

- ed6fcf2 allocator: Fix tests (#9727) (overlookmotel)

## [0.57.0] - 2025-03-11

### Documentation

- 31a2618 allocator: Add safety constraint for `String::from_raw_parts_in` (#9640) (overlookmotel)

### Refactor

- 44101bd allocator: Refactor and improve safty comments of `String::from_strs_array_in` (#9639) (overlookmotel)

## [0.56.1] - 2025-03-07

### Features

- 8b51a75 allocator: Add `String::from_strs_array_in` (#9329) (Dunqing)

## [0.55.0] - 2025-03-05

### Features

- d55dbe2 ast/estree: Raw transfer (experimental) (#9516) (overlookmotel)

## [0.53.0] - 2025-02-26

### Refactor

- d94fc15 allocator: Reduce scope of `unsafe` blocks (#9319) (overlookmotel)
- b09249c ast/estree: Rename serializers and serialization methods (#9284) (overlookmotel)

## [0.52.0] - 2025-02-21

- 216b33f ast/estree: [**BREAKING**] Replace `serde` with custom `ESTree` serializer (#9256) (overlookmotel)

### Features


### Documentation

- 3414824 oxc: Enable `clippy::too_long_first_doc_paragraph` (#9237) (Boshen)

### Refactor

- e32d6e2 allocator, linter: Shorten `serde` impls (#9254) (overlookmotel)

## [0.49.0] - 2025-02-10

### Styling

- a4a8e7d all: Replace `#[allow]` with `#[expect]` (#8930) (overlookmotel)

## [0.48.0] - 2025-01-24

### Features

- 2a2ad53 allocator: Add `Allocator::capacity` and `used_bytes` methods (#8621) (overlookmotel)
- 6801c81 allocator: Add `Allocator::new` and `with_capacity` methods (#8620) (overlookmotel)

### Performance

- 787aaad allocator: Make `String` non-drop (#8617) (overlookmotel)

### Documentation

- c1d243b allocator: Improve docs for `Allocator` (#8623) (overlookmotel)
- 01a5e5d allocator: Improve docs for `HashMap` (#8616) (overlookmotel)
- 87568a1 allocator: Reformat docs (#8615) (overlookmotel)

### Refactor

- ae8db53 allocator: Move `Allocator` into own module (#8656) (overlookmotel)
- 0f85bc6 allocator: Reduce repeat code to prevent `Drop` types in arena (#8655) (overlookmotel)
- de76eb1 allocator: Reorder `Box` methods (#8654) (overlookmotel)

## [0.47.0] - 2025-01-18

- fae4cd2 allocator: [**BREAKING**] Remove `Vec::into_string` (#8571) (overlookmotel)

- 95bc0d7 allocator: [**BREAKING**] `Allocator` do not deref to `bumpalo::Bump` (#8569) (overlookmotel)

### Features

- bf4e5e1 allocator: Add `HashMap` (#8553) (overlookmotel)

### Bug Fixes

- e87c001 allocator: Statically prevent memory leaks in allocator (#8570) (overlookmotel)

### Performance

- 76ea52b allocator: Inline `Box` methods (#8572) (overlookmotel)
- 93df57f allocator: `#[inline(always)]` methods of `Vec` which just delegate to `allocator_api2` (#8567) (overlookmotel)
- 5a28d68 allocator: `#[inline(always)]` methods of `HashMap` which just delegate to `hashbrown` (#8565) (overlookmotel)

### Documentation

- fa1a6d5 allocator: Update docs for `Vec` (#8555) (overlookmotel)

### Refactor

- ac05134 allocator: `String` type (#8568) (overlookmotel)
- 68fab81 allocator: Rename inner `Vec` type (#8566) (overlookmotel)

## [0.45.0] - 2025-01-11

### Features

- 6c7acac allocator: Implement `IntoIterator` for `&mut Vec` (#8389) (overlookmotel)
- 06e1780 minifier: Improve `StatementFusion` (#8194) (Boshen)

### Bug Fixes

- eb25bc0 allocator: Fix lifetimes on `IntoIterator` for `Vec` (#8388) (overlookmotel)

## [0.43.0] - 2024-12-21

### Features

- 75b775c allocator: `Vec<u8>::into_string` (#8017) (overlookmotel)
- 8547e02 ast: Implement `allocator_api2` for `Allocator` (#8043) (Boshen)

### Performance

- 414e828 semantic: Allocate symbol data in Allocator (#8012) (Boshen)

## [0.39.0] - 2024-12-04

### Bug Fixes

- 896ff86 minifier: Do not fold if statement block with lexical declaration (#7519) (Boshen)

## [0.37.0] - 2024-11-21

### Features

- 39afb48 allocator: Introduce `Vec::from_array_in` (#7331) (overlookmotel)

## [0.34.0] - 2024-10-26

### Features

- 419343b traverse: Implement `GetAddress` for `Ancestor` (#6877) (overlookmotel)

### Refactor

- adb5039 allocator: Add `impl GetAddress for Address` (#6891) (overlookmotel)

## [0.33.0] - 2024-10-24

- e1c2d30 allocator: [**BREAKING**] Make `Vec` non-drop (#6623) (overlookmotel)

### Bug Fixes


### Refactor

- ab8aa2f allocator: Move `GetAddress` trait into `oxc_allocator` (#6738) (overlookmotel)

## [0.32.0] - 2024-10-19

### Features

- 5ee1ef3 allocator: Add `Vec::into_boxed_slice` (#6195) (DonIsaac)

### Documentation

- 9f555d7 allocator: Clarify docs for `Box` (#6625) (overlookmotel)
- 06e75b0 allocator: Enable lint warnings on missing docs, and add missing doc comments (#6613) (DonIsaac)

## [0.31.0] - 2024-10-08

### Performance

- 5db9b30 allocator: Use lower bound of size hint when creating Vecs from an iterator (#6194) (DonIsaac)

### Refactor

- f7d1136 allocator: Remove unnecessary `Vec` impl (#6213) (overlookmotel)

## [0.30.2] - 2024-09-27

### Documentation

- 3099709 allocator: Document `oxc_allocator` crate (#6037) (DonIsaac)

## [0.27.0] - 2024-09-06

### Features

- e8bdd12 allocator: Add `AsMut` impl for `Box` (#5515) (overlookmotel)

## [0.25.0] - 2024-08-23

### Refactor

- a4247e9 allocator: Move `Box` and `Vec` into separate files (#5034) (overlookmotel)

## [0.24.3] - 2024-08-18

### Refactor

- a6967b3 allocator: Correct code comment (#4904) (overlookmotel)
- 90d0b2b allocator, ast, span, ast_tools: Use `allocator` as var name for `Allocator` (#4900) (overlookmotel)

## [0.24.2] - 2024-08-12

### Features

- 8e10e25 allocator: Introduce `Address` (#4810) (overlookmotel)

## [0.24.0] - 2024-08-08

### Features

- 23b0040 allocator: Introduce `CloneIn` trait. (#4726) (rzvxa)

## [0.23.0] - 2024-08-01

### Performance

- 4c6d19d allocator: Use capacity hint (#4584) (Luca Bruno)

## [0.22.0] - 2024-07-23

### Refactor

- 504daed allocator: Rename fn params for `Box::new_in` (#4431) (overlookmotel)

## [0.17.2] - 2024-07-08

### Features

- 115ac3b allocator: Introduce `FromIn` and `IntoIn` traits. (#4088) (rzvxa)

## [0.15.0] - 2024-06-18

### Features

- 8f5655d linter: Add eslint/no-useless-constructor (#3594) (Don Isaac)

## [0.13.0] - 2024-05-14

### Refactor

- 7e1fe36 ast: Squash nested enums (#3115) (overlookmotel)

## [0.12.5] - 2024-04-22

### Refactor

- 6bc18e1 bench: Reuse allocator in parser + lexer benchmarks (#3053) (overlookmotel)

## [0.12.4] - 2024-04-19

### Features

- 063b281 allocator: Make `Box`'s PhantomData own the passed in `T` (#2952) (Boshen)

## [0.6.0] - 2024-02-03

### Documentation

- a1271af allocator: Document behaviour of `Box` (Boshen)

## [0.5.0] - 2024-01-12

### Features

- a6d9356 allocator: Add `From` API (#1908) (Boshen)

## [0.4.0] - 2023-12-08

### Refactor

- 1a576f6 rust: Move to workspace lint table (#1444) (Boshen)

## [0.2.0] - 2023-09-14

### Refactor
- 12798e0 Improve code coverage a little bit (Boshen)- fdf288c Improve code coverage in various places (#721) (Boshen)

