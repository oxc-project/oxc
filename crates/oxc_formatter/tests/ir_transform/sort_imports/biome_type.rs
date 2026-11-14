use super::super::assert_format;

// Expected outputs match `biome check --write` (Biome 2.2.4) with `organizeImports` and no groups.

#[test]
fn should_sort_path_separator_first() {
    assert_format(
        r#"
import a from "pkg-extra/a";
import b from "pkg/b";
import c from "pkg.config";
import d from "pkg/a";
"#,
        r#"{ "sortImports": { "type": "biome" } }"#,
        r#"
import d from "pkg/a";
import b from "pkg/b";
import c from "pkg.config";
import a from "pkg-extra/a";
"#,
    );
}

#[test]
fn should_sort_uppercase_before_lowercase() {
    let input = r#"
import t from "./tab";
import p from "./parts";
import h from "./THead";
import o from "./PopOver";
import s from "./Th";
"#;
    let expected = r#"
import o from "./PopOver";
import p from "./parts";
import h from "./THead";
import s from "./Th";
import t from "./tab";
"#;
    assert_format(input, r#"{ "sortImports": { "type": "biome" } }"#, expected);
    // `ignoreCase` has no effect
    assert_format(
        input,
        r#"{ "sortImports": { "type": "biome", "ignoreCase": false } }"#,
        expected,
    );
}

#[test]
fn should_sort_paths_by_proximity() {
    assert_format(
        r#"
import a from "./a";
import i from ".";
import p from "..";
import b from "../../b";
import pa from "../a";
import c from "./b/c";
"#,
        r#"{ "sortImports": { "type": "biome" } }"#,
        r#"
import b from "../../b";
import p from "..";
import pa from "../a";
import i from ".";
import a from "./a";
import c from "./b/c";
"#,
    );
}

#[test]
fn should_sort_by_source_kind_within_a_group() {
    assert_format(
        r##"
import l from "~/lib";
import u from "#utils";
import r from "react";
import s from "@scope/pkg";
import fs from "node:fs";
import x from "https://esm.sh/x";
"##,
        r#"{ "sortImports": { "type": "biome", "groups": [["builtin", "external", "internal", "subpath"]] } }"#,
        r##"
import x from "https://esm.sh/x";
import fs from "node:fs";
import s from "@scope/pkg";
import r from "react";
import u from "#utils";
import l from "~/lib";
"##,
    );
}

#[test]
fn should_sort_type_imports_first_for_the_same_source() {
    assert_format(
        r#"
import { a } from "x";
import type { A } from "x";
"#,
        r#"{ "sortImports": { "type": "biome" } }"#,
        r#"
import type { A } from "x";
import { a } from "x";
"#,
    );
}

#[test]
fn should_sort_numbers_naturally() {
    assert_format(
        r#"
import a from "./item10";
import b from "./item9";
import c from "./item1";
"#,
        r#"{ "sortImports": { "type": "biome" } }"#,
        r#"
import c from "./item1";
import b from "./item9";
import a from "./item10";
"#,
    );
}
