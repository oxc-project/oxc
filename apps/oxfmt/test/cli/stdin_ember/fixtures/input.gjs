import Component from '@glimmer/component';
export default class Foo extends Component {
<template>
      <button {{on "click" this.onClick}}>{{yield}}</button>
</template>
}
