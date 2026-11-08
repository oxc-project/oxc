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
    :title="title"
    @update:model-value="update"
    a="first"
    class="classes"
    id="example"
    tabindex="0"
    v-if="visible"
    v-model="value"
    z="last"
  />
  <Widget a="before" z="before" v-bind="props" b="after" y="after" />
  <Widget a="before" z="before" :[key]="value" b="after" y="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget z="last" id="first" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

## `{"sortAttributes":{"order":"desc"}}`

`````vue
<template>
  <Widget
    z="last"
    v-model="value"
    v-if="visible"
    tabindex="0"
    id="example"
    class="classes"
    a="first"
    @update:model-value="update"
    :title="title"
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
    id="example"
    class="classes"
    :title="title"
    @update:model-value="update"
    a="first"
    tabindex="0"
    v-if="visible"
    v-model="value"
    z="last"
  />
  <Widget a="before" z="before" v-bind="props" b="after" y="after" />
  <Widget a="before" z="before" :[key]="value" b="after" y="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget z="last" id="first" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

## `{"sortAttributes":{"groups":["^data-","^id$"]}}`

`````vue
<template>
  <Widget
    id="example"
    :title="title"
    @update:model-value="update"
    a="first"
    class="classes"
    tabindex="0"
    v-if="visible"
    v-model="value"
    z="last"
  />
  <Widget a="before" z="before" v-bind="props" b="after" y="after" />
  <Widget a="before" z="before" :[key]="value" b="after" y="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget z="last" id="first" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````

## `{"sortAttributes":true,"singleAttributePerLine":true,"bracketSameLine":true}`

`````vue
<template>
  <Widget
    :title="title"
    @update:model-value="update"
    a="first"
    class="classes"
    id="example"
    tabindex="0"
    v-if="visible"
    v-model="value"
    z="last" />
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
    z="last"
    id="first"
    :id="second"
    a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>

`````
