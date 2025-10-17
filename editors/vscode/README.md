# ⚓ Oxc

The Oxidation Compiler is creating a suite of high-performance tools for JavaScript and TypeScript.

## Installation

Any of the below options can be used to install the extension.

- Install through the VS Code extensions marketplace by searching for `Oxc`. Verify the identifier is `oxc.oxc-vscode`.
- From within VS Code, open the Quick Open (Ctrl+P or Cmd+P on macOS) and execute `ext install oxc.oxc-vscode`.

## Oxlint

This is the linter for Oxc. The currently supported features are listed below.

- Highlighting for warnings or errors identified by Oxlint
- Quick fixes to fix a warning or error when possible
- JSON schema validation for supported Oxlint configuration files (does not include ESLint configuration files)
- Command to fix all auto-fixable content within the current text editor.
- Support for `source.fixAll.oxc` as a code action provider. Configure this in your settings `editor.codeActionsOnSave`
  to automatically apply fixes when saving the file.
- Support for multi root workspaces

## Oxfmt

This is the formatter for Oxc. The currently supported features are listed below.

- Experimental formatting with `oxc.fmt.experimental`

To enable it, use a VSCode `settings.json` like:

```json
{
  "oxc.fmt.experimental": true,
  "editor.defaultFormatter": "oxc.oxc-vscode"
  // Or enable it for specific files:
  // "[javascript]": {
  //   "editor.defaultFormatter": "oxc.oxc-vscode"
  // },
}
```

## Configuration

### Window Configuration

Following configuration are supported via `settings.json` and effect the window editor:

| Key                 | Default Value | Possible Values                  | Description                                                                                  |
| ------------------- | ------------- | -------------------------------- | -------------------------------------------------------------------------------------------- |
| `oxc.enable`        | `true`        | `true` \| `false`                | Enables the language server to receive lint diagnostics                                      |
| `oxc.requireConfig` | `false`       | `true` \| `false`                | Start the language server only when a `.oxlintrc.json` file exists in one of the workspaces. |
| `oxc.trace.server`  | `off`         | `off` \| `messages` \| `verbose` | Traces the communication between VS Code and the language server.                            |
| `oxc.path.server`   | -             | `<string>`                       | Path to Oxc language server binary. Mostly for testing the language server.                  |

### Workspace Configuration

Following configuration are supported via `settings.json` and can be changed for each workspace:

| Key                           | Default Value | Possible Values             | Description                                                                                                                                      |
| ----------------------------- | ------------- | --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `oxc.lint.run`                | `onType`      | `onSave` \| `onType`        | Run the linter on save (onSave) or on type (onType)                                                                                              |
| `oxc.configPath`              | `null`        | `null` \| `<string>`        | Path to ESlint configuration. Keep it empty to enable nested configuration.                                                                      |
| `oxc.tsConfigPath`            | `null`        | `null` \| `<string>`        | Path to TypeScript configuration. If your `tsconfig.json` is not at the root, alias paths will not be resolve correctly for the `import` plugin. |
| `oxc.unusedDisableDirectives` | `allow`       | `allow` \| `warn` \| `deny` | Define how directive comments like `// oxlint-disable-line` should be reported, when no errors would have been reported on that line anyway.     |
| `oxc.typeAware`               | `false`       | `false` \| `true`           | Enable type aware linting.                                                                                                                       |
| `oxc.flags`                   | -             | `Record<string, string>`    | Custom flags passed to the language server.                                                                                                      |
| `oxc.fmt.experimental`        | `false`       | `false` \| `true`           | Enable experimental formatting support. This feature is experimental and might not work as expected.                                             |
| `oxc.fmt.configPath`          | `null`        | `<string>` \| `null`        | Path to an oxfmt configuration file. When `null`, the server will use `.oxfmtrc.json` at the workspace root.                                     |

#### Flags

- `key: disable_nested_config`: Disabled nested configuration and searches only for `configPath`
- `key: fix_kind`: default: `"safe_fix"`, possible values `"safe_fix" | "safe_fix_or_suggestion" | "dangerous_fix" | "dangerous_fix_or_suggestion" | "none" | "all"`

## Testing

Run `pnpm server:build:debug` to build the language server.
After that, you can test the vscode plugin + E2E Tests with `pnpm test`.
