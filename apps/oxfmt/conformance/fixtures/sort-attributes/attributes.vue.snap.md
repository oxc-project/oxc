# Input

`````vue
<template>
  <Widget @update:model-value="update" tabindex="0" z="last" :title="title" v-if="visible" v-model="value" id="example" class="classes" a="first" />
  <Widget z="before" a="before" v-bind="props" y="after" b="after" />
  <Widget z="before" a="before" :[key]="value" y="after" b="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget z="last" id="first" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

# Output

## `{"sortAttributes":true}`

`````vue
<template>
  <Widget
    @update:model-value="update"
    tabindex="0"
    z="last"
    :title="title"
    v-if="visible"
    v-model="value"
    a="first"
    class="classes"
    id="example"
  />
  <Widget a="before" z="before" v-bind="props" b="after" y="after" />
  <Widget a="before" z="before" :[key]="value" b="after" y="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget id="first" z="last" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

## `{"sortAttributes":{"order":"desc"}}`

`````vue
<template>
  <Widget
    @update:model-value="update"
    z="last"
    tabindex="0"
    :title="title"
    v-if="visible"
    v-model="value"
    id="example"
    class="classes"
    a="first"
  />
  <Widget z="before" a="before" v-bind="props" y="after" b="after" />
  <Widget z="before" a="before" :[key]="value" y="after" b="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget z="last" id="first" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

## `{"sortAttributes":{"groups":["^id$","^class(?:Name)?$","$DEFAULT","^data-","^aria-"],"order":"asc"}}`

`````vue
<template>
  <Widget
    @update:model-value="update"
    tabindex="0"
    z="last"
    :title="title"
    v-if="visible"
    v-model="value"
    id="example"
    class="classes"
    a="first"
  />
  <Widget a="before" z="before" v-bind="props" b="after" y="after" />
  <Widget a="before" z="before" :[key]="value" b="after" y="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget id="first" z="last" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

## `{"sortAttributes":{"groups":["^data-","^id$"]}}`

`````vue
<template>
  <Widget
    @update:model-value="update"
    tabindex="0"
    z="last"
    :title="title"
    v-if="visible"
    v-model="value"
    id="example"
    a="first"
    class="classes"
  />
  <Widget a="before" z="before" v-bind="props" b="after" y="after" />
  <Widget a="before" z="before" :[key]="value" b="after" y="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget id="first" z="last" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

## `{"sortAttributes":true,"singleAttributePerLine":true,"bracketSameLine":true}`

`````vue
<template>
  <Widget
    @update:model-value="update"
    tabindex="0"
    z="last"
    :title="title"
    v-if="visible"
    v-model="value"
    a="first"
    class="classes"
    id="example" />
  <Widget
    a="before"
    z="before"
    v-bind="props"
    b="after"
    y="after" />
  <Widget
    a="before"
    z="before"
    :[key]="value"
    b="after"
    y="after" />
  <Widget
    z="last"
    :id="first"
    v-bind:id="second"
    a="first" />
  <Widget
    id="first"
    z="last"
    :id="second"
    a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````
