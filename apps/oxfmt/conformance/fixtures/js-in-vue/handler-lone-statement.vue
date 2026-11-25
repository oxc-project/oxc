<template>
  <button
    @click="foo?.bar;"
    @input="(foo?.bar);"
    @change="foo()?.bar;"
    @blur="foo?.();"
    @dblclick="a = !a;"
    @focus=""
    @keyup="  "
    @keydown=" ; "
  ></button>
</template>
