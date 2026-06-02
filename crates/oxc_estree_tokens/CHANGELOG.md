# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.116.0] - 2026-03-02

### 🚀 Features

- 25c2e25 estree/tokens: Add function to update tokens in place (#19856) (overlookmotel)
- 9e11dc6 parser,estree,coverage: Collect tokens in parser and convert to ESTree format (#19497) (camc314)

### 🐛 Bug Fixes

- 7682e5a linter/plugins: Decode escapes in identifier tokens (#19838) (overlookmotel)
- 06767ed estree/tokens: Convert `this` tokens in `TSTypeName` (#19815) (overlookmotel)
- f5694ce estree/tokens: Reverse field order of `regex` object in tokens (#19679) (overlookmotel)
- b2b7a55 estree/tokens: Generate tokens for files with BOM (#19535) (overlookmotel)
- 50a7514 estree: Fix tokens for JSX (#19524) (overlookmotel)

### ⚡ Performance

- c1bfdcf estree/tokens: Preallocate sufficient space for tokens JSON (#19851) (overlookmotel)
- 4b0611a estree/tokens: Introduce `ESTreeTokenConfig` trait (#19842) (overlookmotel)
- 81bab90 estree/tokens: Do not JSON-encode keyword, punctuator, etc tokens (#19814) (overlookmotel)
- 6260ddd estree/tokens: Do not JSON-encode `this` identifiers (#19813) (overlookmotel)
- b378f4a estree/tokens: Do not JSON-encode JSX identifiers (#19812) (overlookmotel)
- 5016d92 estree/tokens: Handle regex tokens separately (#19796) (overlookmotel)
- 780a68e estree/tokens: Use strings from AST for identifier tokens (#19744) (overlookmotel)
- ec88f6a estree/tokens: Serialize tokens while visiting AST (#19726) (overlookmotel)
- bc6507f estree/tokens: Serialize with `ESTree` not `serde` (#19725) (overlookmotel)
- ec24859 estree/tokens: Do not branch on presence of override twice (#19721) (overlookmotel)
- dac14be estree/tokens: Replace hash map with `Vec` (#19718) (overlookmotel)
- b9d2443 estree/tokens: Replace multiple hash sets into a single hash map (#19716) (overlookmotel)
- 8940f66 estree/tokens: Serialize tokens to compact JSON (#19572) (overlookmotel)

### 📚 Documentation

- b2b7a64 estree/tokens: Correct comment (#19873) (overlookmotel)
- 0399311 estree/tokens: Improve comments (#19836) (overlookmotel)

