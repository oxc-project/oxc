# Input

`````vue
<template>
  <li v-for="item   in items // c" :key="item">x</li>
  <a :a="{ b } // c" :b="x  +  y // c" @click="go( ) // c" :d="{
    a: 1, // c
  }" />
  <C #default="{ item } // c" />
  <p>{{ a  +  b // c }}</p>
  <a :a="{ b } // c
  " :b="x  +  y // c
  " />
  <C #default="{ item } // c
  " />
  <a :a="{ b } // c
  /* d */" />
  <C #default="{ item } // c
  /* d */" />
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <li v-for="item   in items // c" :key="item">x</li>
  <a
    :a="{ b } // c"
    :b="x  +  y // c"
    @click="go( ) // c"
    :d="{
      a: 1, // c
    }"
  />
  <C #default="{ item } // c" />
  <p>{{ a  +  b // c }}</p>
  <a
    :a="
      { b } // c
    "
    :b="
      x + y // c
    "
  />
  <C
    #default="
      { item } // c
    "
  />
  <a
    :a="
      { b } // c
      /* d */
    "
  />
  <C
    #default="
      { item } // c
      /* d */
    "
  />
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <li v-for="item   in items // c" :key="item">x</li>
  <a
    :a="{ b } // c"
    :b="x  +  y // c"
    @click="go( ) // c"
    :d="{
      a: 1, // c
    }"
  />
  <C #default="{ item } // c" />
  <p>{{ a  +  b // c }}</p>
  <a
    :a="
      { b } // c
    "
    :b="
      x + y // c
    "
  />
  <C
    #default="
      { item } // c
    "
  />
  <a
    :a="
      { b } // c
      /* d */
    "
  />
  <C
    #default="
      { item } // c
      /* d */
    "
  />
</template>

`````
