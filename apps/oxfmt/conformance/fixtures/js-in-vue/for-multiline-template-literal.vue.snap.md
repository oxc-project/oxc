# Input

`````vue
<template>
  <div v-for="(item = `line
${foo}`, index) in items">{{ item }} - {{ index }}</div>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <div
    v-for="(
      item = `line
${foo}`,
      index
    ) in items"
  >
    {{ item }} - {{ index }}
  </div>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <div
    v-for="(
      item = `line
${foo}`,
      index
    ) in items"
  >
    {{ item }} - {{ index }}
  </div>
</template>

`````
