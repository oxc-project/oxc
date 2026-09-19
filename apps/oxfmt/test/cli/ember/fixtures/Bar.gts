import Component from '@glimmer/component';
interface Signature { Args: { count: number } }
export default class Bar extends Component<Signature> {
<template>
      <span>{{@count}}</span>
</template>
}
