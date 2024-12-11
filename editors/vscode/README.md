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
