import Component from '@glimmer/component';

// `this.always` is an object, so it is always truthy: a type-aware *rule* finding
// (`no-unnecessary-condition`) anchored inside `<template>`.
export default class Widget extends Component {
  label = 'hi';
  always = { a: 1 };

  <template>
    {{#if this.always}}<span>{{this.label}}</span>{{/if}}
  </template>
}
