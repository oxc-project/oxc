import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { computed } from '@ember/object';
import { action } from '@ember/object';

// Test file that should PASS - no rule violations
// This demonstrates correct Ember patterns

export default class ValidComponent extends Component {
  @tracked count = 0;
  @tracked name = 'Test';
  @tracked items = [];

  // Valid computed property with all dependencies
  @computed('count', 'name')
  get displayText() {
    return `${this.name}: ${this.count}`;
  }

  // Valid computed property - just returns the items array reference
  @computed('items')
  get itemsRef() {
    // No nested property access - just return the reference
    return this.items;
  }

  // Valid action method
  @action
  increment() {
    this.count++;
  }

  // Valid action method with parameters
  @action
  updateName(newName) {
    this.name = newName;
  }

  // Valid action method that modifies array
  @action
  addItem(item) {
    this.items = [...this.items, item];
  }

  <template>
    <div class="valid-component">
      <h2>{{this.displayText}}</h2>
      <p>Items: {{this.itemCount}}</p>
      
      <button type="button" {{on "click" this.increment}}>
        Increment
      </button>
      
      <button type="button" {{on "click" (fn this.updateName "New Name")}}>
        Update Name
      </button>
    </div>
  </template>
}

