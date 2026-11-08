<template>
  <button
    @click="foo?.bar;"
    @input="(foo?.bar);"
    @change="foo()?.bar;"
    @blur="foo?.();"
    @focus=""
    @keyup="  "
    @keydown=" ; "
  ></button>
</template>
