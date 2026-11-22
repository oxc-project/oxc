# Input

`````vue
<template>
  <li v-for="({ a = 'k'in o }) in items">x</li>
  <li v-for="({ a = ('k'in o) }) of items">x</li>
  <li v-for="({ a=of }) in items">x</li>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <li v-for="({ a = 'k'in o }) in items">x</li>
  <li v-for="({ a = ('k'in o) }) of items">x</li>
  <li v-for="({ a=of }) in items">x</li>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <li v-for="({ a = 'k'in o }) in items">x</li>
  <li v-for="({ a = ('k'in o) }) of items">x</li>
  <li v-for="({ a=of }) in items">x</li>
</template>

`````
