# Input

`````vue
<script setup="  {row}  "></script>
<style vars="  {color}  "></style>
<template>
  <C slot-scope="  {row}  ">x</C>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<script setup="  {row}  "></script>
<style vars="  {color}  "></style>
<template>
  <C slot-scope="  {row}  ">x</C>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<script setup="  {row}  "></script>
<style vars="  {color}  "></style>
<template>
  <C slot-scope="  {row}  ">x</C>
</template>

`````
