# Input

`````vue
<template>
  <a v-slot="  /* a */  " b="c" />
  <a #default="/* a */ /* b */" />
  <a v-for="(/* d */) in xs" />
  <a v-for="/* d */ of xs" />
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <a v-slot="  /* a */  " b="c" />
  <a #default="/* a */ /* b */" />
  <a v-for="(/* d */) in xs" />
  <a v-for="/* d */ of xs" />
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <a v-slot="  /* a */  " b="c" />
  <a #default="/* a */ /* b */" />
  <a v-for="(/* d */) in xs" />
  <a v-for="/* d */ of xs" />
</template>

`````
