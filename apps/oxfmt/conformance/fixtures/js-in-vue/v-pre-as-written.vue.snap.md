# Input

`````vue
<template>
  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <div v-pre :a="b+c">{{a+b}} <span @click="x+y">{{c}}</span></div>
</template>

`````
