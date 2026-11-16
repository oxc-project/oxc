<template>
  <li v-for="(item = 'a, b', i) in items">x</li>
  <li v-for="(item /* a, b */, i) in items">x</li>
</template>
