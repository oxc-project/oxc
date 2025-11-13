import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';

export default class DemoComponent extends Component {
  @tracked count = 0;

  // Intentional issues for demo:
  unused_variable = 'this is never used';  // unused variable

  increment() {
    debugger;  // debugger statement
    this.count++;
    var oldStyle = 'var instead of const';  // prefer const/let
    console.log('Count:', this.count);  // console.log
  }

  <template>
    <div class="demo">
      <h2>Counter: {{this.count}}</h2>
      <button type="button" {{on "click" this.increment}}>
        Increment
      </button>
      <p>Value: {{this.unused_variable}}</p>
    </div>
  </template>
}
