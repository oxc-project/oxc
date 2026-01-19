# PR #18229 Feedback - Summary of Changes

Hi @Sysix! I have addressed all the review feedback in the latest commit:

1. **Reverted Formatter Changes**: Reverted all unrelated changes in `oxc_formatter`. I will extract the regex support for import sorting into a separate PR as suggested.
2. **Centralized Accessibility Utilities**: Moved `INTERACTIVE_ROLES`, `NON_INTERACTIVE_ROLES`, and `is_interactive_role` to `crates/oxc_linter/src/utils/react.rs`.
3. **Refactored `interactive-supports-focus`**:
   - Added check for `disabled` and `aria-disabled` props.
   - Implemented dual diagnostics (generic and element-specific) to match upstream.
   - Improved `tabIndex` validation (handles numeric/string literals and `undefined`).
   - Cleaned up help message casing.
4. **Implementation of `no-interactive-element-to-noninteractive-role`**: Added this rule and verified it locally.
5. **Testing**: Greatly expanded the test suite for `interactive-supports-focus` using cases from the upstream ESLint plugin.

Verified all changes locally with `cargo check` and `cargo test`. All snapshots are updated. Ready for a re-review! CC @Boshen
