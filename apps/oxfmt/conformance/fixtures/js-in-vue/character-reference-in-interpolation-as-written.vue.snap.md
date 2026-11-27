# Input

`````vue
<template>
  <p>{{ '&quot;' }} {{ '&quot' }} {{ '&times;' }} {{ a  +  b }}</p>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <p>{{ '&quot;' }} {{ '&quot' }} {{ "&times;" }} {{ a + b }}</p>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <p>{{ '&quot;' }} {{ '&quot' }} {{ '&times;' }} {{ a + b }}</p>
</template>

`````
