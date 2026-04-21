use super::super::assert_format;

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
import { j } from "../j";
import { K, L, M } from "../k";
import "./style.css";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
import fs from "fs";
import path from "path";

import { c1, c2, c3, c4 } from "c";
import { e1 } from "e/a";
import { e2 } from "e/b";
import type { T } from "t";

import { b1, b2 } from "~/b";
import { c1 } from "~/c";
import type { I } from "~/i";
import { i1, i2, i3 } from "~/i";

import type { A } from ".";
import a from ".";
import h from "../../h";
import type { F } from "../f";
import { j } from "../j";
import { K, L, M } from "../k";
import type { D } from "./d";
import type { H } from "./index.d.ts";

import "./style.css";
"#,
    );
    // Input is already in the correct order, should remain unchanged
    assert_format(
        r#"
import fs from "fs";
import path from "path";

import { c1, c2, c3, c4 } from "c";
import { e1 } from "e/a";
import { e2 } from "e/b";
import type { T } from "t";

import { b1, b2 } from "~/b";
import { c1 } from "~/c";
import type { I } from "~/i";
import { i1, i2, i3 } from "~/i";

import type { A } from ".";
import a from ".";
import h from "../../h";
import type { F } from "../f";
import { j } from "../j";
import { K, L, M } from "../k";
import type { D } from "./d";
import type { H } from "./index.d.ts";

import "./style.css";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
import fs from "fs";
import path from "path";

import { c1, c2, c3, c4 } from "c";
import { e1 } from "e/a";
import { e2 } from "e/b";
import type { T } from "t";

import { b1, b2 } from "~/b";
import { c1 } from "~/c";
import type { I } from "~/i";
import { i1, i2, i3 } from "~/i";

import type { A } from ".";
import a from ".";
import h from "../../h";
import type { F } from "../f";
import { j } from "../j";
import { K, L, M } from "../k";
import type { D } from "./d";
import type { H } from "./index.d.ts";

import "./style.css";
"#,
    );
    // Ignore comments
    assert_format(
        r#"
import type { T } from "t";

// @ts-expect-error missing types
import { t } from "t";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
import type { T } from "t";
// @ts-expect-error missing types
import { t } from "t";
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
  "sortImports": {
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
    // Side-effect imports preserve order when sortSideEffects: false
    assert_format(
        r#"
import a from "aaaa";

import "bbb";
import "./cc";
import "../d";
"#,
        r#"{
  "sortImports": {
    "groups": ["external", "side_effect", "unknown"],
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
  "sortImports": {
    "groups": ["external", "side_effect", "unknown"],
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
  "sortImports": {
    "groups": ["external", "side_effect", "unknown"],
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
  "sortImports": {
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
  "sortImports": {
    "groups": ["side_effect", "unknown"]
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
  "sortImports": {
    "groups": [["side_effect", "side_effect_style"], "unknown"]
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
  "sortImports": {
    "groups": ["side_effect", "side_effect_style", "unknown"]
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
  "sortImports": {
    "groups": ["side_effect_style", "unknown"]
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
  "sortImports": {
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
    // prioritizes index imports over sibling imports
    assert_format(
        r#"
import a from "./a";

import b from "./index";
"#,
        r#"{
  "sortImports": {
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
  "sortImports": {
    "groups": ["side_effect_style", "side_effect"]
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
  "sortImports": {
    "groups": ["side_effect", "style"]
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
  "sortImports": {
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
  "sortImports": {
    "groups": ["side_effect-import", "external", "value-import"],
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
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
  "sortImports": {
    "groups": ["internal", "side_effect_style", "side_effect"]
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
  "sortImports": {
    "groups": ["builtin", "external", "side_effect"]
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
