// Annex B.3.5 disallows a `var` initializer in a for-in head in strict mode.
"use strict";
for (var a = 1 in b);
