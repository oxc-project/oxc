use crate::{test, test_same};

#[test]
fn merge_imports() {
    test(
        "import { foo } from 'foo'; import { bar } from 'foo';",
        "import { foo, bar } from 'foo';",
    );
    test(
        "import { foo } from 'foo'; import { bar } from 'bar'; import { foo2 } from 'foo';",
        "import { foo, foo2 } from 'foo'; import { bar } from 'bar';",
    );
    test(
        "import { foo as foo1 } from 'foo'; import { bar as bar1 } from 'foo';",
        "import { foo as foo1, bar as bar1 } from 'foo';",
    );
    test(
        "import { foo } from 'foo'; import { bar } from 'bar'; import { foo2 } from 'foo'; import { bar2 } from 'bar';",
        "import { foo, foo2 } from 'foo'; import { bar, bar2 } from 'bar';",
    );
    test(
        "import { foo } from 'foo'; import { bar } from 'foo'; import { baz } from 'foo';",
        "import { foo, bar, baz } from 'foo';",
    );
    test("import foo from 'foo'; import { bar } from 'foo';", "import foo, { bar } from 'foo';");
    test("import { bar } from 'foo'; import foo from 'foo';", "import foo, { bar } from 'foo';");
    test("import foo from 'foo'; import * as ns from 'foo';", "import foo, * as ns from 'foo';");
    test("import * as ns from 'foo'; import foo from 'foo';", "import foo, * as ns from 'foo';");
    test(
        "import foo from 'foo'; import bar from 'foo';",
        "import foo, { default as bar } from 'foo';",
    );
    test(
        "import foo from 'foo'; import bar from 'foo'; import baz from 'foo';",
        "import foo, { default as bar, default as baz } from 'foo';",
    );

    test_same("import { foo } from 'foo'; import * as ns from 'foo';");
    test(
        "import foo from 'foo'; import bar from 'foo'; import * as ns from 'foo';",
        "import foo, { default as bar } from 'foo'; import * as ns from 'foo';",
    );
    test_same("import * as ns1 from 'foo'; import * as ns2 from 'foo';");
    test_same("import { foo } from 'foo' with { type: 'json' }; import { bar } from 'foo';");
    test_same(
        "import { foo } from 'foo' with { type: 'json' }; import { bar } from 'foo' with { type: 'css' };",
    );
}
