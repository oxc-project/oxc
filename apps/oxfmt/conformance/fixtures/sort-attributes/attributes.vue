<template>
  <Widget @update:model-value="update" tabindex="0" z="last" :title="title" v-if="visible" v-model="value" id="example" class="classes" a="first" />
  <Widget z="before" a="before" v-bind="props" y="after" b="after" />
  <Widget z="before" a="before" :[key]="value" y="after" b="after" />
  <Widget z="last" :id="first" v-bind:id="second" a="first" />
  <Widget z="last" id="first" :id="second" a="first" />
  <!-- prettier-ignore -->
  <Widget z="last"   a="first" />
</template>
