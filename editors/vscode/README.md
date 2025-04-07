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

## Configuration

Following configuration are supported via `settings.json`:

| Key                | Default Value | Possible Values                  | Description                                                                 |
| ------------------ | ------------- | -------------------------------- | --------------------------------------------------------------------------- |
| `oxc.lint.run`     | `onType`      | `onSave` \| `onType`             | Run the linter on save (onSave) or on type (onType)                         |
| `oxc.enable`       | `true`        | `true` \| `false`                | Enables the language server to receive lint diagnostics                     |
| `oxc.trace.server` | `off`         | `off` \| `messages` \| `verbose` | races the communication between VS Code and the language server.            |
| `oxc.configPath`   | `null`        | `null`\| `<string>`              | Path to ESlint configuration. Keep it empty to enable nested configuration. |
| `oxc.path.server`  | -             | `<string>`                       | Path to Oxc language server binary. Mostly for testing the language server. |
| `oxc.flags`        | -             | `Record<string, string>`         | Specific Oxlint flags to pass to the language server.                       |

### Flags

- `key: disable_nested_config`: Disabled nested configuration and searches only for `configPath`
- `key: fix_kind`: default: `"safe_fix"`, possible values `"safe_fix" | "safe_fix_or_suggestion" | "dangerous_fix" | "dangerous_fix_or_suggestion" | "none" | "all"`

## Testing

Run `pnpm server:build:debug` to build the language server.
After that, you can test the vscode plugin + E2E Tests with `pnm test`
