# Input

`````vue
<template>
  <button
    @click="foo?.bar;"
    @input="(foo?.bar);"
    @change="foo()?.bar;"
    @blur="foo?.();"
    @focus=""
    @keyup="  "
    @keydown=" ; "
  ></button>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <button
    @click="foo?.bar;"
    @input="foo?.bar;"
    @change="foo()?.bar;"
    @blur="foo?.()"
    @focus=""
    @keyup=""
    @keydown=""
  ></button>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <button
    @click="foo?.bar;"
    @input="foo?.bar;"
    @change="foo()?.bar;"
    @blur="foo?.()"
    @focus=""
    @keyup=""
    @keydown=""
  ></button>
</template>

`````
