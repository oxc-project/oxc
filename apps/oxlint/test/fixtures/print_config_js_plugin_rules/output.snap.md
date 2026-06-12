# Exit code
0

# stdout
```
{
  "plugins": [
    "unicorn",
    "typescript",
    "oxc"
  ],
  "jsPlugins": [
    "./plugin.ts"
  ],
  "categories": {
    "correctness": "allow"
  },
  "rules": {
    "no-debugger": "warn",
    "print-config-plugin/with-options": [
      "deny",
      [
        {
          "mode": "strict"
        }
      ]
    ]
  },
  "settings": {
    "jsx-a11y": {
      "polymorphicPropName": null,
      "components": {},
      "attributes": {}
    },
    "next": {
      "rootDir": []
    },
    "react": {
      "formComponents": [],
      "linkComponents": [],
      "version": null,
      "componentWrapperFunctions": []
    },
    "jsdoc": {
      "ignorePrivate": false,
      "ignoreInternal": false,
      "ignoreReplacesDocs": true,
      "overrideReplacesDocs": true,
      "augmentsExtendsReplacesDocs": false,
      "implementsReplacesDocs": false,
      "exemptDestructuredRootsFromChecks": false,
      "tagNamePreference": {}
    },
    "vitest": {
      "typecheck": false
    },
    "jest": {
      "version": null
    }
  },
  "env": {
    "builtin": true
  },
  "globals": {},
  "ignorePatterns": []
}
```

# stderr
```
```
