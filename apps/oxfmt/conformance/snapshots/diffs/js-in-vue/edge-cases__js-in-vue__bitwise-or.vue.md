# edge-cases/js-in-vue/bitwise-or.vue

## Option 1

`````json
{"printWidth":80}
`````

### Diff

`````diff
===================================================================
--- prettier
+++ oxfmt
@@ -1,15 +1,15 @@
 <template>
   <p
     v-if="
       aaaaaaaaaaaaaaaaaaaa |
-        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb |
-        cccccccccccccccccccccccc
+      bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb |
+      cccccccccccccccccccccccc
     "
   >
     {{
-      value
-        | thisIsARealSuperLongFilterPipe("arg1", arg2)
-        | anotherPipeLongJustForFun
+      value |
+      thisIsARealSuperLongFilterPipe("arg1", arg2) |
+      anotherPipeLongJustForFun
     }}
   </p>
 </template>

`````

### Actual (oxfmt)

`````vue
<template>
  <p
    v-if="
      aaaaaaaaaaaaaaaaaaaa |
      bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb |
      cccccccccccccccccccccccc
    "
  >
    {{
      value |
      thisIsARealSuperLongFilterPipe("arg1", arg2) |
      anotherPipeLongJustForFun
    }}
  </p>
</template>

`````

### Expected (prettier)

`````vue
<template>
  <p
    v-if="
      aaaaaaaaaaaaaaaaaaaa |
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb |
        cccccccccccccccccccccccc
    "
  >
    {{
      value
        | thisIsARealSuperLongFilterPipe("arg1", arg2)
        | anotherPipeLongJustForFun
    }}
  </p>
</template>

`````
