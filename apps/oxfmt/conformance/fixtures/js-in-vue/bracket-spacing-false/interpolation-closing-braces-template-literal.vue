<template>
  <p>{{ `${ {a: 1} }` }}</p>
  <p>{{ `${ a || {b: 1} }` }}</p>
  <p :title="`${ {a: 1} }`">x</p>
</template>
