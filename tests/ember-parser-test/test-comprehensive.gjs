import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { computed } from '@ember/object';
import { action } from '@ember/object';
import { inject as service } from '@ember/service';

// Test file with multiple ember rule violations

export default class ComprehensiveTestComponent extends Component {
  @service router;
  @service store;

  @tracked count = 0;
  @tracked name = 'Test';
  @tracked items = [];

  // Issue 1: Missing dependencies in computed property
  // ember/require-computed-property-dependencies
  @computed('count')
  get total() {
    return this.count * 2 + this.name.length; // Uses 'name' but not in dependencies
  }

  // Issue 2: Missing dependencies - nested property access
  @computed('items')
  get itemCount() {
    return this.items.length + this.count; // Uses 'count' but not in dependencies
  }

  // Issue 3: Empty Glimmer component class (if no methods/actions)
  // This class has methods, so it shouldn't trigger no-empty-glimmer-component-classes

  // Issue 4: Using .get() method (if applicable)
  // ember/no-get
  getValue() {
    const model = this.store.peekRecord('user', 1);
    if (model) {
      // This pattern might trigger no-get if the rule checks for .get() usage
      return model.get('name');
    }
    return null;
  }

  // Issue 5: Using .set() method (ember/no-set-in-getter or similar)
  @computed('count')
  get computedWithSideEffect() {
    // This is a computed property that shouldn't have side effects
    this.name = 'changed'; // Side effect in getter
    return this.count * 2;
  }

  // Valid action method
  @action
  increment() {
    this.count++;
  }

  // Valid action method
  @action
  updateName(newName) {
    this.name = newName;
  }

  <template>
    <div class="comprehensive-test">
      <h1>Comprehensive Test Component</h1>
      <p>Count: {{this.count}}</p>
      <p>Total: {{this.total}}</p>
      <p>Item Count: {{this.itemCount}}</p>
      <p>Name: {{this.name}}</p>
      
      <button type="button" {{on "click" this.increment}}>
        Increment
      </button>
      
      <button type="button" {{on "click" (fn this.updateName "New Name")}}>
        Update Name
      </button>
    </div>
  </template>
}

