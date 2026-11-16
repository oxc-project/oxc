<template>
  <Widget @input="input" z="last" :bound-z="z" title="Title" id="id" v-if="show" v-for="item in items" v-model="value" class="classes" data-z="z" data-a="a" :bound-a="a" a="first" tabindex="0" @click="click" v-custom="custom" />
</template>
