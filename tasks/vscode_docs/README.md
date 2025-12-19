# VSCode Docs

This task generates the Configuration section of the VSCode extension README.md from the package.json.

## Usage

### Update the README

```bash
cargo run -p vscode_docs update
# or
just vscode-docs
```

## CI Verification

The CI workflow verifies the README is up-to-date by:

1. Running `cargo run -p vscode_docs update` to regenerate the configuration
2. Running `pnpm fmt` to format the entire project
3. Running `git diff --exit-code` to ensure no changes were made

## How it works

1. Reads the `editors/vscode/package.json` file
2. Extracts the configuration properties from `contributes.configuration.properties`
3. Separates them into Window and Workspace configurations based on the `scope` property
4. Generates markdown tables for each configuration type
5. Replaces the content between `<!-- START_GENERATED_CONFIGURATION -->` and `<!-- END_GENERATED_CONFIGURATION -->` markers in the README

## Features

- Includes deprecated fields in a separate section with an empty row separator
- Generates properly formatted markdown tables with Key, Default Value, Possible Values, and Description columns
- Includes FixKind enum values if referenced in any configuration
- Sorts configuration options alphabetically for consistent output
