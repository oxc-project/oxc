import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { action } from '@ember/object';

// Intentional linting issues:
const unused_import = 'never used';  // no-unused-vars
var badVar = 'use const instead';  // no-var

interface Signature {
  Args: {
    initialValue?: number;
  };
}

export default class CounterComponent extends Component<Signature> {
  @tracked count: number = this.args.initialValue ?? 0;

  @action
  increment(): void {
    debugger;  // no-debugger
    this.count++;
    console.log('New count:', this.count);  // no-console
  }

  @action
  decrement(): void {
    this.count--;
  }

  get displayValue(): string {
    return `Count: ${this.count}`;
  }

  get isHigh(): boolean {
    return this.count > 10;
  }

  <template>
    <div class="counter">
      <h2>{{this.displayValue}}</h2>
      <button type="button" {{on "click" this.increment}}>
        Increment
      </button>
      <button type="button" {{on "click" this.decrement}}>
        Decrement
      </button>
      {{#if this.isHigh}}
        <p class="warning">Count is getting high!</p>
      {{/if}}
    </div>
  </template>
}
