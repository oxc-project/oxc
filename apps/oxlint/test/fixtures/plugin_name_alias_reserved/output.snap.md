# Exit code
1

# stdout
```
Failed to parse oxlint configuration file.

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mPlugin name 'jsdoc' is reserved, and cannot be used for JS plugins.
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m The 'jsdoc' plugin is already implemented natively in Rust within oxlint.
  [38;2;225;80;80;1m│[0m Using both the native and JS versions would create ambiguity about which rules to use.
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m To use an external 'jsdoc' plugin instead, provide a custom alias:
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m "jsPlugins": [{ "name": "jsdoc-js", "specifier": "eslint-plugin-jsdoc" }]
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m Then reference rules using your alias:
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m "rules": {
  [38;2;225;80;80;1m│[0m   "jsdoc-js/rule-name": "error"
  [38;2;225;80;80;1m│[0m }
  [38;2;225;80;80;1m│[0m 
  [38;2;225;80;80;1m│[0m See: https://oxc.rs/docs/guide/usage/linter/js-plugins.html[0m
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
