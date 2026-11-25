# Changelog

All notable changes to this package will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0).

## [0.80.0] - 2026-11-23

### 🚀 Features

- d3af78a formatter_htmlish/angular: Support Tailwind class sorting (#29737) (leaysgur)
- 0fe3d0c formatter_htmlish/angular: Format template JS expressions (#29726) (leaysgur)
- ae32aed formatter_htmlish/angular: Implement printer w/o JS expressions (#29657) (leaysgur)

### 🐛 Bug Fixes

- f1816d8 formatter_htmlish: Count a blank line after unclosed element (#29881) (leaysgur)
- 5952610 formatter_htmlish/angular: Do not break a parent for ICU (#29874) (leaysgur)
- 596c70d formatter_htmlish/angular: Break as-written block parameters (#29873) (leaysgur)
- 1fb897f formatter_htmlish: Keep blank lines before borrowed `}}` when preserved (#29872) (leaysgur)

## [0.79.0] - 2026-11-16

### 🐛 Bug Fixes

- d41ed14 formatter: Keep the `;` of a lone statement in a Vue event handler (#29354) (leaysgur)
- 16b84d3 formatter/htmlish: Print CDATA sections as nodes of their own (#29348) (leaysgur)
- 79064df formatter_htmlish/angular: Keep block params, ignore whole block (#29258) (leaysgur)
- 4d50307 formatter_htmlish: Keep attrs holding character reference (#29257) (leaysgur)
- bed54ab formatter_htmlish: Keep whitespace as written only between two CJ characters (#29171) (leaysgur)
- a3660bb formatter_htmlish: Keep empty comment only element as-is (#29159) (leaysgur)
- 49b832a formatter_htmlish/vue: Preserve `v-for` alias holding `in|of` (#29156) (leaysgur)

### 📚 Documentation

- 9660ce4 oxfmt: Update htmlish and oxfmt divergences (#29334) (leaysgur)

## [0.78.0] - 2026-11-09

### ⚡ Performance

- dde9c5c formatter_htmlish: Skip `</script` escape if possible (#29078) (leaysgur)

### 📚 Documentation

- eb2eb2d formatter_htmlish: Cover block elements in `DIVERGENCES.md#interpolation-element-as-text` (#28901) (leaysgur)

## [0.77.0] - 2026-11-06

### 💥 BREAKING CHANGES

- 00d1a67 oxfmt: [**BREAKING**] Format `parser:vue` files by `oxc_formatter_htmlish` (#28848) (leaysgur)

### 🚀 Features

- 9a81e58 formatter_htmlish: Format Vue templates (#28847) (leaysgur)

### 🐛 Bug Fixes

- 4d4104d formatter_htmlish: Preserve `doctype` case in embedded (#28797) (leaysgur)

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

