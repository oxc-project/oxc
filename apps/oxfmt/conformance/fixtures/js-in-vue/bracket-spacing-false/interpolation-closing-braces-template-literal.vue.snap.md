# Input

`````vue
<template>
  <p>{{ `${ {a: 1} }` }}</p>
  <p>{{ `${ a || {b: 1} }` }}</p>
  <p :title="`${ {a: 1} }`">x</p>
</template>

`````

# Output

## `{}`

`````vue
<template>
  <p>{{ `${({ a: 1 })}` }}</p>
  <p>{{ `${a || ({ b: 1 })}` }}</p>
  <p :title="`${{ a: 1 }}`">x</p>
</template>

`````

## `{"bracketSpacing":false}`

`````vue
<template>
  <p>{{ `${({a: 1})}` }}</p>
  <p>{{ `${a || ({b: 1})}` }}</p>
  <p :title="`${{a: 1}}`">x</p>
</template>

`````
