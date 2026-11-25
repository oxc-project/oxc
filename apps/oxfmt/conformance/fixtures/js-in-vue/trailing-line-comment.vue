<template>
  <li v-for="item   in items // c" :key="item">x</li>
  <a :a="{ b } // c" :b="x  +  y // c" @click="go( ) // c" :d="{
    a: 1, // c
  }" />
  <C #default="{ item } // c" />
  <p>{{ a  +  b // c }}</p>
  <a :a="{ b } // c
  " :b="x  +  y // c
  " />
  <C #default="{ item } // c
  " />
  <a :a="{ b } // c
  /* d */" />
  <C #default="{ item } // c
  /* d */" />
</template>
