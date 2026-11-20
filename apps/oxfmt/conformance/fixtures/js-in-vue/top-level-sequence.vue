<template>
  <p :class="a,b" v-show="a,b">{{ a,b }}</p>
  <p :class="(a),b" v-show="/* c */ (a),b">{{ (a,b),c }}</p>
  <p v-if="a,b" @click="a,b">{{ (a,b) }}</p>
  <p v-else-if="a,b" :class="( /* c */ a,b)">{{ ((a,b)) }}</p>
</template>
