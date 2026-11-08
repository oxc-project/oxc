# Input

`````vue
<template>
  <Widget v-model="one" :modelValue="two" />
  <Widget :modelValue="two" v-model="one" />
  <Widget v-model:title="one" :title="two" />
  <Widget v-model:foo-bar="one" :fooBar="two" />
  <Widget v-model="one" @update:modelValue="two" />
  <Widget v-on:click="z" @click="a" />
  <Widget @click="z" v-on:click="a" />
  <Widget @click="z" :onClick="a" />
  <Widget :foo-bar="one" v-bind:fooBar.camel="two" />
  <Widget z="before" a="before" v-model="one" y="between" b="between" :modelValue="two" x="after" c="after" />
</template>

`````

# Output

## `{"sortAttributes":true}`

`````vue
<template>
  <Widget v-model="one" :modelValue="two" />
  <Widget :modelValue="two" v-model="one" />
  <Widget v-model:title="one" :title="two" />
  <Widget v-model:foo-bar="one" :fooBar="two" />
  <Widget v-model="one" @update:modelValue="two" />
  <Widget v-on:click="z" @click="a" />
  <Widget @click="z" v-on:click="a" />
  <Widget @click="z" :onClick="a" />
  <Widget :foo-bar="one" v-bind:fooBar.camel="two" />
  <Widget
    a="before"
    z="before"
    v-model="one"
    b="between"
    y="between"
    :modelValue="two"
    c="after"
    x="after"
  />
</template>

`````

## `{"sortAttributes":{"order":"desc"}}`

`````vue
<template>
  <Widget v-model="one" :modelValue="two" />
  <Widget :modelValue="two" v-model="one" />
  <Widget v-model:title="one" :title="two" />
  <Widget v-model:foo-bar="one" :fooBar="two" />
  <Widget v-model="one" @update:modelValue="two" />
  <Widget v-on:click="z" @click="a" />
  <Widget @click="z" v-on:click="a" />
  <Widget @click="z" :onClick="a" />
  <Widget :foo-bar="one" v-bind:fooBar.camel="two" />
  <Widget
    z="before"
    a="before"
    v-model="one"
    y="between"
    b="between"
    :modelValue="two"
    x="after"
    c="after"
  />
</template>

`````

## `{"sortAttributes":{"groups":["^v-","^@","^:","$DEFAULT"]}}`

`````vue
<template>
  <Widget v-model="one" :modelValue="two" />
  <Widget :modelValue="two" v-model="one" />
  <Widget v-model:title="one" :title="two" />
  <Widget v-model:foo-bar="one" :fooBar="two" />
  <Widget v-model="one" @update:modelValue="two" />
  <Widget v-on:click="z" @click="a" />
  <Widget @click="z" v-on:click="a" />
  <Widget @click="z" :onClick="a" />
  <Widget :foo-bar="one" v-bind:fooBar.camel="two" />
  <Widget
    a="before"
    z="before"
    v-model="one"
    b="between"
    y="between"
    :modelValue="two"
    c="after"
    x="after"
  />
</template>

`````
