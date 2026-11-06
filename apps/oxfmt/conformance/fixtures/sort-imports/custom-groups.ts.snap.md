# Input

`````ts
import { foo } from "./foo";
import { util } from "~/utils/util";
import { store } from "~/stores/store";

`````

# Output

## `{"sortImports":{"newlinesBetween":false,"customGroups":[{"elementNamePattern":["~/stores/**"],"groupName":"stores"},{"elementNamePattern":["~/utils/**"],"groupName":"utils"}],"groups":["stores","utils","sibling"]}}`

`````ts
import { store } from "~/stores/store";
import { util } from "~/utils/util";
import { foo } from "./foo";

`````
