import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { action } from '@ember/object';

interface Signature {
  Args: {
    initialValue?: number;
  };
}

export default class CounterComponent extends Component<Signature> {
  @tracked count: number = this.args.initialValue ?? 0;

  @action
  increment(): void {
    this.count++;
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
