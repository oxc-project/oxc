# edge-cases/ember/suppressed-terminator.gjs

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,7 @@
 // The ignored `const` gains a terminator; the ignored bare tag does not.
 // See DIVERGENCES.md#suppressed-declaration-terminator
 // prettier-ignore
-const a = <template>  a  </template>
+const a = <template>  a  </template>;
 
 // prettier-ignore
 <template>  bare  </template>

`````

### Actual (oxfmt)

`````gjs
// The ignored `const` gains a terminator; the ignored bare tag does not.
// See DIVERGENCES.md#suppressed-declaration-terminator
// prettier-ignore
const a = <template>  a  </template>;

// prettier-ignore
<template>  bare  </template>

`````

### Expected (prettier)

`````gjs
// The ignored `const` gains a terminator; the ignored bare tag does not.
// See DIVERGENCES.md#suppressed-declaration-terminator
// prettier-ignore
const a = <template>  a  </template>

// prettier-ignore
<template>  bare  </template>

`````

## Option 2

`````json
{"printWidth":120,"singleQuote":true,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,7 +1,7 @@
 // The ignored `const` gains a terminator; the ignored bare tag does not.
 // See DIVERGENCES.md#suppressed-declaration-terminator
 // prettier-ignore
-const a = <template>  a  </template>
+const a = <template>  a  </template>;
 
 // prettier-ignore
 <template>  bare  </template>

`````

### Actual (oxfmt)

`````gjs
// The ignored `const` gains a terminator; the ignored bare tag does not.
// See DIVERGENCES.md#suppressed-declaration-terminator
// prettier-ignore
const a = <template>  a  </template>;

// prettier-ignore
<template>  bare  </template>

`````

### Expected (prettier)

`````gjs
// The ignored `const` gains a terminator; the ignored bare tag does not.
// See DIVERGENCES.md#suppressed-declaration-terminator
// prettier-ignore
const a = <template>  a  </template>

// prettier-ignore
<template>  bare  </template>

`````
