# Exit code
1

# stdout
```
Failed to parse oxlint configuration file.

  [38;2;225;80;80;1m×[0m [38;2;225;80;80;1mFailed to load JS plugin: no_name
  [38;2;225;80;80;1m│[0m   Error: Plugin must either define `meta.name`, be loaded from an NPM package with a `name` field in `package.json`, or be given an alias in config
  [38;2;225;80;80;1m│[0m     at getPluginName (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:15403:8)
  [38;2;225;80;80;1m│[0m     at registerPlugin (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:15336:15)
  [38;2;225;80;80;1m│[0m     at loadPlugin (file:///Users/pwagenet/Development/OSS/oxc3/apps/oxlint/dist/lint.js:15330:35)[0m
```

# stderr
```
WARNING: JS plugins are experimental and not subject to semver.
Breaking changes are possible while JS plugins support is under development.
```
