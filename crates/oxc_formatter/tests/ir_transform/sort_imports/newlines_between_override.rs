use super::super::assert_format;

#[test]
fn global_true_with_override_false_suppresses_blank_line() {
    // Global newlinesBetween: true, but override suppresses blank line between
    // value-builtin/value-external and value-parent
    assert_format(
        r#"
import path from "path";
import { foo } from "../foo";
import { bar } from "./bar";
import { baz } from "baz";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": true,
                "groups": [
                    ["value-builtin", "value-external"],
                    { "newlinesBetween": false },
                    "value-parent",
                    "value-sibling"
                ]
            }
        }"#,
        r#"
import { baz } from "baz";
import path from "path";
import { foo } from "../foo";

import { bar } from "./bar";
"#,
    );
}

#[test]
fn global_false_with_override_true_inserts_blank_line() {
    // Global newlinesBetween: false, but override inserts blank line between
    // value-builtin/value-external and value-parent
    assert_format(
        r#"
import path from "path";
import { foo } from "../foo";
import { bar } from "./bar";
import { baz } from "baz";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": false,
                "groups": [
                    ["value-builtin", "value-external"],
                    { "newlinesBetween": true },
                    "value-parent",
                    "value-sibling"
                ]
            }
        }"#,
        r#"
import { baz } from "baz";
import path from "path";

import { foo } from "../foo";
import { bar } from "./bar";
"#,
    );
}

#[test]
fn multiple_overrides() {
    // Global newlinesBetween: true with multiple overrides
    assert_format(
        r#"
import path from "path";
import type { T } from "t";
import { foo } from "../foo";
import { bar } from "./bar";
import { baz } from "baz";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": true,
                "groups": [
                    "type-import",
                    { "newlinesBetween": false },
                    ["value-builtin", "value-external"],
                    "value-parent",
                    { "newlinesBetween": false },
                    "value-sibling"
                ]
            }
        }"#,
        r#"
import type { T } from "t";
import { baz } from "baz";
import path from "path";

import { foo } from "../foo";
import { bar } from "./bar";
"#,
    );
}

#[test]
fn override_with_subgroups() {
    // Override between a subgroup and a single group
    assert_format(
        r#"
import path from "path";
import { foo } from "../foo";
import { bar } from "./bar";
import { baz } from "baz";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": false,
                "groups": [
                    ["value-builtin", "value-external"],
                    { "newlinesBetween": true },
                    ["value-parent", "value-sibling"]
                ]
            }
        }"#,
        r#"
import { baz } from "baz";
import path from "path";

import { foo } from "../foo";
import { bar } from "./bar";
"#,
    );
}

#[test]
fn all_overrides_false_is_same_as_global_false() {
    // All boundaries explicitly set to false
    assert_format(
        r#"
import path from "path";
import { foo } from "../foo";
import { bar } from "./bar";
import { baz } from "baz";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": true,
                "groups": [
                    ["value-builtin", "value-external"],
                    { "newlinesBetween": false },
                    "value-parent",
                    { "newlinesBetween": false },
                    "value-sibling"
                ]
            }
        }"#,
        r#"
import { baz } from "baz";
import path from "path";
import { foo } from "../foo";
import { bar } from "./bar";
"#,
    );
}

#[test]
fn override_does_not_affect_non_adjacent_groups() {
    // Override between groups 0 and 1, but groups 1 and 2 use global (true)
    assert_format(
        r#"
import path from "path";
import type { T } from "t";
import { foo } from "../foo";
import { baz } from "baz";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": true,
                "groups": [
                    "type-import",
                    { "newlinesBetween": false },
                    ["value-builtin", "value-external"],
                    "value-parent"
                ]
            }
        }"#,
        r#"
import type { T } from "t";
import { baz } from "baz";
import path from "path";

import { foo } from "../foo";
"#,
    );
}

#[test]
fn skipped_intermediate_groups_with_override_false() {
    // Groups: type-import, value-builtin, value-internal, value-parent
    // No imports match value-internal, so group index jumps from 1 to 3.
    // Both intermediate overrides are false, so no blank line.
    assert_format(
        r#"
import path from "path";
import { foo } from "../foo";
import type { T } from "t";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": true,
                "groups": [
                    "type-import",
                    { "newlinesBetween": false },
                    "value-builtin",
                    { "newlinesBetween": false },
                    "value-internal",
                    { "newlinesBetween": false },
                    "value-parent"
                ]
            }
        }"#,
        r#"
import type { T } from "t";
import path from "path";
import { foo } from "../foo";
"#,
    );
}

#[test]
fn skipped_intermediate_groups_with_mixed_overrides() {
    // Groups: type-import, value-builtin, value-internal, value-parent
    // No imports match value-internal, so group index jumps from 1 to 3.
    // Overrides: false between type-import/value-builtin, true between value-builtin/value-internal.
    // Since one boundary in the path is true, a blank line IS inserted.
    assert_format(
        r#"
import path from "path";
import { foo } from "../foo";
import type { T } from "t";
"#,
        r#"{
            "experimentalSortImports": {
                "newlinesBetween": false,
                "groups": [
                    "type-import",
                    "value-builtin",
                    { "newlinesBetween": true },
                    "value-internal",
                    "value-parent"
                ]
            }
        }"#,
        r#"
import type { T } from "t";
import path from "path";

import { foo } from "../foo";
"#,
    );
}
