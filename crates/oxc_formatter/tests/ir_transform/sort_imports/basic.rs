use super::super::assert_format;

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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
  "sortImports": {},
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
        r#"{ "sortImports": { "partitionByComment": true } }"#,
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
  "sortImports": {
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
        r#"{ "sortImports": { "partitionByComment": true } }"#,
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
fn should_not_attach_comment_if_newline_between_import() {
    assert_format(
        r#"
// THIS IS NOT MOVED

import b from "b";
// AND ALSO THIS IS NOT MOVED

import a from "a";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
// THIS IS NOT MOVED

import a from "a";
// AND ALSO THIS IS NOT MOVED
import b from "b";
"#,
    );
    assert_format(
        r#"
// THIS CAN BE MOVED
// BECAUSE OF THIS
import b from "b";
// AND ALSO THIS CAN BE MOVED
import a from "a";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
// AND ALSO THIS CAN BE MOVED
import a from "a";
// THIS CAN BE MOVED
// BECAUSE OF THIS
import b from "b";
"#,
    );
    assert_format(
        r#"
// THIS IS A GENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import { apiClient } from "../../apiClient";
import { useMutation, UseMutationOptions } from "@tanstack/react-query";
import { ApiErrorDisconnected, ApiError } from "../../apiErrors";
import { validateResponse } from "../../validateResponse";
// Use zod

import { z } from "zod";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
// THIS IS A GENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
import { z } from "zod";

import { apiClient } from "../../apiClient";
import { ApiErrorDisconnected, ApiError } from "../../apiErrors";
// Use zod
import { validateResponse } from "../../validateResponse";
"#,
    );
    assert_format(
        r#"
// THIS IS A GENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// Use zod

import { z } from "zod";
"#,
        r#"{
  "sortImports": {
    "partitionByNewline": true,
    "newlinesBetween": false
  }
        }"#,
        r#"
// THIS IS A GENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// Use zod

import { z } from "zod";
"#,
    );
    assert_format(
        r#"
// THIS IS A GENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// Use zod

import { z } from "zod";
"#,
        r#"{
  "sortImports": {
    "newlinesBetween": false
  }
        }"#,
        r#"
// THIS IS A GENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// Use zod
import { z } from "zod";
"#,
    );
    assert_format(
        r#"
// FIXED

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// C1

// C2

import { z } from "zod";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
// FIXED

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// C1
// C2
import { z } from "zod";
"#,
    );
    assert_format(
        r#"
// FIXED

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// C1

// C2

import { z } from "zod";
"#,
        r#"{
  "sortImports": {
    "newlinesBetween": false,
    "partitionByNewline": true
  }
        }"#,
        r#"
// FIXED

import { useMutation, UseMutationOptions } from "@tanstack/react-query";
// C1

// C2

import { z } from "zod";
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
        r#"{ "sortImports": { "order": "desc" } }"#,
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
        r#"{ "sortImports": { "order": "asc" } }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": { "sortSideEffects": false } }"#,
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
        r#"{ "sortImports": { "sortSideEffects": false } }"#,
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
        r#"{ "sortImports": { "sortSideEffects": true } }"#,
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
        r#"{ "sortImports": { "sortSideEffects": true } }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": {} }"#,
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
        r#"{ "sortImports": { "ignoreCase": true } }"#,
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
        r#"{ "sortImports": { "ignoreCase": false } }"#,
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
        r#"{ "sortImports": { "ignoreCase": false } }"#,
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
        r#"{ "sortImports": { "ignoreCase": false } }"#,
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
        r##"{ "sortImports": { "internalPattern": ["#"] } }"##,
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
fn should_sort_with_multiline_comments_attached_to_each_import() {
    assert_format(
        r#"
/*
 * hi
 */
import cn from "classnames"
import type { Hello } from "pkg"
"#,
        r#"{ "sortImports": {} }"#,
        r#"
/*
 * hi
 */
import cn from "classnames";
import type { Hello } from "pkg";
"#,
    );
    assert_format(
        r#"
/*
 * hi
 */
import cn from "classnames"
import { Hello } from "pkg"
"#,
        r#"{ "sortImports": {} }"#,
        r#"
/*
 * hi
 */
import cn from "classnames";
import { Hello } from "pkg";
"#,
    );
    // Each multiline comment attached to its own import
    assert_format(
        r#"
/*
 * for b
 */
import b from "b"
/*
 * for a
 */
import a from "a"
"#,
        r#"{ "sortImports": {} }"#,
        r#"
/*
 * for a
 */
import a from "a";
/*
 * for b
 */
import b from "b";
"#,
    );

    // Consecutive multiline comments before imports
    // Comments are attached to the immediately following import (import b)
    assert_format(
        r#"
/*
 * comment1
 */
/*
 * comment2
 */
import b from "b"
import a from "a"
"#,
        r#"{ "sortImports": {} }"#,
        r#"
import a from "a";
/*
 * comment1
 */
/*
 * comment2
 */
import b from "b";
"#,
    );

    // Multiline comment separated by empty line (partitioned)
    assert_format(
        r#"
/*
 * comment
 */

import b from "b"
import a from "a"
"#,
        r#"{ "sortImports": {} }"#,
        r#"
/*
 * comment
 */

import a from "a";
import b from "b";
"#,
    );

    // Mix of single-line and multiline block comments
    assert_format(
        r#"
/* single */ import b from "b"
/*
 * multiline
 */
import a from "a"
"#,
        r#"{ "sortImports": {} }"#,
        r#"
/*
 * multiline
 */
import a from "a";
/* single */ import b from "b";
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
        r#"{ "sortImports": { "newlinesBetween": false } }"#,
        r#"
import { a1, a2, a3 } from "a";
import type { T } from "t";
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
        r#"{ "sortImports": { "newlinesBetween": true } }"#,
        r#"
import { a1, a2, a3 } from "a";
import type { T } from "t";

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
        r#"{ "sortImports": { "newlinesBetween": false } }"#,
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
fn should_sort_multiline_import_containing_comment() {
    // "a" -> "b" -> "c"
    assert_format(
        r#"
import c from "c";
import b from "b";
import {
  type Data,
  // this comment makes the source of this import become "Data"
  xyz,
} from "a";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
import {
  type Data,
  // this comment makes the source of this import become "Data"
  xyz,
} from "a";
import b from "b";
import c from "c";
"#,
    );
}

// ---

#[test]
fn issue_17788() {
    // Should not panic
    assert_format(
        r"
`/*`
",
        r#"{ "sortImports": {} }"#,
        r"
`/*`;
",
    );
    // Should not panic
    assert_format(
        r"
acc[path] = `src/${path
.split('/')
.map((s) => s[0].toUpperCase() + s.slice(1))
.join('/')}/*`;
",
        r#"{ "sortImports": {} }"#,
        r#"
acc[path] = `src/${path
  .split("/")
  .map((s) => s[0].toUpperCase() + s.slice(1))
  .join("/")}/*`;
"#,
    );
}
