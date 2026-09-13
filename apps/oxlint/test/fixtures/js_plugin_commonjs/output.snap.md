# Exit code
1

# stdout
```
Failed to parse oxlint configuration file.

  x Failed to load JS plugin: ./plugin.js
  |   <fixture>/plugin.js:1
  | import { definePlugin } from "#oxlint/plugins";
  | ^^^^^^
  | 
  | SyntaxError: Cannot use import statement outside a module
  | 
  | Plugins must be ES modules. Add `"type": "module"` to the nearest `package.json`, or give the plugin file an `.mjs` or `.mts` extension.
```

# stderr
```
(node:XXXXX) Warning: Failed to load the ES module: <fixture>/plugin.js. Make sure to set "type": "module" in the nearest package.json file or use the .mjs extension.
(Use `node --trace-warnings ...` to show where the warning was created)
```
