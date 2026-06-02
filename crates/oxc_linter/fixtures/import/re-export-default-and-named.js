// Re-export default and named exports from the same source. This
// should be allowed by the no-named-as-default rule.
export { userEvent as default } from './re-export-default-and-named-source';
export { userEvent } from './re-export-default-and-named-source';
