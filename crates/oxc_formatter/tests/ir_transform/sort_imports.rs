use super::assert_format;
use oxc_formatter::{FormatOptions, QuoteStyle, Semicolons, SortImports, SortOrder};

#[test]
fn should_not_sort_by_default() {
    assert_format(
        r#"
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import type d from "d";
import a from "a";
"#,
        &FormatOptions { experimental_sort_imports: None, ..Default::default() },
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
import type d from "d";
import a from "a";
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
        r#"
import a from "a";
import { b1, type b2, b3 as b33 } from "b";
import * as c from "c";
import type d from "d";
"#,
    );
    // Alphabetical ASC order by default
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
    );
    // Dynamic imports should not affect sorting
    assert_format(
        r#"
import c from "c";
import b from "b";
import("a");
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
        r#"
import b from "b";
import c from "c";
import("a");
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            quote_style: QuoteStyle::Single,
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_newline: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_newline: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_newline: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_newline: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_comment: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_comment: true,
                ..Default::default()
            }),
            quote_style: QuoteStyle::Single,
            semicolons: Semicolons::AsNeeded,
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_comment: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_newline: true,
                partition_by_comment: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_newline: true,
                partition_by_comment: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                partition_by_newline: true,
                partition_by_comment: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
    // Z-A
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                order: SortOrder::Desc,
                ..Default::default()
            }),
            ..Default::default()
        },
        r#"
import { log2 } from "./log2";
import { log1p } from "./log1p";
import { log10 } from "./log10";
import { log } from "./log";
"#,
    );
    // A-Z - default
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                order: SortOrder::Asc,
                ..Default::default()
            }),
            ..Default::default()
        },
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
    );
    assert_format(
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log1p } from "./log1p";
import { log2 } from "./log2";
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
        r#"
import a from "a";
import b from "b";
import "s";
import c from "c";
import z from "z";
"#,
    );
    // Side effect imports stay in their original positions if `sort_side_effects: false`
    assert_format(
        r#"
import c from "c";
import b from "b";
import "s";
import a from "a";
import z from "z";
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                sort_side_effects: false,
                ..Default::default()
            }),
            ..Default::default()
        },
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
import "c";
import "bb";
import "aaa";
"#,
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                sort_side_effects: false,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                sort_side_effects: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                sort_side_effects: true,
                ..Default::default()
            }),
            ..Default::default()
        },
        r#"
import "aaa";
import "bb";
import "c";
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports::default()),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                ignore_case: true,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                ignore_case: false,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                ignore_case: false,
                ..Default::default()
            }),
            ..Default::default()
        },
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
        &FormatOptions {
            experimental_sort_imports: Some(SortImports {
                ignore_case: false,
                ..Default::default()
            }),
            ..Default::default()
        },
        r#"
import { B } from "B";
import { Z } from "Z";
import { a } from "a";
import { z } from "z";
"#,
    );
}
