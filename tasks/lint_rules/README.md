# tasks/lint_rules

Task to update implementation progress for each linter plugin.

```sh
Usage:
  $ cargo run <plugin_name> [--update]

Arguments:
  plugin_name: Name of the target plugin

Options:
  --update: Update GitHub issue comment
  -h, --help: Show this message
```

Environment variables `GITHUB_TOKEN` is required when `--update` is specified.
