# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Codebase Overview

Oxc (The Oxidation Compiler) is a high-performance JavaScript/TypeScript toolchain written in Rust. It includes a parser, linter (oxlint), formatter, transformer, minifier, and other tools. The project is organized as a Rust workspace with multiple crates implementing different components.

## Essential references

- Agent-specific guide: See [AGENTS.md](AGENTS.md)
- Architecture details: See [ARCHITECTURE.md](ARCHITECTURE.md)

## Development

You run in an environment where `ast-grep` is available; whenever a search requires syntax-aware or structural matching, default to `ast-grep --lang rust -p '<pattern>'` (or set `--lang` appropriately) and avoid falling back to text-only tools like `rg` or `grep` unless I explicitly request a plain-text search.
