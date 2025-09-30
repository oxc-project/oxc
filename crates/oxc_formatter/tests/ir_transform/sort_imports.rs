use super::assert_format;
use oxc_formatter::{FormatOptions, QuoteStyle, Semicolons, SortImports};

#[test]
fn should_not_sort_if_options_is_none() {
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
fn should_remove_newlines_between_imports() {
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

const a = 1;
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

const a = 1;
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
