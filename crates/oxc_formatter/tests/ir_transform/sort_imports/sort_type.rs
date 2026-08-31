use super::super::assert_format;

#[test]
fn natural_is_the_default() {
    assert_format(
        r#"
import { log10 } from "./log10";
import { log2 } from "./log2";
import { log } from "./log";
"#,
        r#"{ "sort": { "imports": { "type": "natural" } } }"#,
        r#"
import { log } from "./log";
import { log2 } from "./log2";
import { log10 } from "./log10";
"#,
    );
}

#[test]
fn alphabetical_is_code_point_order() {
    assert_format(
        r#"
import { log10 } from "./log10";
import { log2 } from "./log2";
import { log } from "./log";
"#,
        r#"{ "sort": { "imports": { "type": "alphabetical" } } }"#,
        r#"
import { log } from "./log";
import { log10 } from "./log10";
import { log2 } from "./log2";
"#,
    );
    // With `ignoreCase: false`, uppercase sorts before lowercase (code points).
    assert_format(
        r#"
import { a } from "a";
import { B } from "B";
"#,
        r#"{ "sort": { "imports": { "type": "alphabetical", "ignoreCase": false } } }"#,
        r#"
import { B } from "B";
import { a } from "a";
"#,
    );
    // Default `ignoreCase: true`: keys are lowercased first, so `a` sorts before `B`.
    assert_format(
        r#"
import { B } from "B";
import { a } from "a";
"#,
        r#"{ "sort": { "imports": { "type": "alphabetical" } } }"#,
        r#"
import { a } from "a";
import { B } from "B";
"#,
    );
}

#[test]
fn line_length_sorts_by_printed_width() {
    assert_format(
        r#"
import { ccc } from "./ccc";
import { a } from "./a";
import { bb } from "./bb";
"#,
        r#"{ "sort": { "imports": { "type": "line-length" } } }"#,
        r#"
import { a } from "./a";
import { bb } from "./bb";
import { ccc } from "./ccc";
"#,
    );
    assert_format(
        r#"
import { a } from "./a";
import { ccc } from "./ccc";
import { bb } from "./bb";
"#,
        r#"{ "sort": { "imports": { "type": "line-length", "order": "desc" } } }"#,
        r#"
import { ccc } from "./ccc";
import { bb } from "./bb";
import { a } from "./a";
"#,
    );
}

#[test]
fn unsorted_only_groups() {
    // Builtins before externals (default groups), but source order inside each group.
    assert_format(
        r#"
import z from "z";
import a from "a";
import path from "path";
import fs from "fs";
"#,
        r#"{ "sort": { "imports": { "type": "unsorted" } } }"#,
        r#"
import path from "path";
import fs from "fs";

import z from "z";
import a from "a";
"#,
    );
}

#[test]
fn special_characters_trim_and_remove() {
    // keep: `_c` < `b` ('_' precedes letters)
    assert_format(
        r#"
import b from "b";
import c from "_c";
"#,
        r#"{ "sort": { "imports": { "specialCharacters": "keep" } } }"#,
        r#"
import c from "_c";
import b from "b";
"#,
    );
    // trim: `_c` compares as `c`, so `b` first
    assert_format(
        r#"
import c from "_c";
import b from "b";
"#,
        r#"{ "sort": { "imports": { "specialCharacters": "trim" } } }"#,
        r#"
import b from "b";
import c from "_c";
"#,
    );
    // remove: `a.c` compares as `ac`, after `ab`
    assert_format(
        r#"
import ac from "a.c";
import ab from "ab";
"#,
        r#"{ "sort": { "imports": { "specialCharacters": "remove" } } }"#,
        r#"
import ab from "ab";
import ac from "a.c";
"#,
    );
}

#[test]
fn fallback_sort_breaks_ties() {
    // Same source twice: primary compare is Equal.
    // Default fallback (`unsorted`) keeps source order.
    assert_format(
        r#"
import { zz } from "./a";
import { b } from "./a";
"#,
        r#"{ "sort": { "imports": {} } }"#,
        r#"
import { zz } from "./a";
import { b } from "./a";
"#,
    );
    assert_format(
        r#"
import { zz } from "./a";
import { b } from "./a";
"#,
        r#"{ "sort": { "imports": { "fallbackSort": { "type": "line-length" } } } }"#,
        r#"
import { b } from "./a";
import { zz } from "./a";
"#,
    );
    assert_format(
        r#"
import { b } from "./a";
import { zz } from "./a";
"#,
        r#"{ "sort": { "imports": { "fallbackSort": { "type": "line-length", "order": "desc" } } } }"#,
        r#"
import { zz } from "./a";
import { b } from "./a";
"#,
    );
}
