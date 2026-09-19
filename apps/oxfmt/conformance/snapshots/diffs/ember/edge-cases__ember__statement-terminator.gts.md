# edge-cases/ember/statement-terminator.gts

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,11 +1,1 @@
-// A template tag in statement position is a declaration: no terminator, and no `export
-// default`, which is a second spelling of the same thing. We print the `as` and `satisfies`
-// forms alike; the plugin omits the terminator for one and emits it for the other.
-// See DIVERGENCES.md#template-tag-statement-terminator
-<template>explicit</template>
-
-<template>bare</template>
-
-<template>as</template> as Foo;
-
-<template>satisfies</template> satisfies Foo;
+ERROR
\ No newline at end of file

`````

### Actual (oxfmt)

`````gts
ERROR
`````

### Expected (prettier)

`````gts
// A template tag in statement position is a declaration: no terminator, and no `export
// default`, which is a second spelling of the same thing. We print the `as` and `satisfies`
// forms alike; the plugin omits the terminator for one and emits it for the other.
// See DIVERGENCES.md#template-tag-statement-terminator
<template>explicit</template>

<template>bare</template>

<template>as</template> as Foo;

<template>satisfies</template> satisfies Foo;

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
@@ -1,11 +1,1 @@
-// A template tag in statement position is a declaration: no terminator, and no `export
-// default`, which is a second spelling of the same thing. We print the `as` and `satisfies`
-// forms alike; the plugin omits the terminator for one and emits it for the other.
-// See DIVERGENCES.md#template-tag-statement-terminator
-<template>explicit</template>
-
-<template>bare</template>
-
-<template>as</template> as Foo;
-
-<template>satisfies</template> satisfies Foo;
+ERROR
\ No newline at end of file

`````

### Actual (oxfmt)

`````gts
ERROR
`````

### Expected (prettier)

`````gts
// A template tag in statement position is a declaration: no terminator, and no `export
// default`, which is a second spelling of the same thing. We print the `as` and `satisfies`
// forms alike; the plugin omits the terminator for one and emits it for the other.
// See DIVERGENCES.md#template-tag-statement-terminator
<template>explicit</template>

<template>bare</template>

<template>as</template> as Foo;

<template>satisfies</template> satisfies Foo;

`````
