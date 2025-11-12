import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { action } from '@ember/object';

export default class CounterComponent extends Component {
  @tracked count = 0;

  @action
  increment() {
    this.count++;
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
