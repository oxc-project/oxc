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
