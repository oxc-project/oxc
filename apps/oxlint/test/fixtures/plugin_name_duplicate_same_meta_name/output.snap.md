# Exit code
1

# stdout
```
Failed to parse configuration file.

  x Plugin name 'shared-name' is already in use.
  | 
  | Multiple plugins cannot share the same name or alias.
  | Each plugin must have a unique identifier to avoid conflicts.
  | 
  | Please provide a different alias for one of the plugins:
  | 
  | "jsPlugins": [
  |   { "name": "shared-name", "specifier": "plugin-one" },
  |   { "name": "shared-name-alt", "specifier": "plugin-two" }
  | ]
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
