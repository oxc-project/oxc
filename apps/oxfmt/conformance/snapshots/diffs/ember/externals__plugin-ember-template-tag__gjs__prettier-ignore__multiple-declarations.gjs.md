# externals/plugin-ember-template-tag/gjs/prettier-ignore/multiple-declarations.gjs

## Option 1

`````json
{"printWidth":80,"ember":true}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -14,5 +14,5 @@
   <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
 </template>,
 ModVar4 = <template>
   Second module variable template.
-</template>
+</template>;

`````

### Actual (oxfmt)

`````gjs
// prettier-ignore
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 = <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool = false, ModVar3 =<template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4 = <template>
  Second module variable template.
</template>;

`````

### Expected (prettier)

`````gjs
// prettier-ignore
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 = <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool = false, ModVar3 =<template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4 = <template>
  Second module variable template.
</template>

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
@@ -14,5 +14,5 @@
   <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
 </template>,
 ModVar4 = <template>
   Second module variable template.
-</template>
+</template>;

`````

### Actual (oxfmt)

`````gjs
// prettier-ignore
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 = <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool = false, ModVar3 =<template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4 = <template>
  Second module variable template.
</template>;

`````

### Expected (prettier)

`````gjs
// prettier-ignore
const ModVar1 = <template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar2 = <template>
  Second module variable template.
</template>,
    num = 1;

// prettier-ignore
const bool = false, ModVar3 =<template>

  <h1>   Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template. Module variable template.   </h1>
</template>,
ModVar4 = <template>
  Second module variable template.
</template>

`````
