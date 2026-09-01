use super::super::assert_format;

#[test]
fn supports_combination_of_predefined_and_custom_groups() {
    assert_format(
        r#"
import type { T } from "t";

// @ts-expect-error missing types
import { t } from "t";
"#,
        r#"
{
    "sortImports":  {
        "groups": [
            "side_effect_style",
            "type-external",
            "type-internal",
            "type-builtin",
            "type-sibling",
            "type-parent",
            "side_effect",
            "type-index",
            "internal",
            "external",
            "sibling",
            "unknown",
            "builtin",
            "parent",
            "index",
            "style",
            "type-import",
            "myCustomGroup1"
        ],
        "customGroups": [
            {
                "groupName": "myCustomGroup1",
                "elementNamePattern": ["x"],
                "modifiers": [
                    "type"
                ]
            }
        ]
    }
}
"#,
        r#"
import type { T } from "t";

// @ts-expect-error missing types
import { t } from "t";
"#,
    );
}

#[test]
fn handles_complex_projects_with_many_custom_groups() {
    assert_format(
        r#"
import { useCartStore } from "~/stores/cartStore.ts";
import { useUserStore } from "~/stores/userStore.ts";

import { getCart } from "~/services/cartService.ts";

import { connect } from "~/utils/ws.ts";
import { formattingDate } from "~/utils/dateTime.ts";

import { useFetch } from "~/composable/useFetch.ts";
import { useDebounce } from "~/composable/useDebounce.ts";
import { useMouseMove } from "~/composable/useMouseMove.ts";

import ComponentA from "~/components/ComponentA.vue";
import ComponentB from "~/components/ComponentB.vue";
import ComponentC from "~/components/ComponentC.vue";

import CartComponentA from "./cart/CartComponentA.vue";
import CartComponentB from "./cart/CartComponentB.vue";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "elementNamePattern": ["~/validators/**"],
                "groupName": "validators"
            },
            {
                "elementNamePattern": ["~/composable/**"],
                "groupName": "composable"
            },
            {
                "elementNamePattern": ["~/components/**"],
                "groupName": "components"
            },
            {
                "elementNamePattern": ["~/services/**"],
                "groupName": "services"
            },
            {
                "elementNamePattern": ["~/widgets/**"],
                "groupName": "widgets"
            },
            {
                "elementNamePattern": ["~/stores/**"],
                "groupName": "stores"
            },
            {
                "elementNamePattern": ["~/logics/**"],
                "groupName": "logics"
            },
            {
                "elementNamePattern": ["~/assets/**"],
                "groupName": "assets"
            },
            {
                "elementNamePattern": ["~/utils/**"],
                "groupName": "utils"
            },
            {
                "elementNamePattern": ["~/pages/**"],
                "groupName": "pages"
            },
            {
                "elementNamePattern": ["~/ui/**"],
                "groupName": "ui"
            }
        ],
        "groups": [
            ["builtin", "external"],
            "internal",
            "stores",
            "services",
            "validators",
            "utils",
            "logics",
            "composable",
            "ui",
            "components",
            "pages",
            "widgets",
            "assets",
            "parent",
            "sibling",
            "side_effect",
            "index",
            "style",
            "unknown"
        ]
    }
}
"#,
        r#"
import { useCartStore } from "~/stores/cartStore.ts";
import { useUserStore } from "~/stores/userStore.ts";

import { getCart } from "~/services/cartService.ts";

import { formattingDate } from "~/utils/dateTime.ts";
import { connect } from "~/utils/ws.ts";

import { useDebounce } from "~/composable/useDebounce.ts";
import { useFetch } from "~/composable/useFetch.ts";
import { useMouseMove } from "~/composable/useMouseMove.ts";

import ComponentA from "~/components/ComponentA.vue";
import ComponentB from "~/components/ComponentB.vue";
import ComponentC from "~/components/ComponentC.vue";

import CartComponentA from "./cart/CartComponentA.vue";
import CartComponentB from "./cart/CartComponentB.vue";
"#,
    );

    assert_format(
        r#"
import CartComponentA from "./cart/CartComponentA.vue";
import CartComponentB from "./cart/CartComponentB.vue";

import { connect } from "~/utils/ws.ts";
import { getCart } from "~/services/cartService.ts";

import { useUserStore } from "~/stores/userStore.ts";
import { formattingDate } from "~/utils/dateTime.ts";

import { useFetch } from "~/composable/useFetch.ts";
import { useCartStore } from "~/stores/cartStore.ts";
import { useDebounce } from "~/composable/useDebounce.ts";
import { useMouseMove } from "~/composable/useMouseMove.ts";

import ComponentA from "~/components/ComponentA.vue";
import ComponentB from "~/components/ComponentB.vue";
import ComponentC from "~/components/ComponentC.vue";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "elementNamePattern": ["~/validators/**"],
                "groupName": "validators"
            },
            {
                "elementNamePattern": ["~/composable/**"],
                "groupName": "composable"
            },
            {
                "elementNamePattern": ["~/components/**"],
                "groupName": "components"
            },
            {
                "elementNamePattern": ["~/services/**"],
                "groupName": "services"
            },
            {
                "elementNamePattern": ["~/widgets/**"],
                "groupName": "widgets"
            },
            {
                "elementNamePattern": ["~/stores/**"],
                "groupName": "stores"
            },
            {
                "elementNamePattern": ["~/logics/**"],
                "groupName": "logics"
            },
            {
                "elementNamePattern": ["~/assets/**"],
                "groupName": "assets"
            },
            {
                "elementNamePattern": ["~/utils/**"],
                "groupName": "utils"
            },
            {
                "elementNamePattern": ["~/pages/**"],
                "groupName": "pages"
            },
            {
                "elementNamePattern": ["~/ui/**"],
                "groupName": "ui"
            }
        ],
        "groups": [
            ["builtin", "external"],
            "internal",
            "stores",
            "services",
            "validators",
            "utils",
            "logics",
            "composable",
            "ui",
            "components",
            "pages",
            "widgets",
            "assets",
            "parent",
            "sibling",
            "side_effect",
            "index",
            "style",
            "unknown"
        ]
    }
}
"#,
        r#"
import { useCartStore } from "~/stores/cartStore.ts";
import { useUserStore } from "~/stores/userStore.ts";

import { getCart } from "~/services/cartService.ts";

import { formattingDate } from "~/utils/dateTime.ts";
import { connect } from "~/utils/ws.ts";

import { useDebounce } from "~/composable/useDebounce.ts";
import { useFetch } from "~/composable/useFetch.ts";
import { useMouseMove } from "~/composable/useMouseMove.ts";

import ComponentA from "~/components/ComponentA.vue";
import ComponentB from "~/components/ComponentB.vue";
import ComponentC from "~/components/ComponentC.vue";

import CartComponentA from "./cart/CartComponentA.vue";
import CartComponentB from "./cart/CartComponentB.vue";
"#,
    );
}

#[test]
fn glob_pattern_suffix_matching() {
    assert_format(
        r#"
import { setup } from "./setup.mock.ts";
import { a } from "./a.ts";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "mocks",
                "elementNamePattern": ["**/*.mock.ts"]
            }
        ],
        "groups": [
            "mocks",
            "unknown"
        ]
    }
}
"#,
        r#"
import { setup } from "./setup.mock.ts";

import { a } from "./a.ts";
"#,
    );
}

#[test]
fn glob_pattern_brace_expansion() {
    assert_format(
        r#"
import { createApp } from "vue";
import React from "react";
import Vuetify from "vuetify";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "frameworks",
                "elementNamePattern": ["{react,vue}"]
            }
        ],
        "groups": [
            "frameworks",
            "unknown"
        ]
    }
}
"#,
        r#"
import React from "react";
import { createApp } from "vue";

import Vuetify from "vuetify";
"#,
    );
}

#[test]
fn glob_pattern_exact_match() {
    assert_format(
        r#"
import { createApp } from "vue";
import Vuetify from "vuetify";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "vue-core",
                "elementNamePattern": ["vue"]
            }
        ],
        "groups": [
            "vue-core",
            "unknown"
        ]
    }
}
"#,
        r#"
import { createApp } from "vue";

import Vuetify from "vuetify";
"#,
    );
}

#[test]
fn custom_group_with_selector_only() {
    // Custom group matching by selector only (no elementNamePattern)
    assert_format(
        r#"
import { foo } from "foo";
import type { Bar } from "bar";
import { baz } from "baz";
import type { Qux } from "qux";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "types",
                "selector": "type"
            }
        ],
        "groups": [
            "types",
            "unknown"
        ]
    }
}
"#,
        r#"
import type { Bar } from "bar";
import type { Qux } from "qux";

import { baz } from "baz";
import { foo } from "foo";
"#,
    );
}

#[test]
fn custom_group_with_modifiers_only() {
    // Custom group matching by modifiers only (no elementNamePattern)
    assert_format(
        r#"
import { foo } from "foo";
import type { Bar } from "bar";
import { baz } from "baz";
import type { Qux } from "qux";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "type-imports",
                "modifiers": ["type"]
            }
        ],
        "groups": [
            "type-imports",
            "unknown"
        ]
    }
}
"#,
        r#"
import type { Bar } from "bar";
import type { Qux } from "qux";

import { baz } from "baz";
import { foo } from "foo";
"#,
    );
}

#[test]
fn custom_group_with_selector_and_pattern() {
    // Custom group matching by selector + elementNamePattern
    assert_format(
        r#"
import type { InternalType } from "~/types";
import type { ExternalType } from "ext-lib";
import { internalUtil } from "~/utils";
import { externalUtil } from "ext-lib";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "internal-types",
                "selector": "internal",
                "elementNamePattern": ["~/**"]
            }
        ],
        "groups": [
            "internal-types",
            "unknown"
        ],
        "internalPattern": ["~/"]
    }
}
"#,
        r#"
import type { InternalType } from "~/types";
import { internalUtil } from "~/utils";

import type { ExternalType } from "ext-lib";
import { externalUtil } from "ext-lib";
"#,
    );
}

#[test]
fn custom_group_with_selector_modifiers_and_pattern() {
    // Custom group matching by selector + modifiers + elementNamePattern (all AND)
    assert_format(
        r#"
import type { InternalType } from "~/types";
import type { ExternalType } from "ext-lib";
import { internalUtil } from "~/utils";
import { externalUtil } from "ext-lib";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "internal-type-imports",
                "selector": "internal",
                "modifiers": ["type"],
                "elementNamePattern": ["~/**"]
            }
        ],
        "groups": [
            "internal-type-imports",
            "unknown"
        ],
        "internalPattern": ["~/"]
    }
}
"#,
        r#"
import type { InternalType } from "~/types";

import type { ExternalType } from "ext-lib";
import { externalUtil } from "ext-lib";
import { internalUtil } from "~/utils";
"#,
    );
}

#[test]
fn custom_group_no_match_falls_to_unknown() {
    // When selector doesn't match, import falls to unknown
    assert_format(
        r#"
import { foo } from "foo";
import { bar } from "bar";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "types-only",
                "selector": "type"
            }
        ],
        "groups": [
            "types-only",
            "unknown"
        ]
    }
}
"#,
        r#"
import { bar } from "bar";
import { foo } from "foo";
"#,
    );
}

#[test]
fn custom_group_multiple_modifiers_and_logic() {
    // All specified modifiers must match (AND logic)
    // "type" + "named" matches only `import type { ... }`, not `import type X` (default only)
    assert_format(
        r#"
import type Bar from "bar";
import type { Foo } from "foo";
import { regular } from "regular";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "type-named",
                "modifiers": ["type", "named"]
            }
        ],
        "groups": [
            "type-named",
            "unknown"
        ]
    }
}
"#,
        r#"
import type { Foo } from "foo";

import type Bar from "bar";
import { regular } from "regular";
"#,
    );
}

#[test]
fn selector_external_groups_external_imports() {
    // selector: "external" separates external from sibling imports
    assert_format(
        r#"
import a from "a";
import b from "./b";
import c from "c";
import d from "./d";
import e from "e";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "externalImports",
                "selector": "external"
            }
        ],
        "groups": [
            "externalImports",
            "unknown"
        ]
    }
}
"#,
        r#"
import a from "a";
import c from "c";
import e from "e";

import b from "./b";
import d from "./d";
"#,
    );
}

#[test]
fn custom_groups_with_predefined_type_group() {
    // Custom groups take priority over predefined groups.
    // `import type { T } from "t"` matches custom "primary" (pattern "t") before predefined "type".
    assert_format(
        r#"
import type { T } from "t";
import a1 from "@a/a1";
import a2 from "@a/a2";
import b1 from "@b/b1";
import b2 from "@b/b2";
import b3 from "@b/b3";
import { c } from "c";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "primary",
                "elementNamePattern": ["t", "@a/**"]
            },
            {
                "groupName": "secondary",
                "elementNamePattern": ["@b/**"],
                "modifiers": ["value"]
            }
        ],
        "groups": [
            "type",
            "primary",
            "secondary",
            "unknown"
        ]
    }
}
"#,
        r#"
import a1 from "@a/a1";
import a2 from "@a/a2";
import type { T } from "t";

import b1 from "@b/b1";
import b2 from "@b/b2";
import b3 from "@b/b3";

import { c } from "c";
"#,
    );
}

#[test]
fn multiple_custom_groups_with_different_selectors() {
    // Each custom group uses a different selector to categorize imports
    assert_format(
        r#"
import fs from "node:fs";
import { foo } from "external-lib";
import { bar } from "~/internal";
import { baz } from "./sibling";
"#,
        r#"
{
    "sortImports": {
        "internalPattern": ["~/"],
        "customGroups": [
            {
                "groupName": "builtins",
                "selector": "builtin"
            },
            {
                "groupName": "externals",
                "selector": "external"
            },
            {
                "groupName": "internals",
                "selector": "internal"
            }
        ],
        "groups": [
            "builtins",
            "externals",
            "internals",
            "unknown"
        ]
    }
}
"#,
        r#"
import fs from "node:fs";

import { foo } from "external-lib";

import { bar } from "~/internal";

import { baz } from "./sibling";
"#,
    );
}

#[test]
fn same_pattern_differentiated_by_modifiers() {
    // Same elementNamePattern used in two custom groups, differentiated by type vs value modifiers
    assert_format(
        r#"
import type { FooType } from "@scope/foo";
import { foo } from "@scope/foo";
import type { BarType } from "@scope/bar";
import { bar } from "@scope/bar";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "scope-types",
                "elementNamePattern": ["@scope/**"],
                "modifiers": ["type"]
            },
            {
                "groupName": "scope-values",
                "elementNamePattern": ["@scope/**"],
                "modifiers": ["value"]
            }
        ],
        "groups": [
            "scope-types",
            "scope-values",
            "unknown"
        ]
    }
}
"#,
        r#"
import type { BarType } from "@scope/bar";
import type { FooType } from "@scope/foo";

import { bar } from "@scope/bar";
import { foo } from "@scope/foo";
"#,
    );
}

#[test]
fn custom_side_effect_group_regroups_side_effect_imports() {
    // A catch-all `selector: "side_effect"` custom group is an explicit opt-in:
    // side-effect imports move between groups even with the default `sortSideEffects: false`.
    // The side-effect custom group is defined first,
    // so it wins over the path-based `warp-drive` group that would also match the side-effect import.
    let config = r#"
{
    "sortImports": {
        "groups": [
            "side-effect",
            "ember-glimmer",
            ["builtin", "external"],
            "warp-drive",
            ["parent", "sibling", "index"]
        ],
        "customGroups": [
            { "groupName": "side-effect", "selector": "side_effect" },
            { "groupName": "ember-glimmer", "elementNamePattern": ["@ember/**"] },
            { "groupName": "warp-drive", "elementNamePattern": ["@warp-drive/**"] }
        ]
    }
}
"#;

    // Side-effect import at the head of the input
    assert_format(
        r#"
import "@warp-drive/ember/install";
import { bar } from "@warp-drive/core";
"#,
        config,
        r#"
import "@warp-drive/ember/install";

import { bar } from "@warp-drive/core";
"#,
    );

    // Side-effect import in the middle of the input
    assert_format(
        r#"
import { css } from "@ember/css";
import "@warp-drive/ember/install";
import { bar } from "@warp-drive/core";
"#,
        config,
        r#"
import "@warp-drive/ember/install";

import { css } from "@ember/css";

import { bar } from "@warp-drive/core";
"#,
    );
}

#[test]
fn custom_side_effect_style_group_regroups_style_side_effects() {
    // Symmetric opt-in for `selector: "side_effect_style"`
    assert_format(
        r#"
import { a } from "a";
import "./styles.css";
"#,
        r#"
{
    "sortImports": {
        "groups": ["side-effect-style", "external", "unknown"],
        "customGroups": [
            { "groupName": "side-effect-style", "selector": "side_effect_style" }
        ]
    }
}
"#,
        r#"
import "./styles.css";

import { a } from "a";
"#,
    );
}

#[test]
fn non_catch_all_custom_group_does_not_regroup_side_effects() {
    // A pattern-based custom group that happens to match a side-effect import is NOT an opt-in:
    // with `sortSideEffects: false` the import stays in place.
    assert_format(
        r#"
import { a } from "a";
import "pkg/setup";
"#,
        r#"
{
    "sortImports": {
        "groups": ["pkg", "unknown"],
        "customGroups": [
            { "groupName": "pkg", "elementNamePattern": ["pkg/**"] }
        ]
    }
}
"#,
        r#"
import { a } from "a";
import "pkg/setup";
"#,
    );

    // Even with `selector: "side_effect"`, a group narrowed by a pattern is not catch-all
    assert_format(
        r#"
import { a } from "a";
import "pkg/setup";
"#,
        r#"
{
    "sortImports": {
        "groups": ["pkg", "unknown"],
        "customGroups": [
            { "groupName": "pkg", "selector": "side_effect", "elementNamePattern": ["pkg/**"] }
        ]
    }
}
"#,
        r#"
import { a } from "a";
import "pkg/setup";
"#,
    );
}

#[test]
fn side_effect_import_does_not_match_specifier_modifiers() {
    // A genuine side-effect import has no specifiers at all:
    // it must not inherit `named`/`default`/`wildcard` from the following import,
    // nor mistake its own module source for a default binding.
    // `sortSideEffects: true` so the (miss)computed group placement becomes observable.
    let named_only_config = r#"
{
    "sortImports": {
        "sortSideEffects": true,
        "groups": ["named-only", "unknown"],
        "customGroups": [
            { "groupName": "named-only", "modifiers": ["named"] }
        ]
    }
}
"#;
    let defaults_config = r#"
{
    "sortImports": {
        "sortSideEffects": true,
        "groups": ["defaults", "unknown"],
        "customGroups": [
            { "groupName": "defaults", "modifiers": ["default"] }
        ]
    }
}
"#;

    // Followed by a named import
    assert_format(
        r#"
import "@warp-drive/ember/install";
import { bar } from "@warp-drive/core";
"#,
        named_only_config,
        r#"
import { bar } from "@warp-drive/core";

import "@warp-drive/ember/install";
"#,
    );

    // Followed by a default import; also covers the own-source-as-default artifact
    assert_format(
        r#"
import "./setup";
import a from "a";
"#,
        defaults_config,
        r#"
import a from "a";

import "./setup";
"#,
    );

    // At the end of the import chunk (no following import to leak from)
    assert_format(
        r#"
import a from "a";
import "./setup";
"#,
        defaults_config,
        r#"
import a from "a";

import "./setup";
"#,
    );

    // Followed by a wildcard import
    assert_format(
        r#"
import "./setup";
import * as ns from "n";
"#,
        r#"
{
    "sortImports": {
        "sortSideEffects": true,
        "groups": ["wildcards", "unknown"],
        "customGroups": [
            { "groupName": "wildcards", "modifiers": ["wildcard"] }
        ]
    }
}
"#,
        r#"
import * as ns from "n";

import "./setup";
"#,
    );

    // A blank line and a comment between imports do not stop the leak either
    assert_format(
        r#"
import "./setup";

// comment
import { bar } from "b";
"#,
        named_only_config,
        r#"
// comment
import { bar } from "b";

import "./setup";
"#,
    );
}

#[test]
fn default_modifier_requires_actual_default_binding() {
    // Named specifier names and a namespace alias are `Text` elements in the head,
    // but they are not default bindings and must not produce the `default` modifier.
    assert_format(
        r#"
import { b } from "b";
import * as ns from "n";
import d from "d";
"#,
        r#"
{
    "sortImports": {
        "groups": ["defaults", "unknown"],
        "customGroups": [
            { "groupName": "defaults", "modifiers": ["default"] }
        ]
    }
}
"#,
        r#"
import d from "d";

import { b } from "b";
import * as ns from "n";
"#,
    );
}

#[test]
fn side_effect_import_keeps_value_and_side_effect_modifiers() {
    // `value` and `side_effect` modifiers keep their meaning for side-effect imports,
    // so `modifiers: ["side_effect"]` / `["value"]` custom groups still match them.
    assert_format(
        r#"
import a from "a";
import "./setup";
"#,
        r#"
{
    "sortImports": {
        "sortSideEffects": true,
        "groups": ["se", "unknown"],
        "customGroups": [
            { "groupName": "se", "modifiers": ["side_effect"] }
        ]
    }
}
"#,
        r#"
import "./setup";

import a from "a";
"#,
    );

    assert_format(
        r#"
import a from "a";
import "./setup";
"#,
        r#"
{
    "sortImports": {
        "sortSideEffects": true,
        "groups": ["values", "unknown"],
        "customGroups": [
            { "groupName": "values", "modifiers": ["value"] }
        ]
    }
}
"#,
        r#"
import "./setup";
import a from "a";
"#,
    );
}

#[test]
fn selector_sibling_with_type_modifier() {
    // selector "sibling" + modifiers ["type"] matches only type sibling imports
    assert_format(
        r#"
import a from "a";
import b from "./b";
import type c from "./c";
import type d from "./d";
import e from "e";
"#,
        r#"
{
    "sortImports": {
        "customGroups": [
            {
                "groupName": "typeSiblings",
                "selector": "sibling",
                "modifiers": ["type"]
            }
        ],
        "groups": [
            "typeSiblings",
            "unknown"
        ]
    }
}
"#,
        r#"
import type c from "./c";
import type d from "./d";

import b from "./b";
import a from "a";
import e from "e";
"#,
    );
}
