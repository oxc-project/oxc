# externals/plugin-ember-template-tag/gjs/prettier-ignore/one-line.gjs

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,2 +1,2 @@
 // prettier-ignore
-const Oneline = <template>      Module variable template (one line). </template>
+const Oneline = <template>      Module variable template (one line). </template>;

`````

### Actual (oxfmt)

`````gjs
// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template>;

`````

### Expected (prettier)

`````gjs
// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template>

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
@@ -1,2 +1,2 @@
 // prettier-ignore
-const Oneline = <template>      Module variable template (one line). </template>
+const Oneline = <template>      Module variable template (one line). </template>;

`````

### Actual (oxfmt)

`````gjs
// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template>;

`````

### Expected (prettier)

`````gjs
// prettier-ignore
const Oneline = <template>      Module variable template (one line). </template>

`````
