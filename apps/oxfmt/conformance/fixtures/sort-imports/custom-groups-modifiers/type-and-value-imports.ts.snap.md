# Input

`````ts
import { bar } from "@scope/bar";
import type { FooType } from "@scope/foo";
import { foo } from "@scope/foo";
import type { BarType } from "@scope/bar";

`````

# Output

## `{"sortImports":{"customGroups":[{"groupName":"scope-types","elementNamePattern":["@scope/**"],"modifiers":["type"]},{"groupName":"scope-values","elementNamePattern":["@scope/**"],"modifiers":["value"]}],"groups":["scope-types","scope-values","unknown"]}}`

`````ts
import type { BarType } from "@scope/bar";
import type { FooType } from "@scope/foo";

import { bar } from "@scope/bar";
import { foo } from "@scope/foo";

`````
