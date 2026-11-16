# Input

`````vue
<template>
  <Widget :z="next()" :a="next()" />
  <Widget :a="next()" :z="next()" />
  <Widget :z="state.z" :a="state.a" />
  <Widget :z="z" :a="a" />
  <Widget :z="next()" :a />
  <Widget v-bind:z="next()" v-bind:a="next()" />
  <Widget z="before" a="before" :effect="next()" y="after" b="after" />
  <Widget z="before" a="before" v-bind="next()" y="after" b="after" />
  <Widget z="before" a="before" :[next()]="next()" y="after" b="after" />
</template>

`````

# Output

## `{"sortAttributes":true}`

`````vue
<template>
  <Widget :z="next()" :a="next()" />
  <Widget :a="next()" :z="next()" />
  <Widget :z="state.z" :a="state.a" />
  <Widget :z="z" :a="a" />
  <Widget :z="next()" :a />
  <Widget v-bind:z="next()" v-bind:a="next()" />
  <Widget a="before" z="before" :effect="next()" b="after" y="after" />
  <Widget a="before" z="before" v-bind="next()" b="after" y="after" />
  <Widget a="before" z="before" :[next()]="next()" b="after" y="after" />
</template>

`````

## `{"sortAttributes":{"order":"desc"}}`

`````vue
<template>
  <Widget :z="next()" :a="next()" />
  <Widget :a="next()" :z="next()" />
  <Widget :z="state.z" :a="state.a" />
  <Widget :z="z" :a="a" />
  <Widget :z="next()" :a />
  <Widget v-bind:z="next()" v-bind:a="next()" />
  <Widget z="before" a="before" :effect="next()" y="after" b="after" />
  <Widget z="before" a="before" v-bind="next()" y="after" b="after" />
  <Widget z="before" a="before" :[next()]="next()" y="after" b="after" />
</template>

`````

## `{"sortAttributes":{"groups":["^v-","^@","^:","$DEFAULT"]}}`

`````vue
<template>
  <Widget :z="next()" :a="next()" />
  <Widget :a="next()" :z="next()" />
  <Widget :z="state.z" :a="state.a" />
  <Widget :z="z" :a="a" />
  <Widget :z="next()" :a />
  <Widget v-bind:z="next()" v-bind:a="next()" />
  <Widget a="before" z="before" :effect="next()" b="after" y="after" />
  <Widget a="before" z="before" v-bind="next()" b="after" y="after" />
  <Widget a="before" z="before" :[next()]="next()" b="after" y="after" />
</template>

`````
