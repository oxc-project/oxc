# externals/plugin-ember-template-tag/gjs/prettier-ignore/exported-mod-var.gjs

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
-export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>
+export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Actual (oxfmt)

`````gjs
// prettier-ignore
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Expected (prettier)

`````gjs
// prettier-ignore
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>

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
-export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>
+export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Actual (oxfmt)

`````gjs
// prettier-ignore
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>;

`````

### Expected (prettier)

`````gjs
// prettier-ignore
export const Exported = <template>       Exported variable template. Exported variable template.  Exported variable template.  Exported variable template.  Exported variable template. Exported variable template. Exported variable template. </template>

`````
