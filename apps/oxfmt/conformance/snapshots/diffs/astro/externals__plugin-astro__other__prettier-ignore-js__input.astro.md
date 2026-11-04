# externals/plugin-astro/other/prettier-ignore-js/input.astro

## Option 1

`````json
{"printWidth":80,"astro":{}}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -7,6 +7,6 @@
 matrix(
   1, 0, 0,
   0, 1, 0,
   0, 0, 1
-)
+);
 ---

`````

### Actual (oxfmt)

`````astro
---
import { matrix } from "math";

matrix(1, 0, 0, 0, 1, 0, 0, 0, 1);

// prettier-ignore
matrix(
  1, 0, 0,
  0, 1, 0,
  0, 0, 1
);
---

`````

### Expected (prettier)

`````astro
---
import { matrix } from "math";

matrix(1, 0, 0, 0, 1, 0, 0, 0, 1);

// prettier-ignore
matrix(
  1, 0, 0,
  0, 1, 0,
  0, 0, 1
)
---

`````
