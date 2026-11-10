# React Compiler rule migration

Oxlint 1.79.0 replaced `react/react-compiler` with individual rules. The old
`reportAllBailouts` option and its nested `environment` options are no longer
accepted on that rule. This guide applies to standalone Oxlint and to Oxlint
configuration embedded in Vite+.

## Find the effective configuration

Identify the project's installed Oxlint version, or the version bundled with
Vite+, and inspect its supported rules. Use the project's package manager and
existing lint command; do not upgrade dependencies just to migrate a config.

Search `.oxlintrc.*`, `oxlint.config.*`, `vite.config.*`, shared config imports,
`extends`, and overrides for these settings:

```sh
rg -n --hidden --glob '!node_modules/**' 'react/react-compiler|reportAllBailouts|validateNoCapitalizedCalls' .
```

For Vite+, inspect the `lint` configuration in `vite.config.*` and any Oxlint
configuration it inherits. Keep settings at their existing location, such as
`lint.rules`, rather than introducing a competing standalone config. For a shared
package, update the owned source or provide local overrides instead of editing
`node_modules`.

Record each legacy rule's effective severity and options separately for the root
config and every override. Severity may be a string, a numeric level, or the first
element of an array. Preserve unrelated plugins, categories, rules, file globs,
and override order.

## Choose the replacement policy

There is no single replacement rule or exact preset. The compiler's validations
have also changed since the combined rule shipped. Preserve the user's chosen
policy; if it is unspecified, explain these decisions before enabling additional
diagnostics:

- With `reportAllBailouts` omitted or `false`, the old rule reported only compiler
  errors and excluded invariants. Keep `react/todo`, `react/invariant`,
  `react/incompatible-library`, and `react/unsupported-syntax` off to retain that
  filtering. The latter two used to produce compiler warnings, which the old
  rule also filtered out.
- With `reportAllBailouts: true`, enable those four rules at the old rule's
  severity as well. This approximates the old reporting scope; compiler changes
  can still add or remove findings.
- `react/exhaustive-effect-dependencies` and `react/memo-dependencies` have
  validations enabled by the split rule implementation that were not enabled by
  the old combined rule. Decide whether to adopt them rather than enabling them
  silently during a compatibility migration.
- Preserve explicit settings for replacement rules that the project already
  has. Resolve conflicts with the legacy setting rather than overwriting a more
  specific policy.
- A legacy `off` setting must disable the replacement family in the same scope,
  including rules enabled by categories or inherited configs. Removing the old
  key alone can accidentally enable diagnostics.

The replacement family currently includes the following rules. Check availability
against the installed version before writing the configuration:

| Concern               | Rule names, with the `react/` prefix                                                                           |
| --------------------- | -------------------------------------------------------------------------------------------------------------- |
| Hooks and memoization | `hooks`, `use-memo`, `void-use-memo`, `preserve-manual-memoization`, `memo-dependencies`                       |
| Effects and state     | `exhaustive-effect-dependencies`, `no-deriving-state-in-effects`, `set-state-in-effect`, `set-state-in-render` |
| Render correctness    | `capitalized-calls`, `error-boundaries`, `globals`, `immutability`, `purity`, `refs`, `static-components`      |
| Compiler diagnostics  | `rule-suppression`, `syntax`, `incompatible-library`, `unsupported-syntax`, `todo`, `invariant`                |

Keep existing `react/rules-of-hooks` and `react/exhaustive-deps` settings. Those
rules have separate implementations and are not renamed by this migration.

Move `environment.validateNoCapitalizedCalls` to the `allow` option on
`react/capitalized-calls`, preserving its severity. For example:

```json
{
  "react/capitalized-calls": ["warn", { "allow": ["CustomFactory"] }]
}
```

Remove the legacy rule and its option object after expanding the selected policy.
Do not copy `reportAllBailouts` onto individual rules. Flag unknown options and
unavailable replacement rules for a decision instead of silently dropping them.

## Preserve overrides and suppressions

Expand the policy independently inside each override. For example, an override
that disabled the old rule for generated files must disable every replacement
enabled by the root or inherited configuration for those same files.

Find comments suppressing `react/react-compiler`. Replace them with the specific
rules needed at each location, preserving the comment's scope. Disabling the
whole replacement family can hide unrelated findings. The old rule also filtered
some Flow suppression comments; those comments do not provide an automatic
equivalent for the new rules. Review them explicitly.

## Validate the result

For standalone Oxlint, inspect the effective configuration for representative
files with `oxlint --print-config <file>`, then run the project's normal lint
command. For Vite+, run its existing `vp lint` command and check the effective
configuration using the installed version's supported options.

Include at least one file matched by each migrated override, plus a file outside
it. Confirm that disabled scopes remain disabled, severity levels are retained,
and the old rule and removed options no longer cause configuration errors.
Review newly surfaced findings against the selected policy; a successful config
parse alone does not establish equivalent behavior.

Report the changed config locations, any additional validations selected by the
user, and remaining behavior differences. Do not claim exact equivalence when
compiler versions or diagnostic filtering differ.

See [Oxlint configuration](https://oxc.rs/docs/guide/usage/linter/config-file-reference.html)
and the [React rule list](https://oxc.rs/docs/guide/usage/linter/rules.html).
