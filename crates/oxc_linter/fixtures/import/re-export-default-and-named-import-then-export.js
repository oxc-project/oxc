// Import-then-export of the same named binding as both default and named.
// This should be allowed by the no-named-as-default rule because both
// the default and named exports refer to the same source binding.
import { userEvent } from './re-export-default-and-named-source';

export { userEvent as default };
export { userEvent };
