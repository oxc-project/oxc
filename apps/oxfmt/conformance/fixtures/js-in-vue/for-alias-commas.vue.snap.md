# Input

`````vue
<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>

`````
