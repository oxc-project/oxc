use crate::{test, test_same, test_target};

#[test]
fn merge_import_and_export() {
    test("import { foo } from 'somewhere'; export { foo };", "export { foo } from 'somewhere';");
    test(
        "import { foo as bar } from 'somewhere'; export { bar as baz };",
        "export { foo as baz } from 'somewhere';",
    );
    test(
        "import foo from 'somewhere'; export { foo };",
        "export { default as foo } from 'somewhere';",
    );
    test(
        "import foo from 'somewhere'; export { foo as bar };",
        "export { default as bar } from 'somewhere';",
    );
    test(
        "import { default as foo } from 'somewhere'; export { foo };",
        "export { default as foo } from 'somewhere';",
    );
    test(
        "import foo, { bar } from 'somewhere'; export { foo, bar };",
        "export { default as foo, bar } from 'somewhere';",
    );
    test(
        "import foo, { bar } from 'somewhere'; export { bar, foo };",
        "export { bar, default as foo } from 'somewhere';",
    );
    test(
        "import { foo, bar as baz } from 'somewhere'; export { baz, foo as qux };",
        "export { bar as baz, foo as qux } from 'somewhere';",
    );
    test(
        "import { foo } from 'somewhere'; export { foo as bar, foo as baz };",
        "export { foo as bar, foo as baz } from 'somewhere';",
    );
    test("import {} from 'somewhere';", "import 'somewhere';");
    test("import {} from 'somewhere'; export {};", "import 'somewhere'; export {};");
    test(
        "import { foo } from 'somewhere' with { type: 'json' }; export { foo };",
        "export { foo } from 'somewhere' with { type: 'json' };",
    );
    test("import * as foo from 'somewhere'; export { foo };", "export * as foo from 'somewhere';");
    test(
        "import * as foo from 'somewhere'; export { foo as bar };",
        "export * as bar from 'somewhere';",
    );
    test_same("import * as foo from 'somewhere'; export { foo, foo as bar };");
    test(
        "import * as foo from 'somewhere'; export { foo as \"bar\" };",
        "export * as \"bar\" from 'somewhere';",
    );
    test_same("import foo, * as bar from 'foo'; export { foo, bar };");
    test_target(
        "import * as foo from 'somewhere'; export { foo };",
        "import * as foo from 'somewhere'; export { foo };",
        "es2019",
    );
    test_target(
        "import * as foo from 'somewhere'; export { foo };",
        "export * as foo from 'somewhere';",
        "es2020",
    );
    // preserve import order
    test(
        "import { foo } from 'foo'; import { bar } from 'bar'; export { bar }; export { foo };",
        "export { foo } from 'foo'; export { bar } from 'bar';",
    );
    test(
        "import { foo } from 'foo'; import 'side-effect'; import { bar } from 'bar'; export { bar }; export { foo };",
        "export { foo } from 'foo'; import 'side-effect'; export { bar } from 'bar';",
    );

    // do not merge if it's used in the module
    test_same("import { foo } from 'somewhere'; export { foo }; console.log(foo);");
    test(
        "import { foo } from 'somewhere'; console.log(0); export { foo };",
        "export { foo } from 'somewhere'; console.log(0);",
    );
    test_same("import foo, { bar } from 'somewhere'; export { bar };");
    test_same("import * as foo from 'somewhere'; console.log(foo); export { foo };");
    test_same("import { foo } from 'somewhere'; export { foo }; eval('foo');");
}
