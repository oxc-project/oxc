// Re-export default and named exports through local aliases that map to
// different remote symbols. This should not be allowed by the
// no-named-as-default rule.
import userEvent, { userEvent as namedUserEvent } from './re-export-default-and-named-alias-misleading-source';

export { userEvent as default };
export { namedUserEvent as userEvent };
