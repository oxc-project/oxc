# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

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

