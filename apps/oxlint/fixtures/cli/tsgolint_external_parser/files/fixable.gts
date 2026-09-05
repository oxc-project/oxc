import Component from '@glimmer/component';

// `as number` is redundant. The fix lands in the script section, where the mapping is
// verbatim, so `--fix` rewrites it. Fixes never come back from inside `<template>`.
const n: number = 1;
const m = n as number;

export default class Fixable extends Component {
  value = m;

  <template>
    <span>{{this.value}}</span>
  </template>
}
