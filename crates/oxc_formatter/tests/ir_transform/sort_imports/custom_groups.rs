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
    "experimentalSortImports":  {
        "groups": [
            "side-effect-style",
            "type-external",
            "type-internal",
            "type-builtin",
            "type-sibling",
            "type-parent",
            "side-effect",
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
    "experimentalSortImports": {
        "customGroups": [
            {
                "elementNamePattern": ["~/validators/"],
                "groupName": "validators"
            },
            {
                "elementNamePattern": ["~/composable/"],
                "groupName": "composable"
            },
            {
                "elementNamePattern": ["~/components/"],
                "groupName": "components"
            },
            {
                "elementNamePattern": ["~/services/"],
                "groupName": "services"
            },
            {
                "elementNamePattern": ["~/widgets/"],
                "groupName": "widgets"
            },
            {
                "elementNamePattern": ["~/stores/"],
                "groupName": "stores"
            },
            {
                "elementNamePattern": ["~/logics/"],
                "groupName": "logics"
            },
            {
                "elementNamePattern": ["~/assets/"],
                "groupName": "assets"
            },
            {
                "elementNamePattern": ["~/utils/"],
                "groupName": "utils"
            },
            {
                "elementNamePattern": ["~/pages/"],
                "groupName": "pages"
            },
            {
                "elementNamePattern": ["~/ui/"],
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
            "side-effect",
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
    "experimentalSortImports": {
        "customGroups": [
            {
                "elementNamePattern": ["~/validators/"],
                "groupName": "validators"
            },
            {
                "elementNamePattern": ["~/composable/"],
                "groupName": "composable"
            },
            {
                "elementNamePattern": ["~/components/"],
                "groupName": "components"
            },
            {
                "elementNamePattern": ["~/services/"],
                "groupName": "services"
            },
            {
                "elementNamePattern": ["~/widgets/"],
                "groupName": "widgets"
            },
            {
                "elementNamePattern": ["~/stores/"],
                "groupName": "stores"
            },
            {
                "elementNamePattern": ["~/logics/"],
                "groupName": "logics"
            },
            {
                "elementNamePattern": ["~/assets/"],
                "groupName": "assets"
            },
            {
                "elementNamePattern": ["~/utils/"],
                "groupName": "utils"
            },
            {
                "elementNamePattern": ["~/pages/"],
                "groupName": "pages"
            },
            {
                "elementNamePattern": ["~/ui/"],
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
            "side-effect",
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
