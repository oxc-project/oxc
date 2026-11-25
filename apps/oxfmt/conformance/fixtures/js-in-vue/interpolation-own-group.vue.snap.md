# Input

`````vue
<template>
  <div>
    <div>
      {{ aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa }} &times;
      {{ b }} =
    </div>
  </div>
</template>

`````

# Output

## `{"printWidth":80}`

`````vue
<template>
  <div>
    <div>
      {{ aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa }}
      &times; {{ b }} =
    </div>
  </div>
</template>

`````

## `{"printWidth":100,"vueIndentScriptAndStyle":true,"singleQuote":true,"semi":false}`

`````vue
<template>
  <div>
    <div>
      {{ aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa }} &times; {{ b }} =
    </div>
  </div>
</template>

`````
