# Input

`````vue
<template>
  <Widget fooBar="first" foo-bar="second" />
  <Widget foo-bar="first" fooBar="second" />
  <Widget fooBarBaz="first" foo-bar-baz="second" />
  <Widget foo1="first" foo-1="second" />
  <Widget foo_bar="first" foo-_bar="second" />
  <Widget z="last" fooBar="first" foo-bar="second" a="first" />
  <Widget z="last" foo-bar="first" a="first" />
</template>

`````

# Output

## `{"sortAttributes":true}`

`````vue
<template>
  <Widget fooBar="first" foo-bar="second" />
  <Widget foo-bar="first" fooBar="second" />
  <Widget fooBarBaz="first" foo-bar-baz="second" />
  <Widget foo1="first" foo-1="second" />
  <Widget foo_bar="first" foo-_bar="second" />
  <Widget z="last" fooBar="first" foo-bar="second" a="first" />
  <Widget a="first" foo-bar="first" z="last" />
</template>

`````

## `{"sortAttributes":{"order":"desc"}}`

`````vue
<template>
  <Widget fooBar="first" foo-bar="second" />
  <Widget foo-bar="first" fooBar="second" />
  <Widget fooBarBaz="first" foo-bar-baz="second" />
  <Widget foo1="first" foo-1="second" />
  <Widget foo_bar="first" foo-_bar="second" />
  <Widget z="last" fooBar="first" foo-bar="second" a="first" />
  <Widget z="last" foo-bar="first" a="first" />
</template>

`````

## `{"sortAttributes":{"groups":["^v-","^@","^:","$DEFAULT"]}}`

`````vue
<template>
  <Widget fooBar="first" foo-bar="second" />
  <Widget foo-bar="first" fooBar="second" />
  <Widget fooBarBaz="first" foo-bar-baz="second" />
  <Widget foo1="first" foo-1="second" />
  <Widget foo_bar="first" foo-_bar="second" />
  <Widget z="last" fooBar="first" foo-bar="second" a="first" />
  <Widget a="first" foo-bar="first" z="last" />
</template>

`````
