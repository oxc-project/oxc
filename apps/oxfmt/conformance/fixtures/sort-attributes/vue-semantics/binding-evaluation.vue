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
