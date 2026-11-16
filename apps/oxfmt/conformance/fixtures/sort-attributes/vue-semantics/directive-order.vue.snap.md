# Input

`````vue
<template>
  <div v-z v-a />
  <div v-a v-z />
  <div v-z="next()" v-a="next()" />
  <div v-z:arg.modifier="value" v-a:[argument]="value" />
  <div v-z v-show="visible" v-a />
  <div z="before" a="before" v-z y="between" b="between" v-a x="after" c="after" />
</template>

`````

# Output

## `{"sortAttributes":true}`

`````vue
<template>
  <div v-z v-a />
  <div v-a v-z />
  <div v-z="next()" v-a="next()" />
  <div v-z:arg.modifier="value" v-a:[argument]="value" />
  <div v-z v-show="visible" v-a />
  <div a="before" z="before" v-z b="between" y="between" v-a c="after" x="after" />
</template>

`````

## `{"sortAttributes":{"order":"desc"}}`

`````vue
<template>
  <div v-z v-a />
  <div v-a v-z />
  <div v-z="next()" v-a="next()" />
  <div v-z:arg.modifier="value" v-a:[argument]="value" />
  <div v-z v-show="visible" v-a />
  <div z="before" a="before" v-z y="between" b="between" v-a x="after" c="after" />
</template>

`````

## `{"sortAttributes":{"groups":["^v-","^@","^:","$DEFAULT"]}}`

`````vue
<template>
  <div v-z v-a />
  <div v-a v-z />
  <div v-z="next()" v-a="next()" />
  <div v-z:arg.modifier="value" v-a:[argument]="value" />
  <div v-z v-show="visible" v-a />
  <div a="before" z="before" v-z b="between" y="between" v-a c="after" x="after" />
</template>

`````
