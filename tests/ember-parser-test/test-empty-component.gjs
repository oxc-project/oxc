import Component from '@glimmer/component';

// Test file for ember/no-empty-glimmer-component-classes
// This should trigger the rule since the class is empty (no methods, no properties)

export default class EmptyComponent extends Component {
  // Empty class - should trigger ember/no-empty-glimmer-component-classes
  <template>
    <div>Empty component</div>
  </template>
}

