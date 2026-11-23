<template>
  <li v-for="({ a = 'k'in o }) in items">x</li>
  <li v-for="({ a = ('k'in o) }) of items">x</li>
  <li v-for="({ a=of }) in items">x</li>
</template>
