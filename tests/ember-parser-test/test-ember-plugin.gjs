import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { computed } from '@ember/object';

export default class TestComponent extends Component {
  @tracked count = 0;
  @tracked name = 'Test';

  // Issue: Missing dependencies in computed (ember/require-computed-macros-dependencies)
  // This computed property uses 'name' but doesn't list it in dependencies
  @computed('count')
  get doubleCount() {
    // Should depend on 'name' too if used
    return this.count * 2 + this.name.length;
  }

  // Issue: Using .get() (ember/no-get) - if this rule exists
  getValue() {
    const obj = { value: 42 };
    // Note: This might not work in modern JS, but if ember/no-get exists, it should catch this pattern
    if (obj.get) {
      return obj.get('value');
    }
    return obj.value;
  }

  <template>
    <div class="test">
      <h2>Count: {{this.count}}</h2>
      <p>Double: {{this.doubleCount}}</p>
      <p>Name: {{this.name}}</p>
    </div>
  </template>
}

