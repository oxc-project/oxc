import Component from '@glimmer/component';

// `cuont` is a typo for `count`, inside `<template>`. Reported as TS2551 under
// `--type-check`, over offsets in this file rather than in the mapper's output.
export default class Counter extends Component {
  count = 1;

  <template>
    <span>{{this.cuont}}</span>
  </template>
}
