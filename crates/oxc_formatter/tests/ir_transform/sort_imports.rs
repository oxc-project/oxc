use super::assert_format;

#[test]
fn should_not_sort_by_default() {
    assert_format(
        r#"
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import type d from "d";
import a from "a";
"#,
        "{}",
        r#"
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import type d from "d";
import a from "a";
"#,
    );
}

// ---

#[test]
fn should_sort() {
    assert_format(
        r#"
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import d from "d";
import a from "a";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import a from "a";
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import d from "d";
"#,
    );
    // Natural ASC order by default
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { log } from "./log";
import { log1p } from "./log1p";
import { log2 } from "./log2";
import { log10 } from "./log10";
"#,
    );
    // Dynamic imports should not affect sorting
    assert_format(
        r#"
import c from "c";
import b from "b";
import("a");
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import b from "b";
import c from "c";
import("a");
"#,
    );
    assert_format(
        r#"
import internal from "~/internal";
import internal2 from "@/internal2";
import external from "external";
import external2 from "@external2";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import external2 from "@external2";
import external from "external";

import internal2 from "@/internal2";
import internal from "~/internal";
"#,
    );
}

#[test]
fn should_handle_shebang() {
    assert_format(
        r#"
#!/usr/bin/node
// b
import { b } from "b";
// a
import { a } from "a";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
#!/usr/bin/node
// a
import { a } from "a";
// b
import { b } from "b";
"#,
    );
}

#[test]
fn should_handle_single_import() {
    assert_format(
        r#"
import A from "a";

console.log(A);
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import A from "a";

console.log(A);
"#,
    );
}

#[test]
fn should_handle_same_source_imports() {
    assert_format(
        r#"
import { z } from "a";
import { y } from "a";
import { x } from "a";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { z } from "a";
import { y } from "a";
import { x } from "a";
"#,
    );
}

#[test]
fn should_sort_regardless_of_quotes() {
    assert_format(
        r#"
import b from "b";
import a from "a";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import a from "a";
import b from "b";
"#,
    );
    // Change quote style
    assert_format(
        r#"
import b from "b";
import a from "a";
"#,
        r#"{
  "experimentalSortImports": {},
  "singleQuote": true
}"#,
        r"
import a from 'a';
import b from 'b';
",
    );
}

#[test]
fn should_sort_by_module_source_not_import_specifier() {
    // Ensure sorting uses the module path after "from", not the import specifier
    assert_format(
        r#"
import { Zoo } from "aaa";
import { Apple } from "zzz";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { Zoo } from "aaa";
import { Apple } from "zzz";
"#,
    );
    // Named imports with similar specifier names but different paths
    assert_format(
        r#"
import { Named } from "./z-path";
import { Named } from "./a-path";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { Named } from "./a-path";
import { Named } from "./z-path";
"#,
    );
    // Multiple specifiers - should sort by path not by first specifier
    assert_format(
        r#"
import { AAA, BBB, CCC } from "./zzz";
import { XXX, YYY, ZZZ } from "./aaa";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { XXX, YYY, ZZZ } from "./aaa";
import { AAA, BBB, CCC } from "./zzz";
"#,
    );
}

#[test]
fn should_support_style_imports_with_query() {
    assert_format(
        r#"
import b from "./b.css?raw";
import a from "./a.css?";
import c from "./c.css";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import a from "./a.css?";
import b from "./b.css?raw";
import c from "./c.css";
"#,
    );
}

#[test]
fn should_remove_newlines_only_between_import_chunks() {
    assert_format(
        r#"
import d from "~/d";

import c from "~/c";


import b from "~/b";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import b from "~/b";
import c from "~/c";
import d from "~/d";
"#,
    );
    // Newlines are removed, but comments are preserved
    assert_format(
        r#"
import d from "./d"; // D
// c1
// c2
import c from "./c"; // C
// b
import b from "./b"; // B
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
// b
import b from "./b"; // B
// c1
// c2
import c from "./c"; // C
import d from "./d"; // D
"#,
    );
    // Between imports and other code, newlines are preserved
    assert_format(
        r#"
import x2 from "./x2";
import x1 from "./x1";
// Empty line below should be preserved

const a = 1;

// These are preserved too

const b = 2;
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import x1 from "./x1";
import x2 from "./x2";
// Empty line below should be preserved

const a = 1;

// These are preserved too

const b = 2;
"#,
    );
}

#[test]
fn should_preserve_inline_comments_during_sorting() {
    assert_format(
        r#"
import { a } from "a";
import { b1, b2 } from "b"; // Comment
import { c } from "c";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { a } from "a";
import { b1, b2 } from "b"; // Comment
import { c } from "c";
"#,
    );
}

#[test]
fn should_stop_grouping_when_other_statements_appear() {
    assert_format(
        r#"
import type { V } from "v";

export type { U } from "u";

import type { T1, T2 } from "t";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import type { V } from "v";

export type { U } from "u";

import type { T1, T2 } from "t";
"#,
    );
    assert_format(
        r#"
import type { V } from "v";
export type { U } from "u";
import type { T1, T2 } from "t";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import type { V } from "v";
export type { U } from "u";
import type { T1, T2 } from "t";
"#,
    );
    // Every line other than import lines should break the grouping
    assert_format(
        r#"
import type { V } from "v";
const X = 1;
import type { T1, T2 } from "t";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import type { V } from "v";
const X = 1;
import type { T1, T2 } from "t";
"#,
    );
    // Still works with multiple import groups
    assert_format(
        r#"
import b from "b";
import a from "a";
const X = 1;
import d from "d";
import c from "c";
export const Y = 2;
import f from "f";
import e from "e";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import a from "a";
import b from "b";
const X = 1;
import c from "c";
import d from "d";
export const Y = 2;
import e from "e";
import f from "f";
"#,
    );
}

// ---

#[test]
fn should_partition_by_newlines() {
    assert_format(
        r#"
import * as atoms from "./atoms";
import * as organisms from "./organisms";
import * as shared from "./shared";

import { Named } from './folder';
import { AnotherNamed } from './second-folder';
"#,
        r#"{
  "experimentalSortImports": {
    "partitionByNewline": true,
    "newlinesBetween": false
  }
}"#,
        r#"
import * as atoms from "./atoms";
import * as organisms from "./organisms";
import * as shared from "./shared";

import { Named } from "./folder";
import { AnotherNamed } from "./second-folder";
"#,
    );
    // Extra newlines are already removed before sorting
    assert_format(
        r#"
import * as atoms from "./atoms";
import * as organisms from "./organisms";
import * as shared from "./shared";


import { Named } from './folder';
import { AnotherNamed } from './second-folder';
"#,
        r#"{
  "experimentalSortImports": {
    "partitionByNewline": true,
    "newlinesBetween": false
  }
}"#,
        r#"
import * as atoms from "./atoms";
import * as organisms from "./organisms";
import * as shared from "./shared";

import { Named } from "./folder";
import { AnotherNamed } from "./second-folder";
"#,
    );
    // More partitions
    assert_format(
        r#"
import D from "d";

import C from "c";

import B from "b";

import A from "a";
"#,
        r#"{
  "experimentalSortImports": {
    "partitionByNewline": true,
    "newlinesBetween": false
  }
}"#,
        r#"
import D from "d";

import C from "c";

import B from "b";

import A from "a";
"#,
    );
    // Ensure comments adjacent to imports stay with their import
    assert_format(
        r#"
import Y from "y";
// Comment for X
import X from "x";

import B from "b";
// Comment for A
import A from "a";
"#,
        r#"{
  "experimentalSortImports": {
    "partitionByNewline": true,
    "newlinesBetween": false
  }
}"#,
        r#"
// Comment for X
import X from "x";
import Y from "y";

// Comment for A
import A from "a";
import B from "b";
"#,
    );
}

#[test]
fn should_partition_by_comment() {
    assert_format(
        r#"
import Y from "y";
import X from "x";
// PARTITION
import B from "b";
import A from "a";
"#,
        r#"{ "experimentalSortImports": { "partitionByComment": true } }"#,
        r#"
import X from "x";
import Y from "y";
// PARTITION
import A from "a";
import B from "b";
"#,
    );
    // Ensure comments with different styles are recognized
    assert_format(
        r"
/* Partition Comment */
// Part: A
import d from './d'
// Part: B
import aaa from './aaa'
import c from './c'
import bb from './bb'
/* Other */
import e from './e'
",
        r#"{
  "experimentalSortImports": {
    "partitionByComment": true
  },
  "singleQuote": true,
  "semi": false
}"#,
        r"
/* Partition Comment */
// Part: A
import d from './d'
// Part: B
import aaa from './aaa'
import bb from './bb'
import c from './c'
/* Other */
import e from './e'
",
    );
    // Multiple comment lines
    assert_format(
        r#"
import C from "c";
// Comment 1
// Comment 2
import B from "b";
import A from "a";
"#,
        r#"{ "experimentalSortImports": { "partitionByComment": true } }"#,
        r#"
import C from "c";
// Comment 1
// Comment 2
import A from "a";
import B from "b";
"#,
    );
}

#[test]
fn should_partition_by_both_newlines_and_comments() {
    assert_format(
        r#"
import X from "x";

import Z from "z";
// PARTITION
import C from "c";

import B from "b";
"#,
        r#"{
  "experimentalSortImports": {
    "partitionByNewline": true,
    "partitionByComment": true,
    "newlinesBetween": false
  }
}"#,
        r#"
import X from "x";

import Z from "z";
// PARTITION
import C from "c";

import B from "b";
"#,
    );
    assert_format(
        r#"
import C from "c";

// Comment
import B from "b";
import A from "a";
"#,
        r#"{
  "experimentalSortImports": {
    "partitionByNewline": true,
    "partitionByComment": true,
    "newlinesBetween": false
  }
}"#,
        r#"
import C from "c";

// Comment
import A from "a";
import B from "b";
"#,
    );
    assert_format(
        r#"
import C from "c";
// Comment

import B from "b";
import A from "a";
"#,
        r#"{
  "experimentalSortImports": {
    "partitionByNewline": true,
    "partitionByComment": true,
    "newlinesBetween": false
  }
}"#,
        r#"
import C from "c";
// Comment

import A from "a";
import B from "b";
"#,
    );
}

// ---

#[test]
fn should_sort_by_order() {
    // Z-A (natural order reversed)
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        r#"{ "experimentalSortImports": { "order": "desc" } }"#,
        r#"
import { log10 } from "./log10";
import { log2 } from "./log2";
import { log1p } from "./log1p";
import { log } from "./log";
"#,
    );
    // A-Z - default (natural order)
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        r#"{ "experimentalSortImports": { "order": "asc" } }"#,
        r#"
import { log } from "./log";
import { log1p } from "./log1p";
import { log2 } from "./log2";
import { log10 } from "./log10";
"#,
    );
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { log } from "./log";
import { log1p } from "./log1p";
import { log2 } from "./log2";
import { log10 } from "./log10";
"#,
    );
}

// ---

#[test]
fn should_sort_side_effects() {
    // Side effect imports stay in their original positions by default
    assert_format(
        r#"
import c from "c";
import b from "b";
import "s";
import a from "a";
import z from "z";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import a from "a";
import b from "b";
import "s";
import c from "c";
import z from "z";
"#,
    );
    assert_format(
        r#"
import y from "y";
import "z";
import "x";
import a from "a";
"#,
        r#"{ "experimentalSortImports": { "sortSideEffects": false } }"#,
        r#"
import a from "a";
import "z";
import "x";
import y from "y";
"#,
    );
    // Keep original order
    assert_format(
        r#"
import "c";
import "bb";
import "aaa";
"#,
        r#"{ "experimentalSortImports": { "sortSideEffects": false } }"#,
        r#"
import "c";
import "bb";
import "aaa";
"#,
    );
    // When `sort_side_effects: true`, all imports are sorted
    assert_format(
        r#"
import y from "y";
import a from "a";
import "z";
import "x";
"#,
        r#"{ "experimentalSortImports": { "sortSideEffects": true } }"#,
        r#"
import a from "a";
import "x";
import y from "y";
import "z";
"#,
    );
    assert_format(
        r#"
import "c";
import "bb";
import "aaa";
"#,
        r#"{ "experimentalSortImports": { "sortSideEffects": true } }"#,
        r#"
import "aaa";
import "bb";
import "c";
"#,
    );
    assert_format(
        r#"
import "./index.css"
import "./animate.css"
import "./reset.css"

"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import "./index.css";
import "./animate.css";
import "./reset.css";
"#,
    );
}

// ---

#[test]
fn should_sort_with_ignore_case_option() {
    // Case-insensitive (ignore_case: true by default)
    assert_format(
        r#"
import { A } from "a";
import { b } from "B";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import { A } from "a";
import { b } from "B";
"#,
    );
    // "a" and "A" are treated as the same, maintaining original order
    assert_format(
        r#"
import x from "A";
import y from "a";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import x from "A";
import y from "a";
"#,
    );
    // Mixed case sorting with ignore_case: true
    assert_format(
        r#"
import { z } from "Z";
import { b } from "B";
import { a } from "a";
"#,
        r#"{ "experimentalSortImports": { "ignoreCase": true } }"#,
        r#"
import { a } from "a";
import { b } from "B";
import { z } from "Z";
"#,
    );
    // Case-sensitive, lowercase comes after uppercase in ASCII
    assert_format(
        r#"
import { a } from "a";
import { B } from "B";
"#,
        r#"{ "experimentalSortImports": { "ignoreCase": false } }"#,
        r#"
import { B } from "B";
import { a } from "a";
"#,
    );
    // Capital A vs lowercase a
    assert_format(
        r#"
import x from "a";
import y from "A";
"#,
        r#"{ "experimentalSortImports": { "ignoreCase": false } }"#,
        r#"
import y from "A";
import x from "a";
"#,
    );
    // Multiple imports with mixed case
    assert_format(
        r#"
import { z } from "z";
import { B } from "B";
import { a } from "a";
import { Z } from "Z";
"#,
        r#"{ "experimentalSortImports": { "ignoreCase": false } }"#,
        r#"
import { B } from "B";
import { Z } from "Z";
import { a } from "a";
import { z } from "z";
"#,
    );
}

// ---

#[test]
fn should_support_internal_pattern_option() {
    assert_format(
        r##"
import type { T } from "a";
import { a } from "a";
import type { S } from "#b";
import c from "#c";
import { b1, b2 } from "#b";
import { d } from "../d";
"##,
        r##"{ "experimentalSortImports": { "internalPattern": ["#"] } }"##,
        r##"
import type { T } from "a";

import { a } from "a";

import type { S } from "#b";

import { b1, b2 } from "#b";
import c from "#c";

import { d } from "../d";
"##,
    );
}

// ---

#[test]
fn should_groups_and_sorts_by_type_and_source() {
    assert_format(
        r#"
import { c1, c2, c3, c4 } from "c";
import { e2 } from "e/b";
import { e1 } from "e/a";
import path from "path";

import { b1, b2 } from "~/b";
import type { I } from "~/i";
import type { D } from "./d";
import fs from "fs";
import { c1 } from "~/c";
import { i1, i2, i3 } from "~/i";

import type { A } from ".";
import type { F } from "../f";
import h from "../../h";
import type { H } from "./index.d.ts";

import a from ".";
import type { T } from "t";
import "./style.css";
import { j } from "../j";
import { K, L, M } from "../k";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import type { T } from "t";

import { c1, c2, c3, c4 } from "c";
import { e1 } from "e/a";
import { e2 } from "e/b";
import fs from "fs";
import path from "path";

import type { I } from "~/i";

import { b1, b2 } from "~/b";
import { c1 } from "~/c";
import { i1, i2, i3 } from "~/i";

import type { A } from ".";
import type { F } from "../f";
import type { D } from "./d";
import type { H } from "./index.d.ts";

import a from ".";
import h from "../../h";
import "./style.css";
import { j } from "../j";
import { K, L, M } from "../k";
"#,
    );
    // Input is already in the correct order, should remain unchanged
    assert_format(
        r#"
import type { T } from "t";

import { c1, c2, c3, c4 } from "c";
import { e1 } from "e/a";
import { e2 } from "e/b";
import fs from "fs";
import path from "path";

import type { I } from "~/i";

import { b1, b2 } from "~/b";
import { c1 } from "~/c";
import { i1, i2, i3 } from "~/i";

import type { A } from ".";
import type { F } from "../f";
import type { D } from "./d";
import type { H } from "./index.d.ts";

import a from ".";
import h from "../../h";
import "./style.css";
import { j } from "../j";
import { K, L, M } from "../k";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import type { T } from "t";

import { c1, c2, c3, c4 } from "c";
import { e1 } from "e/a";
import { e2 } from "e/b";
import fs from "fs";
import path from "path";

import type { I } from "~/i";

import { b1, b2 } from "~/b";
import { c1 } from "~/c";
import { i1, i2, i3 } from "~/i";

import type { A } from ".";
import type { F } from "../f";
import type { D } from "./d";
import type { H } from "./index.d.ts";

import a from ".";
import h from "../../h";
import "./style.css";
import { j } from "../j";
import { K, L, M } from "../k";
"#,
    );
    // Ignore comments
    assert_format(
        r#"
import type { T } from "t";

// @ts-expect-error missing types
import { t } from "t";
"#,
        r#"{ "experimentalSortImports": {} }"#,
        r#"
import type { T } from "t";

// @ts-expect-error missing types
import { t } from "t";
"#,
    );
}

#[test]
fn should_support_newlines_between_option() {
    // Test newlines_between: false (no blank lines between groups)
    assert_format(
        r#"
import d from ".";
import { a1, a2, a3 } from "a";
import { c1, c2, c3 } from "~/c";

import type { T } from "t";
import { e1, e2, e3 } from "../../e";

import { b1, b2 } from "~/b";
"#,
        r#"{ "experimentalSortImports": { "newlinesBetween": false } }"#,
        r#"
import type { T } from "t";
import { a1, a2, a3 } from "a";
import { b1, b2 } from "~/b";
import { c1, c2, c3 } from "~/c";
import d from ".";
import { e1, e2, e3 } from "../../e";
"#,
    );

    // Test newlines_between: true (one blank line between groups - default)
    assert_format(
        r#"
import d from ".";
import { a1, a2, a3 } from "a";
import { c1, c2, c3 } from "~/c";

import type { T } from "t";
import { e1, e2, e3 } from "../../e";

import { b1, b2 } from "~/b";
"#,
        r#"{ "experimentalSortImports": { "newlinesBetween": true } }"#,
        r#"
import type { T } from "t";

import { a1, a2, a3 } from "a";

import { b1, b2 } from "~/b";
import { c1, c2, c3 } from "~/c";

import d from ".";
import { e1, e2, e3 } from "../../e";
"#,
    );

    // Test newlines_between: false removes multiple consecutive blank lines
    assert_format(
        r#"
import { A } from "a";


import y from "~/y";
import z from "~/z";

import b from "~/b";
"#,
        r#"{ "experimentalSortImports": { "newlinesBetween": false } }"#,
        r#"
import { A } from "a";
import b from "~/b";
import y from "~/y";
import z from "~/z";
"#,
    );
}

// ---

#[test]
fn should_sort_by_specific_groups() {
    assert_format(
        r#"
import type { T } from "../t";

import type { U } from "~/u";

import type { V } from "v";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": [
        "type",
        ["builtin", "external"],
        "internal",
        ["parent", "sibling", "index"]
    ]
  }
}"#,
        r#"
import type { T } from "../t";
import type { V } from "v";
import type { U } from "~/u";
"#,
    );
    // Style imports in separate group
    assert_format(
        r#"
import { a1, a2 } from "a";

import styles from "../s.css";
import "./t.css";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": [
        "type",
        ["builtin", "external"],
        "internal-type",
        "internal",
        ["parent-type", "sibling-type", "index-type"],
        ["parent", "sibling", "index"],
        "style",
        "unknown"
    ]
  }
}"#,
        r#"
import { a1, a2 } from "a";

import styles from "../s.css";
import "./t.css";
"#,
    );
    // Side-effect imports in separate group
    assert_format(
        r#"
import { A } from "../a";
import { b } from "./b";

import "../c.js";
import "./d";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": [
        "type",
        ["builtin", "external"],
        "internal-type",
        "internal",
        ["parent-type", "sibling-type", "index-type"],
        ["parent", "sibling", "index"],
        "side-effect",
        "unknown"
    ]
  }
}"#,
        r#"
import { A } from "../a";
import { b } from "./b";

import "../c.js";
import "./d";
"#,
    );
    // Builtin type imports in separate group
    assert_format(
        r#"
import type { Server } from "http";

import a from "a";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["builtin-type", "type"]
  }
}"#,
        r#"
import type { Server } from "http";

import a from "a";
"#,
    );
    // Side-effect imports preserve order when sortSideEffects: false
    assert_format(
        r#"
import a from "aaaa";

import "bbb";
import "./cc";
import "../d";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["external", "side-effect", "unknown"],
    "sortSideEffects": false
  }
}"#,
        r#"
import a from "aaaa";

import "bbb";
import "./cc";
import "../d";
"#,
    );
    // preserves side-effect import order when sorting disabled
    assert_format(
        r#"
import "./cc";
import "bbb";
import e from "e";
import a from "aaaa";
import "../d";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["external", "side-effect", "unknown"],
    "sortSideEffects": false
  }
}"#,
        r#"
import a from "aaaa";
import e from "e";

import "./cc";
import "bbb";
import "../d";
"#,
    );
    assert_format(
        r#"
import "c";
import "bb";
import "aaa";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["external", "side-effect", "unknown"],
    "sortSideEffects": true
  }
}"#,
        r#"
import "aaa";
import "bb";
import "c";
"#,
    );
    // Side-effects stay in original position, only non-side-effects are sorted
    assert_format(
        r#"
import "./z-side-effect.scss";
import b from "./b";
import "./b-side-effect";
import "./g-side-effect.css";
import "./a-side-effect";
import a from "./a";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["unknown"]
  }
}"#,
        r#"
import "./z-side-effect.scss";
import a from "./a";
import "./b-side-effect";
import "./g-side-effect.css";
import "./a-side-effect";
import b from "./b";
"#,
    );
    // Groups side-effect imports together without sorting them
    assert_format(
        r#"
import "./z-side-effect.scss";
import b from "./b";
import "./b-side-effect";
import "./g-side-effect.css";
import "./a-side-effect";
import a from "./a";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["side-effect", "unknown"]
  }
}"#,
        r#"
import "./z-side-effect.scss";
import "./b-side-effect";
import "./g-side-effect.css";
import "./a-side-effect";

import a from "./a";
import b from "./b";
"#,
    );
    // Groups side-effect and style imports together in same group without sorting
    assert_format(
        r#"
import "./z-side-effect.scss";
import b from "./b";
import "./b-side-effect";
import "./g-side-effect.css";
import "./a-side-effect";
import a from "./a";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": [["side-effect", "side-effect-style"], "unknown"]
  }
}"#,
        r#"
import "./z-side-effect.scss";
import "./b-side-effect";
import "./g-side-effect.css";
import "./a-side-effect";

import a from "./a";
import b from "./b";
"#,
    );
    // Separates side-effect and style imports into distinct groups without sorting
    assert_format(
        r#"
import "./z-side-effect.scss";
import b from "./b";
import "./b-side-effect";
import "./g-side-effect.css";
import "./a-side-effect";
import a from "./a";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["side-effect", "side-effect-style", "unknown"]
  }
}"#,
        r#"
import "./b-side-effect";
import "./a-side-effect";

import "./z-side-effect.scss";
import "./g-side-effect.css";

import a from "./a";
import b from "./b";
"#,
    );
    // Groups style side-effect imports separately without sorting
    assert_format(
        r#"
import "./z-side-effect";
import b from "./b";
import "./b-side-effect.scss";
import "./g-side-effect";
import "./a-side-effect.css";
import a from "./a";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["side-effect-style", "unknown"]
  }
}"#,
        r#"
import "./z-side-effect";
import "./b-side-effect.scss";
import "./a-side-effect.css";

import "./g-side-effect";
import a from "./a";
import b from "./b";
"#,
    );
    // handles newlines and comments after fixes
    assert_format(
        r#"
import { b } from "b";
import { a } from "./a"; // Comment after

import { c } from "c";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["unknown", "external"],
    "newlinesBetween": true
  }
}"#,
        r#"
import { a } from "./a"; // Comment after

import { b } from "b";
import { c } from "c";
"#,
    );
    // prioritizes index types over sibling types
    assert_format(
        r#"
import type a from "./a";

import type b from "./index";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["index-type", "sibling-type"]
  }
}"#,
        r#"
import type b from "./index";

import type a from "./a";
"#,
    );
    // prioritizes specific type selectors over generic type group
    assert_format(
        r#"
import type a from "../a";

import type b from "./b";
import type c from "./index";
import type d from "d";
import type e from "timers";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": [
      [
        "index-type",
        "internal-type",
        "external-type",
        "sibling-type",
        "builtin-type"
      ],
      "type"
    ]
  }
}"#,
        r#"
import type b from "./b";
import type c from "./index";
import type d from "d";
import type e from "timers";

import type a from "../a";
"#,
    );
    // prioritizes index imports over sibling imports
    assert_format(
        r#"
import a from "./a";

import b from "./index";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["index", "sibling"]
  }
}"#,
        r#"
import b from "./index";

import a from "./a";
"#,
    );
    // prioritizes style side-effects over generic side-effects
    assert_format(
        r#"
import "something";

import "style.css";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["side-effect-style", "side-effect"]
  }
}"#,
        r#"
import "style.css";

import "something";
"#,
    );
    // prioritizes side-effects over style imports with default exports
    assert_format(
        r#"
import style from "style.css";

import "something";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["side-effect", "style"]
  }
}"#,
        r#"
import "something";

import style from "style.css";
"#,
    );
    // prioritizes external imports over generic import group
    assert_format(
        r#"
import a from "./a";

import b from "b";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["external", "import"]
  }
}"#,
        r#"
import b from "b";

import a from "./a";
"#,
    );
    // prioritizes side-effect imports over value imports
    assert_format(
        r#"
import f from "f";

import "./z";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["side-effect-import", "external", "value-import"],
    "sortSideEffects": true
  }
}"#,
        r#"
import "./z";

import f from "f";
"#,
    );
    // prioritizes default imports over named imports
    assert_format(
        r#"
import f from "f";

import z, { z } from "./z";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["default-import", "external", "named-import"]
  }
}"#,
        r#"
import z, { z } from "./z";

import f from "f";
"#,
    );
    // prioritizes wildcard imports over named imports
    assert_format(
        r#"
import f from "f";

import * as z from "./z";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["wildcard-import", "external", "named-import"]
  }
}"#,
        r#"
import * as z from "./z";

import f from "f";
"#,
    );
    // treats @ symbol pattern as internal imports
    assert_format(
        r#"
import { b } from "b";
import { a } from "@/a";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["external", "internal"],
    "newlinesBetween": true
  }
}"#,
        r#"
import { b } from "b";

import { a } from "@/a";
"#,
    );
    // Supports subpath
    assert_format(
        r##"
import a from "../a";
import b from "./b";
import subpath from "#subpath";
import e from "timers";
import c from "./index";
import d from "d";

import style from "style.css";
"##,
        r#"{
  "experimentalSortImports": {
    "groups": [
        "style",
        [
          "index",
          "internal",
          "subpath",
          "external",
          "sibling",
          "builtin",
          "parent"
        ]
    ]
  }
}"#,
        r##"
import style from "style.css";

import subpath from "#subpath";
import a from "../a";
import b from "./b";
import c from "./index";
import d from "d";
import e from "timers";
"##,
    );
    // Empty groups
    assert_format(
        r#"
import d from "d";
import a from "a";
import * as c from "c";
import { b1, type b2, b3 as b33 } from "b";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": []
  }
}"#,
        r#"
import a from "a";
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import d from "d";
"#,
    );
    assert_format(
        r#"
import d from "d";
import a from "a";
import * as c from "c";
import { b1, type b2, b3 as b33 } from "b";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": [[], []]
  }
}"#,
        r#"
import a from "a";
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import d from "d";
"#,
    );
    // Node.js built-in modules with node: prefix are classified as builtin group
    assert_format(
        r#"
import { writeFile } from "node:fs/promises";
import { useEffect } from "react";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["builtin", "external"]
  }
}"#,
        r#"
import { writeFile } from "node:fs/promises";

import { useEffect } from "react";
"#,
    );
    // Internal pattern side-effects are correctly classified by group priority
    assert_format(
        r#"
import { useClient } from "~/hooks/useClient";
import "~/data";
import "~/css/globals.css";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["internal", "side-effect-style", "side-effect"]
  }
}"#,
        r#"
import { useClient } from "~/hooks/useClient";

import "~/css/globals.css";

import "~/data";
"#,
    );
    // Empty named imports are treated as regular imports not side-effects
    assert_format(
        r#"
import {} from "node:os";
import sqlite from "node:sqlite";
import { describe, test } from "node:test";
import { c } from "c";
import "node:os";
"#,
        r#"{
  "experimentalSortImports": {
    "groups": ["builtin", "external", "side-effect"]
  }
}"#,
        r#"
import {} from "node:os";
import sqlite from "node:sqlite";
import { describe, test } from "node:test";

import { c } from "c";

import "node:os";
"#,
    );
}
