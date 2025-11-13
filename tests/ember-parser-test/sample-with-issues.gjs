import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { action } from '@ember/object';

// Intentional linting issues:
const unused_variable = 'never used';  // no-unused-vars
var oldStyleVar = 'should use let/const';  // no-var

export default class CounterComponent extends Component {
  @tracked count = 0;

  @action
  increment() {
    debugger;  // no-debugger
    this.count++;
    console.log('Count:', this.count);  // no-console
  }

  @action
  decrement() {
    this.count--;
  }

  <template>
    <div class="counter">
      <h2>Counter: {{this.count}}</h2>
      <button type="button" {{on "click" this.increment}}>
        Increment
      </button>
      <button type="button" {{on "click" this.decrement}}>
        Decrement
      </button>
    </div>
  </template>
}
