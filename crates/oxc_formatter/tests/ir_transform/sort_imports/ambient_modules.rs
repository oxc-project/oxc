use super::super::assert_format;

#[test]
fn should_sort_imports_inside_ambient_module() {
    assert_format(
        r#"
declare module "foo" {
  import C from "c";
  import A from "a";
  import B from "b";
  export const x: number;
}
"#,
        r#"{ "sortImports": {} }"#,
        r#"
declare module "foo" {
  import A from "a";
  import B from "b";
  import C from "c";
  export const x: number;
}
"#,
    );
}

// ---

#[test]
fn should_sort_imports_inside_and_outside_ambient_module_independently() {
    // Top-level imports and imports inside the ambient module form independent chunks:
    // each is sorted within its own scope, and reordering one must not affect the other.
    assert_format(
        r#"
import OuterC from "outer-c";
import OuterA from "outer-a";
import OuterB from "outer-b";

declare module "foo" {
  import InnerZ from "inner-z";
  import InnerX from "inner-x";
  import InnerY from "inner-y";
  export const x: number;
}

import OuterE from "outer-e";
import OuterD from "outer-d";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
import OuterA from "outer-a";
import OuterB from "outer-b";
import OuterC from "outer-c";

declare module "foo" {
  import InnerX from "inner-x";
  import InnerY from "inner-y";
  import InnerZ from "inner-z";
  export const x: number;
}

import OuterD from "outer-d";
import OuterE from "outer-e";
"#,
    );
}

// ---

#[test]
fn should_sort_imports_in_multiple_ambient_modules_independently() {
    // Each ambient module's imports form an independent chunk: they are sorted
    // within the module's own body and not merged with imports from other modules.
    assert_format(
        r#"
declare module "first" {
  import C from "c";
  import A from "a";
  export const first_x: number;
}

declare module "second" {
  import Z from "z";
  import M from "m";
  export const second_x: number;
}

declare module "third" {
  import Q from "q";
  import B from "b";
  import N from "n";
  export const third_x: number;
}
"#,
        r#"{ "sortImports": {} }"#,
        r#"
declare module "first" {
  import A from "a";
  import C from "c";
  export const first_x: number;
}

declare module "second" {
  import M from "m";
  import Z from "z";
  export const second_x: number;
}

declare module "third" {
  import B from "b";
  import N from "n";
  import Q from "q";
  export const third_x: number;
}
"#,
    );
}

// ---

#[test]
fn should_sort_imports_across_ambient_module_and_top_level_with_non_imports_in_between() {
    // Ambient modules form chunk boundaries for top-level imports too:
    // top-level imports before the `declare module` form one chunk, and top-level
    // imports after it form a separate chunk, just as they would around any other
    // non-import statement.
    assert_format(
        r#"
import C from "c";
import A from "a";

declare module "first" {
  import Inner1B from "inner1-b";
  import Inner1A from "inner1-a";
  export const x: number;
}

import D from "d";
import B from "b";

declare module "second" {
  import Inner2B from "inner2-b";
  import Inner2A from "inner2-a";
  export const y: number;
}

import F from "f";
import E from "e";
"#,
        r#"{ "sortImports": {} }"#,
        r#"
import A from "a";
import C from "c";

declare module "first" {
  import Inner1A from "inner1-a";
  import Inner1B from "inner1-b";
  export const x: number;
}

import B from "b";
import D from "d";

declare module "second" {
  import Inner2A from "inner2-a";
  import Inner2B from "inner2-b";
  export const y: number;
}

import E from "e";
import F from "f";
"#,
    );
}
