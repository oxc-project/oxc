# Exit code
1

# stdout
```
Failed to parse oxlint configuration file.

  x JS plugin 'oxlint-plugin-dupe' is installed at two versions, which would both be registered under the same plugin name.
  | 
  | 1.0.0 at <fixture>/packages/a/node_modules/oxlint-plugin-dupe/index.js
  | 2.0.0 at <fixture>/packages/b/node_modules/oxlint-plugin-dupe/index.js
  | 
  | Install the same version in both places, or give one of them a different name with an alias in `jsPlugins`.
```

# stderr
```
```
