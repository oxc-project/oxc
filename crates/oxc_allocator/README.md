# Oxc Allocator

A high-performance memory allocator using bump-based arena allocation for fast AST node creation.

## Overview

Oxc uses a bump-based memory arena for faster AST allocations. This crate provides an [`Allocator`] that manages memory efficiently by allocating objects in a single contiguous memory region, avoiding the overhead of individual heap allocations.

## Key Features

- **Bump allocation**: Fast allocation by simply incrementing a pointer
- **Arena-based memory management**: All allocations live for the same lifetime
- **Zero-copy data structures**: Efficient `Box`, `Vec`, `String`, and `HashMap` implementations
- **Optimal for AST operations**: Perfect for parse-transform-emit workflows

## Architecture

The allocator is designed specifically for AST processing workflows where:

1. A large number of nodes are allocated during parsing
2. All nodes have the same lifetime (tied to the AST)
3. Memory is released all at once when the AST is dropped

This approach is significantly faster than using the system allocator for AST operations.

## Features

- `serialize` - Enables serialization support for `Box` and `Vec` with `serde` and `oxc_estree`.
- `pool` - Enables `AllocatorPool`.
- `bitset` - Enables `BitSet`.
- `from_raw_parts` - Adds unsafe `from_raw_parts` method (not recommended for general use).
- `fixed_size` - Makes `AllocatorPool` create large fixed-size allocators, instead of flexibly-sized ones.
  Only supported on 64-bit little-endian platforms at present.
  Usage of this feature is not advisable, and it will be removed as soon as we're able to.
- `track_allocations` - Count allocations and reallocations.
  For internal use only. The APIs provided by this feature are sketchy at best, and possibly
  undefined behavior. Do not enable this feature under any circumstances in production code.
- `disable_track_allocations` - Disables `track_allocations` feature.
  Purpose is to prevent `--all-features` enabling allocation tracking.
