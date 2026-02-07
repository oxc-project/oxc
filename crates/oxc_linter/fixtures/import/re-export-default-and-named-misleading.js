// Re-export default and named exports from different sources. This should
// not be allowed by the no-named-as-default rule.
export { userEvent as default } from './re-export-default-and-named-source';
export { foo as userEvent } from './bar';
