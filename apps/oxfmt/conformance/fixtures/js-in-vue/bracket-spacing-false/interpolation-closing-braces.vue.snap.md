# Input

`````vue
<template>
  <p>{{ { a: { b: 1 } } }}</p>
  <p>{{ { ...{ a: 1 } } }}</p>
  <p>{{ { ...!{ a: 1 } } }}</p>
  <p>{{ { ...(a ? b : { c }) } }}</p>
</template>

`````

# Output

## `{}`

`````vue
<template>
  <p>{{ { a: { b: 1 } } }}</p>
  <p>{{ { ...{ a: 1 } } }}</p>
  <p>{{ { ...!{ a: 1 } } }}</p>
  <p>{{ { ...(a ? b : { c }) } }}</p>
</template>

`````

## `{"bracketSpacing":false}`

`````vue
<template>
  <p>{{ {a: ({b: 1})} }}</p>
  <p>{{ {...({a: 1})} }}</p>
  <p>{{ {...!({a: 1})} }}</p>
  <p>{{ {...(a ? b : {c})} }}</p>
</template>

`````
