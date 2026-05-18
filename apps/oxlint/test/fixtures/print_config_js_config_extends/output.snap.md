# Exit code
0

# stdout
```
{
  "plugins": [
    "unicorn",
    "typescript",
    "oxc",
    "import"
  ],
  "categories": {
    "correctness": "allow"
  },
  "rules": {
    "import/no-unassigned-import": "deny"
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
  "overrides": [
    {
      "files": [
        "**/*.test.ts"
      ],
      "env": null,
      "globals": null,
      "plugins": null,
      "rules": {
        "no-debugger": "deny"
      }
    }
  ],
  "ignorePatterns": []
}
```

# stderr
```
```
