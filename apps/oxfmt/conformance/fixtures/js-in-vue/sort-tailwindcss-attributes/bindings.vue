<template>
  <div :class="['p-4 flex', { 'p-4 flex': a }, `p-4 flex ${b}`]" v-bind:class="a ? 'p-4 flex' : 'm-2 block'" :foo="'p-4 flex'" :bar="'p-4 flex'" v-baz="'p-4 flex'">{{ 'p-4 flex' }}</div>
  <p :class.prop="'p-4 flex'" .class="'p-4 flex'" />
</template>
