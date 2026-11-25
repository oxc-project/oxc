# Input

`````vue
<template>
  <Comp
    #default="{ a = `line
${foo}` }"
  >{{ a }}</Comp>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <Comp
    #default="{
      a = `line
${foo}`,
    }"
    >{{ a }}</Comp
  >
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <Comp
    #default="{
      a = `line
${foo}`,
    }"
    >{{ a }}</Comp
  >
</template>

`````
