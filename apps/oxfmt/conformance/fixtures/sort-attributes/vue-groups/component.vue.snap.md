# Input

`````vue
<template>
  <Widget @input="input" z="last" :bound-z="z" title="Title" id="id" v-if="show" v-for="item in items" v-model="value" class="classes" data-z="z" data-a="a" :bound-a="a" a="first" tabindex="0" @click="click" v-custom="custom" />
</template>

`````

# Output

## `{"sortAttributes":{"groups":["^v-if","^v-else-if","^v-else","^v-for","^v-model","^v-","$CODE_GUIDE","^:","$DEFAULT","^tabindex","^@"],"order":"asc"}}`

`````vue
<template>
  <Widget
    @input="input"
    z="last"
    :bound-z="z"
    id="id"
    title="Title"
    v-if="show"
    v-for="item in items"
    v-model="value"
    class="classes"
    data-a="a"
    data-z="z"
    :bound-a="a"
    a="first"
    tabindex="0"
    @click="click"
    v-custom="custom"
  />
</template>

`````

## `{"sortAttributes":{"groups":["^v-if","^v-else-if","^v-else","^v-for","^v-model","^v-","$CODE_GUIDE","^:","$DEFAULT","^tabindex","^@"],"order":"desc"}}`

`````vue
<template>
  <Widget
    @input="input"
    z="last"
    :bound-z="z"
    id="id"
    title="Title"
    v-if="show"
    v-for="item in items"
    v-model="value"
    class="classes"
    data-z="z"
    data-a="a"
    :bound-a="a"
    a="first"
    tabindex="0"
    @click="click"
    v-custom="custom"
  />
</template>

`````
