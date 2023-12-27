```toml
[language-server]
oxc_language_server = { command = "oxc_language_server" }

[[language]]
name = "javascript"
auto-format = true
comment-token = "//"
file-types = ["js", "mjs", "cjs"]
injection-regex = "(js|javascript)"
language-id = "javascript"
language-servers = ["typescript-language-server", "oxc_language_server"]
roots = []
scope = "source.js"
shebangs = ["node"]

[language.indent]
tab-width = 2
unit = "  "

[[language]]
name = "typescript"
auto-format = true
file-types = ["ts", "mts", "cts"]
injection-regex = "(ts|typescript)"
language-id = "typescript"
language-servers = ["typescript-language-server", "oxc_language_server"]
roots = []
scope = "source.ts"
shebangs = []

[language.indent]
tab-width = 2
unit = "  "

[[language]]
name = "tsx"
auto-format = true
file-types = ["tsx"]
injection-regex = "(tsx)"
language-id = "typescriptreact"
language-servers = ["typescript-language-server", "oxc_language_server"]
roots = []
scope = "source.tsx"

[language.indent]
tab-width = 2
unit = "  "

[[language]]
name = "jsx"
auto-format = true
comment-token = "//"
file-types = ["jsx"]
grammar = "javascript"
injection-regex = "jsx"
language-id = "javascriptreact"
language-servers = ["typescript-language-server", "oxc_language_server"]
roots = []
scope = "source.jsx"

[language.indent]
tab-width = 2
unit = "  "

```
