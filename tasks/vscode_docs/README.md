# VSCode Docs

This task generates the Configuration section of the VSCode extension README.md from the package.json.

## Usage

### Update the README

```bash
cargo run -p vscode_docs update
# or
just vscode-docs
```

### Check if the README is up-to-date

```bash
cargo run -p vscode_docs check
```

This will be used in CI to verify that the README is always in sync with the package.json.

## How it works

1. Reads the `editors/vscode/package.json` file
2. Extracts the configuration properties from `contributes.configuration.properties`
3. Separates them into Window and Workspace configurations based on the `scope` property
4. Generates markdown tables for each configuration type
5. Replaces the content between `<!-- START_GENERATED_CONFIGURATION -->` and `<!-- END_GENERATED_CONFIGURATION -->` markers in the README

## Features

- Includes deprecated fields in a separate section with an empty row separator
- Generates properly formatted markdown tables with Key, Default Value, Possible Values, and Description columns
- Runs `just fmt` automatically after generation to ensure consistent formatting
- Includes FixKind enum values if referenced in any configuration
- Sorts configuration options alphabetically for consistent output
