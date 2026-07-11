# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.139.0] - 2026-07-06

### 🐛 Bug Fixes

- d966d0b react_compiler: Remove clippy allows (#24168) (Boshen)
- 854ef8d react_compiler: Compile generic functions instead of over-bailing on type-param hoisting (#24158) (Boshen)
- 093586c react_compiler: Align memoization cache-slot allocation with Babel (#24157) (Boshen)
- 09c8f59 react_compiler: Normalize snapshot fixture paths (#24142) (camc314)
- f13df97 react_compiler: Drop stray empty statement from catch bindings (#24133) (Boshen)
- cb2a505 react_compiler: Codegen destructuring reassignment targets (#24131) (Boshen)
- b82c394 react_compiler: Propagate codegen invariants instead of emitting empty bodies (#24128) (Boshen)
- 5771982 react_compiler: Render unchanged programs as source in fixture snapshots (#24129) (Boshen)
- 4e9194f react_compiler: Lower `delete obj.prop` to Property/ComputedDelete (#24123) (Boshen)

### ⚡ Performance

- bf1a151 react_compiler: Compile out debug printers (#24184) (Boshen)
- e4b708b react_compiler: Skip compiled files before prefilters (#24171) (Boshen)

## [0.138.0] - 2026-06-29

### 🚀 Features

- f2091b3 ast: Unify old and new `AstBuilder`s (#23875) (overlookmotel)

### ⚡ Performance

- 3ea9304 react_compiler: Use faster API to arena allocate strings (#23849) (overlookmotel)

### 📚 Documentation

- 3d61dea all: Correct capitalization in comments (#23887) (overlookmotel)

## [0.137.0] - 2026-06-18

### 🐛 Bug Fixes

- 20375f9 react_compiler: Keep imports referenced only by a computed key (#23586) (Boshen)

### ⚡ Performance

- 488b382 react_compiler: Borrow binding names in prefilter instead of allocating (#23471) (Yunfei He)

## [0.136.0] - 2026-06-15

### 🚀 Features

- 1490a0a linter/react: Implement react-compiler rule (#23202) (Boshen)
- ec266bb transformer: Run React Compiler as a feature-gated transform pass (#23201) (Boshen)

### 🐛 Bug Fixes

- de38a3f react_compiler: Keep imports referenced only by a local re-export (#23176) (Boshen)

## [0.135.0] - 2026-06-08

### 🚀 Features

- b846ab2 react_compiler: Integrate the Rust port of the React Compiler (#22942) (Boshen)

