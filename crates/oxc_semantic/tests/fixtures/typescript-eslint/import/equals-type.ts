//// @sourceType = module

namespace Bar {}
namespace Qux {
    export namespace Baz {}
}

import type Foo = Bar;
import type Qualified = Qux.Baz;
