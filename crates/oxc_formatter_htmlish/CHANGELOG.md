# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.76.0] - 2026-11-02

### 💥 BREAKING CHANGES

- 296623d oxfmt: [**BREAKING**] Format `parser:html` files by `oxc_formatter_htmlish` (#28273) (Yuji Sugiura)

### 🚀 Features

- 336b2c1 oxfmt: Support html-in-js templates closing a dynamic tag name (#28389) (Yuji Sugiura)
- 0a42af6 formatter_htmlish: Format end tags of omitted `html`, `head` and `body` (#28388) (Yuji Sugiura)
- 44d8c2b formatter_htmlish: Implement HTML-ish formatter `Html` variant (#28272) (Yuji Sugiura)

### 🐛 Bug Fixes

- fef809a formatter_htmlish: Handle XHTML (#28703) (leaysgur)
- f3dd201 formatter_htmlish: Keep SVG text under `xml:space="preserve"` as written (#28382) (Yuji Sugiura)

### ⚡ Performance

- 7cd46cb formatter_htmlish: Search script end tags by `<` in embedded (#28704) (leaysgur)
- dce1485 formatter_htmlish: Skip escaping attr values without the chosen quote (#28700) (leaysgur)
- ceb26be formatter_htmlish: Use `char` to escape attr quotes (#28697) (leaysgur)

### 📚 Documentation

- 7f18c20 formatter_htmlish: Note `class` and `srcset` are formatted with `embeddedLanguageFormatting: off` (#28465) (Yuji Sugiura)
- 01ed68a oxfmt,formatter_htmlish,formatter: Clean up behavior comments (#28384) (Yuji Sugiura)

