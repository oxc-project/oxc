import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { computed } from '@ember/object';

// Test file specifically for ember/require-computed-property-dependencies

export default class ComputedDependenciesTest extends Component {
  @tracked firstName = '';
  @tracked lastName = '';
  @tracked age = 0;
  @tracked items = [];

  // Issue 1: Missing 'lastName' dependency
  @computed('firstName')
  get fullName() {
    return `${this.firstName} ${this.lastName}`;
  }

  // Issue 2: Missing 'age' dependency
  @computed('firstName', 'lastName')
  get description() {
    return `${this.fullName} is ${this.age} years old`;
  }

  // Issue 3: Missing 'items' dependency (nested property)
  @computed('firstName')
  get itemCount() {
    return this.items.length;
  }

  // Issue 4: Missing multiple dependencies
  @computed('firstName')
  get complexComputed() {
    return `${this.firstName} ${this.lastName} - Age: ${this.age} - Items: ${this.items.length}`;
  }

  // Valid: All dependencies listed
  @computed('firstName', 'lastName')
  get validFullName() {
    return `${this.firstName} ${this.lastName}`;
  }

  // Valid: Using only listed dependencies
  @computed('age')
  get isAdult() {
    return this.age >= 18;
  }

  <template>
    <div>
      <p>{{this.fullName}}</p>
      <p>{{this.description}}</p>
      <p>Items: {{this.itemCount}}</p>
    </div>
  </template>
}

