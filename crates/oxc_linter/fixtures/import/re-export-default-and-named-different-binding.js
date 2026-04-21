// Re-export default and named from the same source but different bindings.
// This should not be allowed by the no-named-as-default rule.
export { otherEvent as default } from './re-export-default-and-named-different-binding-source';
export { userEvent } from './re-export-default-and-named-different-binding-source';
