# Input

`````vue
<template>
  <p v-if="aaaaaaaaaaaaaaaaaaaa | bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | cccccccccccccccccccccccc">{{ value | thisIsARealSuperLongFilterPipe("arg1", arg2) | anotherPipeLongJustForFun }}</p>
</template>

`````

# Output

## `{"printWidth":80}`

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

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <p v-if="aaaaaaaaaaaaaaaaaaaa | bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb | cccccccccccccccccccccccc">
    {{ value | thisIsARealSuperLongFilterPipe('arg1', arg2) | anotherPipeLongJustForFun }}
  </p>
</template>

`````
